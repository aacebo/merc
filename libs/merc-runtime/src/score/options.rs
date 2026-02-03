use merc_error::Result;
use rust_bert::pipelines::zero_shot_classification;

use crate::score::ScoreLayer;

#[derive(Default)]
pub struct ScoreOptions {
    model: zero_shot_classification::ZeroShotClassificationConfig,
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

    pub fn build(self) -> Result<ScoreLayer> {
        let model = zero_shot_classification::ZeroShotClassificationModel::new(self.model)?;
        Ok(ScoreLayer { model })
    }
}
