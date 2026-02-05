/// Results for a specific label.
#[derive(Debug, Clone, Default)]
pub struct LabelResult {
    pub expected_count: usize,
    pub detected_count: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub precision: f32,
    pub recall: f32,
    pub f1: f32,
}
