use loom_error::Result;
use rust_bert::pipelines::zero_shot_classification;
use serde_valid::Validate;

use crate::score::{ScoreConfig, ScoreLayer};

#[derive(Default)]
pub struct ScoreOptions {
    model: zero_shot_classification::ZeroShotClassificationConfig,
    config: Option<ScoreConfig>,
}

impl ScoreOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_model(
        mut self,
        config: zero_shot_classification::ZeroShotClassificationConfig,
    ) -> Self {
        self.model = config;
        self
    }

    pub fn with_config(mut self, config: ScoreConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<ScoreLayer> {
        let config = self.config.unwrap_or_default();

        config
            .validate()
            .map_err(|e| loom_error::Error::builder().message(&e.to_string()).build())?;

        let model = zero_shot_classification::ZeroShotClassificationModel::new(self.model)?;
        Ok(ScoreLayer::new(model, config))
    }
}
