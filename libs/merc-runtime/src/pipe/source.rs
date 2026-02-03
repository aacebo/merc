use crate::pipe::Operator;

pub struct Source<T> {
    handler: Box<dyn FnOnce() -> T>,
}

impl<T> Source<T> {
    pub fn new<Fn: FnOnce() -> T + 'static>(handler: Fn) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }

    pub fn run(self) -> T {
        (self.handler)()
    }

    pub fn pipe<Op: Operator<T>>(self, op: Op) -> Source<Op::Output> {
        op.apply(self)
    }
}

impl<T: 'static> From<T> for Source<T> {
    fn from(value: T) -> Self {
        Self {
            handler: Box::new(|| value),
        }
    }
}
