use std::collections::HashMap;

use super::DataSource;

pub struct DataSourceRegistry {
    sources: HashMap<String, Box<dyn DataSource>>,
}

impl DataSourceRegistry {
    pub fn new() -> DataSourceRegistryBuilder {
        DataSourceRegistryBuilder::new()
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    pub fn exists(&self, name: &str) -> bool {
        self.sources.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&dyn DataSource> {
        self.sources.get(name).map(|c| c.as_ref())
    }
}

#[derive(Default)]
pub struct DataSourceRegistryBuilder {
    sources: HashMap<String, Box<dyn DataSource>>,
}

impl DataSourceRegistryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn source<T: DataSource + 'static>(mut self, source: T) -> Self {
        self.sources
            .insert(source.name().to_string(), Box::new(source));
        self
    }

    pub fn build(self) -> DataSourceRegistry {
        DataSourceRegistry {
            sources: self.sources,
        }
    }
}
