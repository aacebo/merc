// Existing lazy/pull-based pipeline system
pub mod operators;
pub mod pipeline;
mod source;
mod transformer;

pub use pipeline::{Layer, LayerContext, LayerResult, Pipeline, PipelineBuilder};
pub use source::*;
pub use transformer::*;

// Re-export extension traits for convenience
pub use operators::{
    AwaitPipe, FanOutBuilder, FanOutPipe, FilterPipe, MapPipe, ParallelBuilder, ParallelPipe,
    RouterBuilder, RouterPipe, SpawnPipe, TryMapPipe,
};

pub trait Operator<Input> {
    type Output;

    fn apply(self, src: Source<Input>) -> Source<Self::Output>;
}

pub trait Pipe<Input> {
    fn pipe<Op: Operator<Input>>(self, op: Op) -> Source<Op::Output>;
}

pub trait Build {
    type Output;

    fn build(self) -> Self::Output;
}
