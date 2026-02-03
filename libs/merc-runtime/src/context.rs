use crate::Map;

#[derive(Debug, Default)]
pub struct Context {
    pub meta: Map,
    pub step: usize,
    pub text: String,
    pub data: Map,
}

impl Context {
    pub fn new(text: &str) -> Self {
        Self {
            meta: Map::default(),
            step: 0,
            text: text.to_string(),
            data: Map::default(),
        }
    }
}
