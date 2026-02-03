use crate::pipe::{Operator, Source};

pub struct MapOperator<Input, Output> {
    handler: Box<dyn FnOnce(Input) -> Output>,
}

impl<Input, Output> MapOperator<Input, Output> {
    pub fn new<Handler: FnOnce(Input) -> Output + 'static>(handler: Handler) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }
}

impl<Input: 'static, Output: 'static> Operator<Input> for MapOperator<Input, Output> {
    type Output = Output;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(|| (self.handler)(src.run()))
    }
}

impl<T: 'static> Source<T> {
    pub fn map<Output: 'static, Handler: FnOnce(T) -> Output + 'static>(
        self,
        handler: Handler,
    ) -> Source<Output> {
        self.pipe(MapOperator::new(handler))
    }
}
