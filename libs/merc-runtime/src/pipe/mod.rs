pub mod operators;
mod source;
mod transformer;

pub use source::*;
pub use transformer::*;

pub trait Operator<Input> {
    type Output;

    fn apply(self, src: Source<Input>) -> Source<Self::Output>;
}
