use crate::score::ScoreOptions;

#[derive(Default)]
pub struct EngineOptions {
    score: ScoreOptions,
}

impl EngineOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_score(mut self, options: ScoreOptions) -> Self {
        self.score = options;
        self
    }
}
