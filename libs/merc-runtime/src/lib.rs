pub mod bench;
mod context;
mod data_source;
mod document;
mod layer;
mod map;
mod media_type;
mod options;
pub mod pipe;
pub mod score;

pub use context::*;
pub use data_source::*;
pub use document::*;
pub use layer::*;
pub use map::*;
pub use media_type::*;
pub use options::*;

pub struct Runtime {
    #[allow(unused)]
    data_sources: Vec<Box<dyn DataSource>>,
}

#[derive(Default)]
pub struct Builder {
    data_sources: Vec<Box<dyn DataSource>>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data_source<T: DataSource + 'static>(mut self, source: T) -> Self {
        self.data_sources.push(Box::new(source));
        self
    }

    pub fn build(self) -> Runtime {
        Runtime {
            data_sources: self.data_sources,
        }
    }
}
