/// Results for a specific category.
#[derive(Debug, Clone, Default)]
pub struct CategoryResult {
    pub total: usize,
    pub correct: usize,
    pub accuracy: f32,
}
