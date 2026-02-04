pub mod bench;
pub mod codec;
mod context;
mod data_source;
mod format;
mod layer;
mod map;
mod media_type;
mod options;
pub mod path;
pub mod pipe;
pub mod score;
pub mod value;

pub use codec::*;
pub use context::*;
pub use data_source::*;
pub use format::*;
pub use layer::*;
pub use map::*;
pub use media_type::*;
pub use options::*;

pub struct Runtime {
    #[allow(unused)]
    codecs: Vec<Box<dyn Codec>>,

    #[allow(unused)]
    sources: Vec<Box<dyn DataSource>>,
}

#[derive(Default)]
pub struct Builder {
    codecs: Vec<Box<dyn Codec>>,
    sources: Vec<Box<dyn DataSource>>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn codec<T: Codec + 'static>(mut self, codec: T) -> Self {
        self.codecs.push(Box::new(codec));
        self
    }

    pub fn source<T: DataSource + 'static>(mut self, source: T) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    pub fn build(self) -> Runtime {
        Runtime {
            codecs: self.codecs,
            sources: self.sources,
        }
    }
}
