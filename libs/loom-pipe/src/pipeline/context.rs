use loom_core::Map;

/// Trait for layer input contexts
pub trait LayerContext: Send + 'static {
    /// Get the text being processed
    fn text(&self) -> &str;

    /// Get the current step in the pipeline
    fn step(&self) -> usize;

    /// Get metadata
    fn meta(&self) -> &Map;
}

/// Result wrapper for layer outputs
pub struct LayerResult<T> {
    pub meta: Map,
    pub output: T,
}

impl<T> LayerResult<T> {
    pub fn new(output: T) -> Self {
        Self {
            meta: Map::default(),
            output,
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for LayerResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", &self.output)
    }
}
