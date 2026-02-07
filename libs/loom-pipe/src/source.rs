use crate::{Build, Operator, Pipe};

pub struct Source<T> {
    handler: Box<dyn FnOnce() -> T + Send>,
}

impl<T> Source<T> {
    pub fn new<Fn: FnOnce() -> T + Send + 'static>(handler: Fn) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }
}

impl<T: Send + 'static> From<T> for Source<T> {
    fn from(value: T) -> Self {
        Self {
            handler: Box::new(|| value),
        }
    }
}

impl<T: 'static> Pipe<T> for Source<T> {
    fn pipe<Op: Operator<T>>(self, op: Op) -> Source<Op::Output> {
        op.apply(self)
    }
}

impl<T> Build for Source<T> {
    type Output = T;

    fn build(self) -> Self::Output {
        (self.handler)()
    }
}
