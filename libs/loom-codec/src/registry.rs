use std::collections::HashMap;

use loom_core::Format;

use super::Codec;

pub struct CodecRegistry {
    codecs: HashMap<Format, Box<dyn Codec>>,
}

impl CodecRegistry {
    pub fn new() -> CodecRegistryBuilder {
        CodecRegistryBuilder::new()
    }

    pub fn len(&self) -> usize {
        self.codecs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.codecs.is_empty()
    }

    pub fn exists(&self, format: Format) -> bool {
        self.codecs.contains_key(&format)
    }

    pub fn get(&self, format: Format) -> Option<&dyn Codec> {
        self.codecs.get(&format).map(|c| c.as_ref())
    }
}

#[derive(Default)]
pub struct CodecRegistryBuilder {
    codecs: HashMap<Format, Box<dyn Codec>>,
}

impl CodecRegistryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn codec<T: Codec + 'static>(mut self, codec: T) -> Self {
        self.codecs.insert(codec.format(), Box::new(codec));
        self
    }

    pub fn build(self) -> CodecRegistry {
        CodecRegistry {
            codecs: self.codecs,
        }
    }
}
