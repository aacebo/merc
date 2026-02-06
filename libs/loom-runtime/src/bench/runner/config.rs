/// Configuration for async benchmark execution.
#[derive(Debug, Clone)]
pub struct AsyncRunConfig {
    /// Maximum number of concurrent inference tasks.
    /// Defaults to 4 for CPU-bound ML inference.
    pub concurrency: usize,

    /// Batch size for batch inference.
    /// If None, uses the scorer's default batch size.
    /// If Some(n), processes n samples per batch.
    pub batch_size: Option<usize>,
}

impl Default for AsyncRunConfig {
    fn default() -> Self {
        Self {
            concurrency: 4,
            batch_size: None,
        }
    }
}
