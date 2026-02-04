use serde::de::DeserializeOwned;

use crate::path::{FieldPath, FieldSegment};
use crate::value::Value;

use super::ConfigError;

/// A view into a section of configuration (can be Object or Array)
#[derive(Debug, Clone)]
pub struct ConfigSection<'a> {
    value: Option<&'a Value>,
    path: FieldPath,
}

impl<'a> ConfigSection<'a> {
    pub(crate) fn new(value: Option<&'a Value>, path: FieldPath) -> Self {
        Self { value, path }
    }

    pub(crate) fn root(value: &'a Value) -> Self {
        Self {
            value: Some(value),
            path: FieldPath::parse("root").expect("valid path"),
        }
    }

    /// The path to this section
    pub fn path(&self) -> &FieldPath {
        &self.path
    }

    /// Whether this section exists
    pub fn exists(&self) -> bool {
        self.value.is_some()
    }

    /// Get the underlying value
    pub fn value(&self) -> Option<&'a Value> {
        self.value
    }

    /// Check if this section is an object
    pub fn is_object(&self) -> bool {
        self.value.map(|v| v.is_object()).unwrap_or(false)
    }

    /// Check if this section is an array
    pub fn is_array(&self) -> bool {
        self.value.map(|v| v.is_array()).unwrap_or(false)
    }

    /// Get value at relative path
    pub fn get(&self, path: &FieldPath) -> Option<&'a Value> {
        self.value?.get_by_path(path)
    }

    /// Get child section by key (for objects)
    pub fn get_section(&self, key: &str) -> ConfigSection<'a> {
        let child_value = self.value.and_then(|v| match v {
            Value::Object(obj) => obj.get(key),
            _ => None,
        });

        // Build child path
        let mut segments: Vec<FieldSegment> = self.path.segments().to_vec();
        segments.push(FieldSegment::Key(key.to_string()));

        // Create a simple path string and parse it
        let child_path_str = if self.path.to_string() == "root" {
            key.to_string()
        } else {
            format!("{}.{}", self.path, key)
        };

        let child_path = FieldPath::parse(&child_path_str).unwrap_or(self.path.clone());

        ConfigSection::new(child_value, child_path)
    }

    /// Get child section by index (for arrays)
    pub fn get_index(&self, index: usize) -> ConfigSection<'a> {
        let child_value = self.value.and_then(|v| match v {
            Value::Array(arr) => arr.get(index),
            _ => None,
        });

        let child_path_str = if self.path.to_string() == "root" {
            format!("[{}]", index)
        } else {
            format!("{}[{}]", self.path, index)
        };

        let child_path = FieldPath::parse(&child_path_str).unwrap_or(self.path.clone());

        ConfigSection::new(child_value, child_path)
    }

    /// Bind this section to a strongly-typed object
    pub fn bind<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        let value = self
            .value
            .ok_or_else(|| ConfigError::not_found(self.path.to_string()))?;

        let json: serde_json::Value = value.into();
        serde_json::from_value(json).map_err(ConfigError::deserialize)
    }

    /// Get object keys (if this section is an object)
    pub fn keys(&self) -> Option<impl Iterator<Item = &'a str>> {
        match self.value? {
            Value::Object(obj) => Some(obj.keys().map(|s| s.as_str())),
            _ => None,
        }
    }

    /// Get array length (if this section is an array)
    pub fn len(&self) -> Option<usize> {
        match self.value? {
            Value::Array(arr) => Some(arr.len()),
            Value::Object(obj) => Some(obj.len()),
            _ => None,
        }
    }

    /// Check if the section is empty
    pub fn is_empty(&self) -> Option<bool> {
        self.len().map(|l| l == 0)
    }

    /// Iterate children as sections
    pub fn children(&self) -> Vec<ConfigSection<'a>> {
        match self.value {
            Some(Value::Object(obj)) => obj
                .iter()
                .map(|(k, v)| {
                    let child_path_str = if self.path.to_string() == "root" {
                        k.clone()
                    } else {
                        format!("{}.{}", self.path, k)
                    };
                    let child_path = FieldPath::parse(&child_path_str).unwrap_or(self.path.clone());
                    ConfigSection::new(Some(v), child_path)
                })
                .collect(),
            Some(Value::Array(arr)) => arr
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let child_path_str = if self.path.to_string() == "root" {
                        format!("[{}]", i)
                    } else {
                        format!("{}[{}]", self.path, i)
                    };
                    let child_path = FieldPath::parse(&child_path_str).unwrap_or(self.path.clone());
                    ConfigSection::new(Some(v), child_path)
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Object;

    fn create_test_config() -> Value {
        use crate::value::Number;

        let mut db = Object::new();
        db.insert("host".to_string(), Value::String("localhost".to_string()));
        db.insert("port".to_string(), Value::Number(Number::Int(5432)));

        let servers = vec![
            {
                let mut s = Object::new();
                s.insert("name".to_string(), Value::String("primary".to_string()));
                Value::Object(s)
            },
            {
                let mut s = Object::new();
                s.insert("name".to_string(), Value::String("secondary".to_string()));
                Value::Object(s)
            },
        ];

        let mut root = Object::new();
        root.insert("database".to_string(), Value::Object(db));
        root.insert("servers".to_string(), Value::Array(servers.into()));

        Value::Object(root)
    }

    #[test]
    fn test_section_exists() {
        let config = create_test_config();
        let section = ConfigSection::root(&config);

        assert!(section.exists());
        assert!(section.get_section("database").exists());
        assert!(!section.get_section("nonexistent").exists());
    }

    #[test]
    fn test_section_is_object() {
        let config = create_test_config();
        let section = ConfigSection::root(&config);

        assert!(section.is_object());
        assert!(section.get_section("database").is_object());
        assert!(!section.get_section("servers").is_object());
    }

    #[test]
    fn test_section_is_array() {
        let config = create_test_config();
        let section = ConfigSection::root(&config);

        assert!(!section.is_array());
        assert!(section.get_section("servers").is_array());
    }

    #[test]
    fn test_section_get_index() {
        let config = create_test_config();
        let section = ConfigSection::root(&config);
        let servers = section.get_section("servers");

        let first = servers.get_index(0);
        assert!(first.exists());
        assert!(first.is_object());

        let path = FieldPath::parse("name").unwrap();
        assert_eq!(first.get(&path).unwrap().as_str(), Some("primary"));
    }

    #[test]
    fn test_section_keys() {
        let config = create_test_config();
        let section = ConfigSection::root(&config);
        let db = section.get_section("database");

        let keys: Vec<_> = db.keys().unwrap().collect();
        assert!(keys.contains(&"host"));
        assert!(keys.contains(&"port"));
    }

    #[test]
    fn test_section_len() {
        let config = create_test_config();
        let section = ConfigSection::root(&config);

        assert_eq!(section.get_section("servers").len(), Some(2));
        assert_eq!(section.get_section("database").len(), Some(2));
    }

    #[test]
    fn test_section_children() {
        let config = create_test_config();
        let section = ConfigSection::root(&config);
        let servers = section.get_section("servers");

        let children = servers.children();
        assert_eq!(children.len(), 2);
    }
}
