use std::any::TypeId;
use std::collections::HashMap;

use loom_error::{Error, ErrorCode, Result};
use loom_pipe::{AnyLayer, Layer, LayerNode};

/// Registry for storing and retrieving layers by name.
pub struct LayerRegistry {
    layers: HashMap<String, Box<dyn AnyLayer>>,
}

impl LayerRegistry {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    /// Register a layer using its name() as the key.
    pub fn register<L>(&mut self, layer: L)
    where
        L: Layer + Sync + 'static,
        L::Input: 'static,
        L::Output: 'static,
    {
        let name = layer.name().to_string();
        self.layers.insert(name, Box::new(LayerNode::new(layer)));
    }

    /// Get a layer by name.
    pub fn get(&self, name: &str) -> Option<&dyn AnyLayer> {
        self.layers.get(name).map(|l| l.as_ref())
    }

    /// Get a layer by name with type checking.
    pub fn get_checked(
        &self,
        name: &str,
        input_type: TypeId,
        output_type: TypeId,
    ) -> Result<&dyn AnyLayer> {
        let layer = self.get(name).ok_or_else(|| {
            Error::builder()
                .code(ErrorCode::NotFound)
                .message(format!("Layer '{}' not found", name))
                .build()
        })?;

        if layer.input_type_id() != input_type {
            return Err(Error::builder()
                .code(ErrorCode::BadArguments)
                .message(format!("Layer '{}' input type mismatch", name))
                .build());
        }

        if layer.output_type_id() != output_type {
            return Err(Error::builder()
                .code(ErrorCode::BadArguments)
                .message(format!("Layer '{}' output type mismatch", name))
                .build());
        }

        Ok(layer)
    }
}

impl Default for LayerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
