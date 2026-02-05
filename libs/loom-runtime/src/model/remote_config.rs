use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationConfig;
use serde::{Deserialize, Serialize};
use tch::Device;

use super::ModelType;

/// Configuration for a remotely fetched model (uses default resources)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteModelConfig {
    /// Model architecture type
    pub model: ModelType,

    /// Whether to lowercase all input text
    #[serde(default)]
    pub lower_case: Option<bool>,

    /// Whether to strip accents from input text
    #[serde(default)]
    pub strip_accents: Option<bool>,

    /// Whether to add a prefix space to input text
    #[serde(default)]
    pub add_prefix_space: Option<bool>,
}

impl From<RemoteModelConfig> for ZeroShotClassificationConfig {
    fn from(config: RemoteModelConfig) -> Self {
        let mut zs_config = Self::default();
        zs_config.model_type = config.model.into();
        zs_config.lower_case = config.lower_case.unwrap_or(false);
        zs_config.strip_accents = config.strip_accents;
        zs_config.add_prefix_space = config.add_prefix_space;
        zs_config.device = Device::cuda_if_available();
        zs_config
    }
}

/// Builder for creating remote model configurations
#[derive(Debug, Clone, Default)]
pub struct RemoteModelConfigBuilder {
    model: ModelType,
    lower_case: Option<bool>,
    strip_accents: Option<bool>,
    add_prefix_space: Option<bool>,
}

impl RemoteModelConfigBuilder {
    /// Create a new remote model configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the model architecture type
    pub fn model(mut self, model: ModelType) -> Self {
        self.model = model;
        self
    }

    /// Set whether to lowercase all input text
    pub fn lower_case(mut self, lower_case: bool) -> Self {
        self.lower_case = Some(lower_case);
        self
    }

    /// Set whether to strip accents from input text
    pub fn strip_accents(mut self, strip_accents: bool) -> Self {
        self.strip_accents = Some(strip_accents);
        self
    }

    /// Set whether to add a prefix space to input text
    pub fn add_prefix_space(mut self, add_prefix_space: bool) -> Self {
        self.add_prefix_space = Some(add_prefix_space);
        self
    }

    /// Build the remote model configuration
    pub fn build(self) -> RemoteModelConfig {
        RemoteModelConfig {
            model: self.model,
            lower_case: self.lower_case,
            strip_accents: self.strip_accents,
            add_prefix_space: self.add_prefix_space,
        }
    }
}
