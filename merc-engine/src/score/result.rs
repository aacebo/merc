pub struct ScoreResult {
    pub score: f32,
    pub categories: Vec<ScoreCategory>,
}

pub struct ScoreCategory {
    pub name: String,
    pub score: f32,
    pub labels: Vec<ScoreLabel>,
}

pub struct ScoreLabel {
    pub name: String,
    pub score: f32,
    pub sentence: usize,
}
