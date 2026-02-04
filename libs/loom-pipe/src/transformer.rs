use crate::{Build, Operator, Pipe, Source};

pub struct Transformer<Input, Output> {
    source: Source<Input>,
    handler: Box<dyn FnOnce(Input) -> Output>,
}

impl<Input, Output> Transformer<Input, Output> {
    pub fn new<Fn: FnOnce(Input) -> Output + 'static>(source: Source<Input>, handler: Fn) -> Self {
        Self {
            source,
            handler: Box::new(handler),
        }
    }
}

impl<Input, Output> Pipe<Input> for Transformer<Input, Output> {
    fn pipe<Op: Operator<Input>>(self, op: Op) -> Source<Op::Output> {
        op.apply(self.source)
    }
}

impl<Input, Output> Build for Transformer<Input, Output> {
    type Output = Output;

    fn build(self) -> Self::Output {
        (self.handler)(self.source.build())
    }
}
