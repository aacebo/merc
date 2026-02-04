use std::path::PathBuf;

use serde::de::DeserializeOwned;

use crate::Format;
use crate::path::{FieldPath, FieldSegment, Path};
use crate::value::{Object, Value};

use super::{ConfigError, ConfigSection};

#[derive(Debug, Clone)]
pub struct ConfigSource {
    pub name: String,
    pub path: Option<Path>,
    pub format: Option<Format>,
}

#[derive(Debug, Clone)]
pub struct ConfigRoot {
    data: Value,
    sources: Vec<ConfigSource>,
}

impl ConfigRoot {
    pub(crate) fn new(data: Value) -> Self {
        Self {
            data,
            sources: Vec::new(),
        }
    }

    pub(crate) fn with_sources(data: Value, sources: Vec<ConfigSource>) -> Self {
        Self { data, sources }
    }

    pub fn as_value(&self) -> &Value {
        &self.data
    }

    pub fn as_value_mut(&mut self) -> &mut Value {
        &mut self.data
    }

    pub fn sources(&self) -> &[ConfigSource] {
        &self.sources
    }

    pub fn get(&self, path: &FieldPath) -> Option<&Value> {
        self.data.get_by_path(path)
    }

    pub fn get_mut(&mut self, path: &FieldPath) -> Option<&mut Value> {
        self.data.get_by_path_mut(path)
    }

    pub fn get_str(&self, path: &FieldPath) -> Option<&str> {
        self.get(path).and_then(|v| v.as_str())
    }

    pub fn get_int(&self, path: &FieldPath) -> Option<i64> {
        self.get(path).and_then(|v| v.as_int())
    }

    pub fn get_float(&self, path: &FieldPath) -> Option<f64> {
        self.get(path).and_then(|v| v.as_float())
    }

    pub fn get_bool(&self, path: &FieldPath) -> Option<bool> {
        self.get(path).and_then(|v| v.as_bool())
    }

    pub fn get_section(&self, path: &FieldPath) -> ConfigSection<'_> {
        ConfigSection::new(self.get(path), path.clone())
    }

    pub fn root_section(&self) -> ConfigSection<'_> {
        ConfigSection::root(&self.data)
    }

    pub fn set(&mut self, path: &FieldPath, value: Value) {
        let segments = path.segments();
        if segments.is_empty() {
            return;
        }
        Self::set_nested(&mut self.data, segments, value);
    }

    fn set_nested(current: &mut Value, segments: &[FieldSegment], value: Value) {
        if segments.is_empty() {
            return;
        }

        let segment = &segments[0];
        let is_last = segments.len() == 1;

        if let FieldSegment::Key(key) = segment {
            if let Value::Object(obj) = current {
                if is_last {
                    obj.insert(key.clone(), value);
                } else {
                    if !obj.contains_key(key) {
                        obj.insert(key.clone(), Value::Object(Object::new()));
                    }
                    if let Some(child) = obj.get_mut(key) {
                        Self::set_nested(child, &segments[1..], value);
                    }
                }
            }
        }
    }

    #[cfg(feature = "json")]
    pub fn write_json(&self, path: impl Into<PathBuf>) -> Result<(), ConfigError> {
        let json: serde_json::Value = (&self.data).into();
        let content = serde_json::to_string_pretty(&json).map_err(ConfigError::parse)?;
        std::fs::write(path.into(), content)?;
        Ok(())
    }

    #[cfg(feature = "yaml")]
    pub fn write_yaml(&self, path: impl Into<PathBuf>) -> Result<(), ConfigError> {
        let yaml: saphyr::Yaml = (&self.data).into();
        let mut out = String::new();
        let mut emitter = saphyr::YamlEmitter::new(&mut out);
        emitter.dump(&yaml).map_err(|e| ConfigError::parse(e))?;
        std::fs::write(path.into(), out)?;
        Ok(())
    }

    pub fn bind<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        let json: serde_json::Value = (&self.data).into();
        serde_json::from_value(json).map_err(ConfigError::deserialize)
    }

    pub fn bind_section<T: DeserializeOwned>(&self, path: &FieldPath) -> Result<T, ConfigError> {
        let value = self
            .get(path)
            .ok_or_else(|| ConfigError::not_found(path.to_string()))?;
        let json: serde_json::Value = value.into();
        serde_json::from_value(json).map_err(ConfigError::deserialize)
    }
}

