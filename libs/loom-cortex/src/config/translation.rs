use serde::{Deserialize, Serialize};

use crate::{CortexDevice, CortexModelSource, CortexModelType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexTranslationConfig {
    pub model: CortexModelType,

    #[serde(default)]
    pub source: CortexModelSource,

    #[serde(default)]
    pub device: CortexDevice,

    #[serde(default)]
    pub source_languages: Vec<String>,

    #[serde(default)]
    pub target_languages: Vec<String>,
}

impl CortexTranslationConfig {
    pub fn new(model: CortexModelType) -> CortexTranslationConfigBuilder {
        CortexTranslationConfigBuilder::new(model)
    }
}

impl Default for CortexTranslationConfig {
    fn default() -> Self {
        Self {
            model: CortexModelType::Marian,
            source: CortexModelSource::Default,
            device: CortexDevice::default(),
            source_languages: Vec::new(),
            target_languages: Vec::new(),
        }
    }
}

pub struct CortexTranslationConfigBuilder {
    model: CortexModelType,
    source: CortexModelSource,
    device: CortexDevice,
    source_languages: Vec<String>,
    target_languages: Vec<String>,
}

impl CortexTranslationConfigBuilder {
    pub fn new(model: CortexModelType) -> Self {
        Self {
            model,
            source: CortexModelSource::default(),
            device: CortexDevice::default(),
            source_languages: Vec::new(),
            target_languages: Vec::new(),
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

    pub fn source_languages(mut self, source_languages: Vec<String>) -> Self {
        self.source_languages = source_languages;
        self
    }

    pub fn target_languages(mut self, target_languages: Vec<String>) -> Self {
        self.target_languages = target_languages;
        self
    }

    pub fn build(self) -> CortexTranslationConfig {
        CortexTranslationConfig {
            model: self.model,
            source: self.source,
            device: self.device,
            source_languages: self.source_languages,
            target_languages: self.target_languages,
        }
    }
}
