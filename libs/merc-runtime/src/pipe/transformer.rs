use crate::pipe::{Operator, Source};

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

    pub fn run(self) -> Output {
        (self.handler)(self.source.run())
    }

    pub fn pipe<Op: Operator<Input>>(self, op: Op) -> Source<Op::Output> {
        op.apply(self.source)
    }
}
