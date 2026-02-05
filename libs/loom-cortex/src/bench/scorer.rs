use super::Decision;

/// A scorer that can evaluate text and produce a scoring output.
pub trait Scorer {
    type Output: ScorerOutput;
    type Error;

    /// Score the given text.
    fn score(&self, text: &str) -> Result<Self::Output, Self::Error>;
}

/// Output from a scorer containing decision, score, and label information.
pub trait ScorerOutput {
    /// The decision (Accept/Reject) for this scoring.
    fn decision(&self) -> Decision;

    /// The overall score value.
    fn score(&self) -> f32;

    /// Labels with their raw (uncalibrated) scores.
    /// Returns tuples of (label_name, raw_score).
    fn labels(&self) -> Vec<(String, f32)>;

    /// Labels that were detected (score > 0).
    fn detected_labels(&self) -> Vec<String> {
        self.labels()
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .map(|(name, _)| name)
            .collect()
    }
}
