use crate::Map;

#[derive(Debug, Default)]
pub struct Context<Input> {
    pub meta: Map,
    pub step: usize,
    pub text: String,
    pub input: Input,
}

impl<Input> Context<Input> {
    pub fn new(text: &str, input: Input) -> Self {
        Self {
            meta: Map::default(),
            step: 0,
            text: text.to_string(),
            input,
        }
    }
}
