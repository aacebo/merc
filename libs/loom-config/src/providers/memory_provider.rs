use crate::path::{FieldPath, FieldSegment};
use crate::value::{Object, Value};

use super::{ConfigError, Provider};

/// In-memory configuration provider
pub struct MemoryProvider {
    data: Value,
}

impl MemoryProvider {
    pub fn new() -> Self {
        Self {
            data: Value::Object(Object::new()),
        }
    }

    pub fn from_value(value: Value) -> Self {
        Self { data: value }
    }

    pub fn from_pairs<I, K, V>(items: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<Value>,
    {
        let mut root = Value::Object(Object::new());

        for (k, v) in items {
            if let Ok(path) = FieldPath::parse(k.as_ref()) {
                Self::set_by_path(&mut root, &path, v.into());
            }
        }

        Self { data: root }
    }

    fn set_by_path(root: &mut Value, path: &FieldPath, value: Value) {
        let segments = path.segments();
        if segments.is_empty() {
            return;
        }

        Self::set_nested(root, segments, value);
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
}

impl Default for MemoryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for MemoryProvider {
    fn name(&self) -> &str {
        "memory"
    }

    fn optional(&self) -> bool {
        true
    }

    fn load(&self) -> Result<Option<Value>, ConfigError> {
        if self.data.is_null() || self.data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.data.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_provider_from_pairs() {
        let provider =
            MemoryProvider::from_pairs([("database.host", "localhost"), ("database.port", "5432")]);

        let value = provider.load().unwrap().unwrap();
        let path = FieldPath::parse("database.host").unwrap();
        assert_eq!(
            value.get_by_path(&path).unwrap().as_str(),
            Some("localhost")
        );
    }

    #[test]
    fn test_memory_provider_from_value() {
        let mut obj = Object::new();
        obj.insert("key".to_string(), Value::String("value".to_string()));

        let provider = MemoryProvider::from_value(Value::Object(obj));
        let value = provider.load().unwrap().unwrap();

        let path = FieldPath::parse("key").unwrap();
        assert_eq!(value.get_by_path(&path).unwrap().as_str(), Some("value"));
    }

    #[test]
    fn test_memory_provider_nested_path() {
        let provider = MemoryProvider::from_pairs([("a.b.c", "deep")]);

        let value = provider.load().unwrap().unwrap();
        let path = FieldPath::parse("a.b.c").unwrap();
        assert_eq!(value.get_by_path(&path).unwrap().as_str(), Some("deep"));
    }

    #[test]
    fn test_memory_provider_empty() {
        let provider = MemoryProvider::new();
        assert!(provider.load().unwrap().is_none());
    }
}
