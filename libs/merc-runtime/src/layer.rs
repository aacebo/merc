use crate::Map;

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
