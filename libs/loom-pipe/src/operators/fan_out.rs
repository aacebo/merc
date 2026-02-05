use crate::{Build, Operator, Source};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;
    use crate::operators::{Filter, Guard};

    #[test]
    fn single_branch() {
        let result = Source::from(5)
            .pipe(FanOut::new().add(Guard::allow(|x| *x > 0)))
            .build();
        assert_eq!(result, vec![Some(5)]);
    }

    #[test]
    fn multiple_branches() {
        let result = Source::from(10)
            .pipe(
                FanOut::new()
                    .add(Guard::allow(|x| *x > 5))
                    .add(Guard::allow(|x| *x > 15))
                    .add(Guard::block(|x| *x > 5)),
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
    fn with_filter() {
        let results = Source::from(vec![1, 2, 3, 4, 5])
            .pipe(
                FanOut::new()
                    .add(Filter::new(|x| *x > 3))
                    .add(Filter::new(|x| *x < 3)),
            )
            .build();
        assert_eq!(results, vec![vec![4, 5], vec![1, 2]]);
    }
}
