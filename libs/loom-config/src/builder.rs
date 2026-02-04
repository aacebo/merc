use std::path::PathBuf;

use crate::value::{Object, Value};

use super::providers::{EnvProvider, FileProvider, MemoryProvider, Provider};
use super::{ConfigError, ConfigRoot};

/// Builder for constructing layered configuration
pub struct ConfigBuilder {
    providers: Vec<Box<dyn Provider>>,
    base_path: Option<PathBuf>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            base_path: None,
        }
    }

    /// Set the base path for file-based providers
    pub fn set_base_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// Add a custom provider
    pub fn add_provider<P: Provider + 'static>(mut self, provider: P) -> Self {
        self.providers.push(Box::new(provider));
        self
    }

    /// Add a JSON configuration file
    #[cfg(feature = "json")]
    pub fn add_json_file<P: AsRef<str>>(self, path: P) -> Self {
        self.add_json_file_optional(path, false)
    }

    /// Add an optional JSON configuration file
    #[cfg(feature = "json")]
    pub fn add_json_file_optional<P: AsRef<str>>(mut self, path: P, optional: bool) -> Self {
        let full_path = self.resolve_path(path.as_ref());
        self.providers
            .push(Box::new(FileProvider::json(full_path, optional)));
        self
    }

    /// Add a YAML configuration file
    #[cfg(feature = "yaml")]
    pub fn add_yaml_file<P: AsRef<str>>(self, path: P) -> Self {
        self.add_yaml_file_optional(path, false)
    }

    /// Add an optional YAML configuration file
    #[cfg(feature = "yaml")]
    pub fn add_yaml_file_optional<P: AsRef<str>>(mut self, path: P, optional: bool) -> Self {
        let full_path = self.resolve_path(path.as_ref());
        self.providers
            .push(Box::new(FileProvider::yaml(full_path, optional)));
        self
    }

    /// Add environment variables with a prefix filter
    /// Variables like "APP_DATABASE_HOST" become "database.host" with prefix "APP_"
    pub fn add_env_variables(mut self, prefix: Option<&str>) -> Self {
        self.providers.push(Box::new(EnvProvider::new(prefix)));
        self
    }

    /// Add in-memory configuration (useful for defaults or testing)
    pub fn add_in_memory<I, K, V>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<Value>,
    {
        self.providers
            .push(Box::new(MemoryProvider::from_pairs(items)));
        self
    }

    /// Add in-memory configuration from a Value
    pub fn add_in_memory_value(mut self, value: Value) -> Self {
        self.providers
            .push(Box::new(MemoryProvider::from_value(value)));
        self
    }

    pub fn build(self) -> Result<ConfigRoot, ConfigError> {
        use super::ConfigSource;

        let mut merged = Value::Object(Object::new());
        let mut sources = Vec::new();

        for provider in &self.providers {
            match provider.load() {
                Ok(Some(value)) => {
                    merged.deep_merge(value);
                    sources.push(ConfigSource {
                        name: provider.name().to_string(),
                        path: provider.path(),
                        format: provider.format(),
                    });
                }
                Ok(None) => {
                    if !provider.optional() {
                        return Err(ConfigError::not_found(provider.name()));
                    }
                }
                Err(e) => {
                    if !provider.optional() {
                        return Err(e);
                    }
                }
            }
        }

        Ok(ConfigRoot::with_sources(merged, sources))
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        if let Some(ref base) = self.base_path {
            base.join(path)
        } else {
            PathBuf::from(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::FieldPath;

    #[test]
    fn test_builder_in_memory() {
        let config = ConfigBuilder::new()
            .add_in_memory([
                ("database.host", "localhost"),
                ("database.port", "5432"),
                ("logging.level", "info"),
            ])
            .build()
            .unwrap();

        let path = FieldPath::parse("database.host").unwrap();
        assert_eq!(config.get_str(&path), Some("localhost"));

        let path = FieldPath::parse("logging.level").unwrap();
        assert_eq!(config.get_str(&path), Some("info"));
    }

    #[test]
    fn test_builder_merge_order() {
        // Later providers should override earlier ones
        let config = ConfigBuilder::new()
            .add_in_memory([("database.host", "first")])
            .add_in_memory([("database.host", "second")])
            .build()
            .unwrap();

        let path = FieldPath::parse("database.host").unwrap();
        assert_eq!(config.get_str(&path), Some("second"));
    }

    #[test]
    fn test_builder_deep_merge() {
        let config = ConfigBuilder::new()
            .add_in_memory([("database.host", "localhost"), ("database.port", "5432")])
            .add_in_memory([("database.host", "remotehost"), ("logging.level", "debug")])
            .build()
            .unwrap();

        // database.host should be overridden
        let path = FieldPath::parse("database.host").unwrap();
        assert_eq!(config.get_str(&path), Some("remotehost"));

        // database.port should still exist
        let path = FieldPath::parse("database.port").unwrap();
        assert_eq!(config.get_str(&path), Some("5432"));

        // logging.level should be added
        let path = FieldPath::parse("logging.level").unwrap();
        assert_eq!(config.get_str(&path), Some("debug"));
    }

    #[test]
    fn test_builder_empty() {
        let config = ConfigBuilder::new().build().unwrap();
        assert!(config.as_value().is_empty());
    }

    #[test]
    fn test_builder_base_path() {
        let builder = ConfigBuilder::new().set_base_path("/config");
        let path = builder.resolve_path("app.json");
        assert_eq!(path, PathBuf::from("/config/app.json"));
    }
}
