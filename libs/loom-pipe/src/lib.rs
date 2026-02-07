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
    // Logical
    And,
    // Existing
    AwaitPipe,
    // Branch
    BranchBuilderComplete,
    BranchBuilderInit,
    BranchBuilderWithCondition,
    BranchBuilderWithThen,
    BranchPipe,
    // Sequence
    Chunk,
    Concat,
    // Time
    Delay,
    // Result/Option
    Expect,
    FanOutBuilder,
    FanOutPipe,
    FilterPipe,
    FlatMap,
    Flatten,
    ForkPipe,
    LogicalPipe,
    MapPipe,
    OptionExpect,
    OptionOkOr,
    OptionPipe,
    OptionUnwrap,
    OptionUnwrapOr,
    OptionUnwrapOrElse,
    Or,
    OrElseMap,
    ParallelBuilder,
    ParallelPipe,
    ResultOk,
    ResultPipe,
    Retry,
    RetryBuilder,
    RetryPipe,
    RouterBuilder,
    RouterPipe,
    SequencePipe,
    TimePipe,
    Timeout,
    TimeoutError,
    TryMapPipe,
    Unwrap,
    UnwrapOr,
    UnwrapOrElse,
    Window,
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
