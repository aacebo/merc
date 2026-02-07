use std::any::{Any, TypeId};

use loom_error::{Error, ErrorCode, Result};

use super::Layer;

/// Type-erased layer for dynamic pipeline construction
pub trait AnyLayer: Send + Sync {
    fn process_any(&self, input: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>>;
    fn name(&self) -> &'static str;
    fn input_type_id(&self) -> TypeId;
    fn output_type_id(&self) -> TypeId;
}

/// Wrapper that implements AnyLayer for any Layer
pub struct LayerNode<L: Layer> {
    layer: L,
}

impl<L: Layer> LayerNode<L> {
    pub fn new(layer: L) -> Self {
        Self { layer }
    }
}

impl<L: Layer + Sync> AnyLayer for LayerNode<L>
where
    L::Input: 'static,
    L::Output: 'static,
{
    fn process_any(&self, input: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        let typed_input = input.downcast::<L::Input>().map_err(|_| {
            Error::builder()
                .code(ErrorCode::BadArguments)
                .message("Type mismatch in pipeline")
                .build()
        })?;

        let result = self.layer.process(*typed_input)?;
        // Extract the inner output from LayerResult
        Ok(Box::new(result.output))
    }

    fn name(&self) -> &'static str {
        self.layer.name()
    }

    fn input_type_id(&self) -> TypeId {
        TypeId::of::<L::Input>()
    }

    fn output_type_id(&self) -> TypeId {
        TypeId::of::<L::Output>()
    }
}
