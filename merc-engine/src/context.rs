use crate::Meta;

#[derive(Debug, Default)]
pub struct Context {
    meta: Meta,
    text: String,
}

impl Context {
    pub fn new(text: &str) -> Self {
        Self {
            meta: Meta::default(),
            text: text.to_string(),
        }
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut Meta {
        &mut self.meta
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}