impl Default for ConfigRoot {
    fn default() -> Self {
        Self::new(Value::Object(Object::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ConfigRoot {
        use crate::value::Number;

        let mut db = Object::new();
        db.insert("host".to_string(), Value::String("localhost".to_string()));
        db.insert("port".to_string(), Value::Number(Number::Int(5432)));

        let servers = vec![
            {
                let mut s = Object::new();
                s.insert("name".to_string(), Value::String("primary".to_string()));
                s.insert("port".to_string(), Value::Number(Number::Int(8080)));
                Value::Object(s)
            },
            {
                let mut s = Object::new();
                s.insert("name".to_string(), Value::String("secondary".to_string()));
                s.insert("port".to_string(), Value::Number(Number::Int(8081)));
                Value::Object(s)
            },
        ];

        let mut root = Object::new();
        root.insert("database".to_string(), Value::Object(db));
        root.insert("servers".to_string(), Value::Array(servers.into()));
        root.insert("debug".to_string(), Value::Bool(true));

        ConfigRoot::new(Value::Object(root))
    }

    #[test]
    fn test_get_str() {
        let config = create_test_config();
        let path = FieldPath::parse("database.host").unwrap();
        assert_eq!(config.get_str(&path), Some("localhost"));
    }

    #[test]
    fn test_get_int() {
        let config = create_test_config();
        let path = FieldPath::parse("database.port").unwrap();
        assert_eq!(config.get_int(&path), Some(5432));
    }

    #[test]
    fn test_get_bool() {
        let config = create_test_config();
        let path = FieldPath::parse("debug").unwrap();
        assert_eq!(config.get_bool(&path), Some(true));
    }

    #[test]
    fn test_get_array_element() {
        let config = create_test_config();
        let path = FieldPath::parse("servers[0].name").unwrap();
        assert_eq!(config.get_str(&path), Some("primary"));

        let path = FieldPath::parse("servers[1].port").unwrap();
        assert_eq!(config.get_int(&path), Some(8081));
    }

    #[test]
    fn test_get_section() {
        let config = create_test_config();
        let path = FieldPath::parse("database").unwrap();
        let section = config.get_section(&path);

        assert!(section.exists());
        assert!(section.is_object());
    }

    #[test]
    fn test_get_nonexistent() {
        let config = create_test_config();
        let path = FieldPath::parse("nonexistent.path").unwrap();
        assert!(config.get(&path).is_none());
    }

    #[test]
    fn test_set() {
        let mut config = ConfigRoot::default();
        let path = FieldPath::parse("database.host").unwrap();
        config.set(&path, Value::String("localhost".to_string()));

        assert_eq!(config.get_str(&path), Some("localhost"));
    }

    #[test]
    fn test_set_nested() {
        let mut config = ConfigRoot::default();
        let path = FieldPath::parse("a.b.c").unwrap();
        config.set(&path, Value::String("deep".to_string()));

        assert_eq!(config.get_str(&path), Some("deep"));
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_bind_section() {
        use serde::Deserialize;

        #[derive(Debug, Deserialize, PartialEq)]
        struct DatabaseConfig {
            host: String,
            port: i64,
        }

        let config = create_test_config();
        let path = FieldPath::parse("database").unwrap();
        let db: DatabaseConfig = config.bind_section(&path).unwrap();

        assert_eq!(
            db,
            DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
            }
        );
    }
}
