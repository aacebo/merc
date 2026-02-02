use merc_error::Result;
use rust_bert::pipelines::zero_shot_classification;

use crate::score::ScoreLayer;

#[derive(Default)]
pub struct ScoreOptions {
    threshold: f32,
    dynamic_threshold: bool,
    model: zero_shot_classification::ZeroShotClassificationConfig,
}

impl ScoreOptions {
    pub fn new() -> Self {
        Self {
            threshold: 0.75,
            ..Default::default()
        }
    }

    pub fn with_threshold(mut self, value: f32) -> Self {
        self.threshold = value;
        self
    }

    pub fn with_dynamic_threshold(mut self, value: bool) -> Self {
        self.dynamic_threshold = value;
        self
    }

    pub fn with_model(
        mut self,
        config: zero_shot_classification::ZeroShotClassificationConfig,
    ) -> Self {
        self.model = config;
        self
    }

    pub fn build(self) -> Result<ScoreLayer> {
        let model = zero_shot_classification::ZeroShotClassificationModel::new(self.model)?;

        Ok(ScoreLayer {
            threshold: self.threshold,
            dynamic_threshold: self.dynamic_threshold,
            model,
        })
    }
}
