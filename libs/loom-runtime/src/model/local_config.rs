use std::path::PathBuf;

use rust_bert::pipelines::common::ModelResource;
use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationConfig;
use rust_bert::resources::LocalResource;
use serde::{Deserialize, Serialize};

use super::ModelType;

/// Configuration for a locally stored model
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalModelConfig {
    /// Model architecture type
    #[serde(default)]
    pub model: ModelType,

    /// Path to model weights file
    #[serde(default)]
    pub weights: Option<PathBuf>,

    /// Path to model configuration file
    #[serde(default)]
    pub config: Option<PathBuf>,

    /// Path to vocabulary file
    #[serde(default)]
    pub vocab: Option<PathBuf>,

    /// Path to merges file (required for some tokenizers like RoBERTa)
    #[serde(default)]
    pub merges: Option<PathBuf>,

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

impl From<LocalModelConfig> for ZeroShotClassificationConfig {
    fn from(config: LocalModelConfig) -> Self {
        // If all required paths are provided, use local resources
        // Otherwise fall back to rust_bert defaults for the model type
        match (config.weights, config.config, config.vocab) {
            (Some(weights), Some(cfg), Some(vocab)) => Self::new(
                config.model.into(),
                ModelResource::Torch(Box::new(LocalResource::from(weights))),
                LocalResource::from(cfg),
                LocalResource::from(vocab),
                config.merges.map(LocalResource::from),
                config.lower_case.unwrap_or(false),
                config.strip_accents,
                config.add_prefix_space,
            ),
            _ => {
                // Fall back to default remote resources
                let mut zs_config = Self::default();
                zs_config.model_type = config.model.into();
                zs_config.lower_case = config.lower_case.unwrap_or(false);
                zs_config.strip_accents = config.strip_accents;
                zs_config.add_prefix_space = config.add_prefix_space;
                zs_config
            }
        }
    }
}

/// Builder for creating local model configurations
#[derive(Debug, Clone, Default)]
pub struct LocalModelConfigBuilder {
    model: ModelType,
    weights: Option<PathBuf>,
    config: Option<PathBuf>,
    vocab: Option<PathBuf>,
    merges: Option<PathBuf>,
    lower_case: Option<bool>,
    strip_accents: Option<bool>,
    add_prefix_space: Option<bool>,
}

impl LocalModelConfigBuilder {
    /// Create a new local model configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the model architecture type
    pub fn model(mut self, model: ModelType) -> Self {
        self.model = model;
        self
    }

    /// Set the path to the model weights file
    pub fn weights(mut self, path: impl Into<PathBuf>) -> Self {
        self.weights = Some(path.into());
        self
    }

    /// Set the path to the model configuration file
    pub fn config(mut self, path: impl Into<PathBuf>) -> Self {
        self.config = Some(path.into());
        self
    }

    /// Set the path to the vocabulary file
    pub fn vocab(mut self, path: impl Into<PathBuf>) -> Self {
        self.vocab = Some(path.into());
        self
    }

    /// Set the path to the merges file (optional, for some tokenizers)
    pub fn merges(mut self, path: impl Into<PathBuf>) -> Self {
        self.merges = Some(path.into());
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

    /// Build the local model configuration
    pub fn build(self) -> LocalModelConfig {
        LocalModelConfig {
            model: self.model,
            weights: self.weights,
            config: self.config,
            vocab: self.vocab,
            merges: self.merges,
            lower_case: self.lower_case,
            strip_accents: self.strip_accents,
            add_prefix_space: self.add_prefix_space,
        }
    }
}
