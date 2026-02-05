use rust_bert::pipelines::{pos_tagging, token_classification};
use serde::{Deserialize, Serialize};

use crate::{CortexDevice, CortexModelSource, CortexModelType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexPosTaggingConfig {
    pub model: CortexModelType,

    #[serde(default)]
    pub source: CortexModelSource,

    #[serde(default)]
    pub device: CortexDevice,

    #[serde(default)]
    pub lower_case: bool,

    #[serde(default)]
    pub strip_accents: Option<bool>,

    #[serde(default)]
    pub add_prefix_space: Option<bool>,
}

impl CortexPosTaggingConfig {
    pub fn new(model: CortexModelType) -> CortexPosTaggingConfigBuilder {
        CortexPosTaggingConfigBuilder::new(model)
    }
}

impl Default for CortexPosTaggingConfig {
    fn default() -> Self {
        Self {
            model: CortexModelType::Bert,
            source: CortexModelSource::Default,
            device: CortexDevice::default(),
            lower_case: false,
            strip_accents: None,
            add_prefix_space: None,
        }
    }
}

pub struct CortexPosTaggingConfigBuilder {
    model: CortexModelType,
    source: CortexModelSource,
    device: CortexDevice,
    lower_case: bool,
    strip_accents: Option<bool>,
    add_prefix_space: Option<bool>,
}

impl CortexPosTaggingConfigBuilder {
    pub fn new(model: CortexModelType) -> Self {
        Self {
            model,
            source: CortexModelSource::default(),
            device: CortexDevice::default(),
            lower_case: false,
            strip_accents: None,
            add_prefix_space: None,
        }
    }

    pub fn source(mut self, source: CortexModelSource) -> Self {
        self.source = source;
        self
    }

    pub fn device(mut self, device: CortexDevice) -> Self {
        self.device = device;
        self
    }

    pub fn lower_case(mut self, lower_case: bool) -> Self {
        self.lower_case = lower_case;
        self
    }

    pub fn strip_accents(mut self, strip_accents: Option<bool>) -> Self {
        self.strip_accents = strip_accents;
        self
    }

    pub fn add_prefix_space(mut self, add_prefix_space: Option<bool>) -> Self {
        self.add_prefix_space = add_prefix_space;
        self
    }

    pub fn build(self) -> CortexPosTaggingConfig {
        CortexPosTaggingConfig {
            model: self.model,
            source: self.source,
            device: self.device,
            lower_case: self.lower_case,
            strip_accents: self.strip_accents,
            add_prefix_space: self.add_prefix_space,
        }
    }
}

impl From<CortexPosTaggingConfig> for pos_tagging::POSConfig {
    fn from(config: CortexPosTaggingConfig) -> Self {
        let token_config = token_classification::TokenClassificationConfig {
            model_type: config.model.into(),
            device: config.device.into(),
            lower_case: config.lower_case,
            strip_accents: config.strip_accents,
            add_prefix_space: config.add_prefix_space,
            ..Default::default()
        };
        pos_tagging::POSConfig::from(token_config)
    }
}
