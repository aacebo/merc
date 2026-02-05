use crate::{Build, Operator, Pipe, Source};

/// Fan-out: send the same input to multiple operators, collect all outputs
pub struct FanOut<Input, Output> {
    branches: Vec<Box<dyn FnOnce(Source<Input>) -> Source<Output> + Send>>,
    _marker: std::marker::PhantomData<fn(Input) -> Output>,
}

impl<Input, Output> FanOut<Input, Output>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add<Op>(mut self, op: Op) -> Self
    where
        Op: Operator<Input, Output = Output> + Send + 'static,
    {
        self.branches
            .push(Box::new(move |src: Source<Input>| op.apply(src)));
        self
    }
}

impl<Input, Output> Default for FanOut<Input, Output>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output> Operator<Input> for FanOut<Input, Output>
where
    Input: Clone + Send + 'static,
    Output: Send + 'static,
{
    type Output = Vec<Output>;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(move || {
            let input = src.build();
            self.branches
                .into_iter()
                .map(|branch| {
                    let cloned_input = input.clone();
                    branch(Source::from(cloned_input)).build()
                })
                .collect()
        })
    }
}

/// Extension trait for starting fan-out execution
pub trait FanOutPipe<T>: Pipe<T> + Sized
where
    T: Clone + Send + 'static,
{
    fn fan_out<O: Send + 'static>(self) -> FanOutBuilder<T, O, Self> {
        FanOutBuilder::new(self)
    }
}

impl<T: Clone + Send + 'static, P: Pipe<T> + Sized> FanOutPipe<T> for P {}

/// Builder for fan-out execution that implements Build and Pipe
pub struct FanOutBuilder<T, O, P> {
    source: P,
    fan_out: FanOut<T, O>,
}

impl<T, O, P> FanOutBuilder<T, O, P>
where
    T: Clone + Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
{
    fn new(source: P) -> Self {
        Self {
            source,
            fan_out: FanOut::new(),
        }
    }

    /// Add a branch operator
    pub fn add<Op>(mut self, op: Op) -> Self
    where
        Op: Operator<T, Output = O> + Send + 'static,
    {
        self.fan_out = self.fan_out.add(op);
        self
    }
}

impl<T, O, P> Build for FanOutBuilder<T, O, P>
where
    T: Clone + Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
{
    type Output = Vec<O>;

    fn build(self) -> Self::Output {
        self.source.pipe(self.fan_out).build()
    }
}

impl<T, O, P> Pipe<Vec<O>> for FanOutBuilder<T, O, P>
where
    T: Clone + Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
{
    fn pipe<Op: Operator<Vec<O>>>(self, op: Op) -> Source<Op::Output> {
        self.source.pipe(self.fan_out).pipe(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;
    use crate::operators::Filter;

    #[test]
    fn single_branch() {
        let result = Source::from(5)
            .pipe(FanOut::new().add(Filter::allow(|x| *x > 0)))
            .build();
        assert_eq!(result, vec![Some(5)]);
    }

    #[test]
    fn multiple_branches() {
        let result = Source::from(10)
            .pipe(
                FanOut::new()
                    .add(Filter::allow(|x| *x > 5))
                    .add(Filter::allow(|x| *x > 15))
                    .add(Filter::block(|x| *x > 5)),
            )
            .build();
        assert_eq!(result, vec![Some(10), None, None]);
    }

    #[test]
    fn no_branches() {
        let result = Source::from(42)
            .pipe(FanOut::<i32, Option<i32>>::new())
            .build();
        assert!(result.is_empty());
    }

    #[test]
    fn default_is_empty() {
        let fan_out: FanOut<i32, Option<i32>> = FanOut::default();
        let result = Source::from(42).pipe(fan_out).build();
        assert!(result.is_empty());
    }

    #[test]
    fn fan_out_pipe_trait() {
        let results = Source::from(10)
            .fan_out()
            .add(Filter::allow(|x| *x > 5))
            .add(Filter::allow(|x| *x > 15))
            .build();
        assert_eq!(results, vec![Some(10), None]);
    }
}
