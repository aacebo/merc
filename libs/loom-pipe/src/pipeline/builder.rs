use super::{Layer, LayerNode, Pipeline, PipelineStage};

/// Builder for constructing type-safe pipelines
pub struct PipelineBuilder<Input, Output> {
    stages: Vec<PipelineStage>,
    _marker: std::marker::PhantomData<fn(Input) -> Output>,
}

impl<Input: Send + 'static> PipelineBuilder<Input, Input> {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Input: Send + 'static> Default for PipelineBuilder<Input, Input> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Input: Send + 'static, Current: Send + 'static> PipelineBuilder<Input, Current> {
    /// Add a layer to the pipeline, transforming Current -> L::Output
    pub fn then<L>(self, layer: L) -> PipelineBuilder<Input, L::Output>
    where
        L: Layer + Sync + 'static,
        L::Input: From<Current>,
    {
        let mut stages = self.stages;
        stages.push(PipelineStage::Layer(Box::new(LayerNode::new(layer))));

        PipelineBuilder {
            stages,
            _marker: std::marker::PhantomData,
        }
    }

    /// Build the final pipeline
    pub fn build(self) -> Pipeline<Input, Current> {
        Pipeline::new(self.stages)
    }
}
