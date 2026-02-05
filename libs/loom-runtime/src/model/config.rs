use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationConfig;
use serde::{Deserialize, Serialize};

use super::{
    LocalModelConfig, LocalModelConfigBuilder, ModelType, RemoteModelConfig,
    RemoteModelConfigBuilder,
};

/// Configuration for a model, either local or remote
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelConfig {
    Local(LocalModelConfig),
    Remote(RemoteModelConfig),
}

impl ModelConfig {
    /// Returns true if this is a local model configuration
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local(_))
    }

    /// Returns true if this is a remote model configuration
    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Remote(_))
    }

    /// Returns the model type for this configuration
    pub fn model_type(&self) -> &ModelType {
        match self {
            Self::Local(local) => &local.model,
            Self::Remote(remote) => &remote.model,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::Remote(RemoteModelConfig::default())
    }
}

impl From<ModelConfig> for ZeroShotClassificationConfig {
    fn from(config: ModelConfig) -> Self {
        match config {
            ModelConfig::Local(local) => local.into(),
            ModelConfig::Remote(remote) => remote.into(),
        }
    }
}

impl From<LocalModelConfig> for ModelConfig {
    fn from(config: LocalModelConfig) -> Self {
        Self::Local(config)
    }
}

impl From<RemoteModelConfig> for ModelConfig {
    fn from(config: RemoteModelConfig) -> Self {
        Self::Remote(config)
    }
}

/// Builder for creating model configurations
#[derive(Debug, Clone, Default)]
pub struct ModelConfigBuilder {
    model: ModelType,
    lower_case: Option<bool>,
    strip_accents: Option<bool>,
    add_prefix_space: Option<bool>,
}

impl ModelConfigBuilder {
    /// Create a new model configuration builder
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

    /// Convert to a local model configuration builder
    pub fn local(self) -> LocalModelConfigBuilder {
        let mut builder = LocalModelConfigBuilder::new().model(self.model);

        if let Some(lower_case) = self.lower_case {
            builder = builder.lower_case(lower_case);
        }

        if let Some(strip_accents) = self.strip_accents {
            builder = builder.strip_accents(strip_accents);
        }

        if let Some(add_prefix_space) = self.add_prefix_space {
            builder = builder.add_prefix_space(add_prefix_space);
        }

        builder
    }

    /// Convert to a remote model configuration builder
    pub fn remote(self) -> RemoteModelConfigBuilder {
        let mut builder = RemoteModelConfigBuilder::new().model(self.model);

        if let Some(lower_case) = self.lower_case {
            builder = builder.lower_case(lower_case);
        }

        if let Some(strip_accents) = self.strip_accents {
            builder = builder.strip_accents(strip_accents);
        }

        if let Some(add_prefix_space) = self.add_prefix_space {
            builder = builder.add_prefix_space(add_prefix_space);
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_local_creates_local_builder() {
        let config = ModelConfigBuilder::new()
            .model(ModelType::Bert)
            .local()
            .weights("/path/to/model.ot")
            .config("/path/to/config.json")
            .vocab("/path/to/vocab.txt")
            .build();
        assert_eq!(config.model, ModelType::Bert);
    }

    #[test]
    fn builder_remote_creates_remote_builder() {
        let config = ModelConfigBuilder::new()
            .model(ModelType::Bert)
            .remote()
            .build();
        assert_eq!(config.model, ModelType::Bert);
    }

    #[test]
    fn local_config_converts_to_model_config() {
        let local = LocalModelConfigBuilder::new()
            .weights("/path/to/model.ot")
            .config("/path/to/config.json")
            .vocab("/path/to/vocab.txt")
            .build();
        let config: ModelConfig = local.into();
        assert!(matches!(config, ModelConfig::Local(_)));
    }

    #[test]
    fn remote_config_converts_to_model_config() {
        let remote = RemoteModelConfigBuilder::new().build();
        let config: ModelConfig = remote.into();
        assert!(matches!(config, ModelConfig::Remote(_)));
    }

    #[test]
    fn default_model_type_is_bart() {
        let config = ModelConfigBuilder::new().remote().build();
        assert_eq!(config.model, ModelType::Bart);
    }
}
