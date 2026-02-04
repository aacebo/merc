use std::env;

use crate::path::FieldPath;
use crate::value::{Number, Object, Value};

use super::{ConfigError, Provider};

/// Configuration provider that reads from environment variables
///
/// Environment variables are mapped to hierarchical keys:
/// - Single underscores become dots (hierarchy separators)
/// - Double underscores remain as single underscores in the key name
///
/// Example with prefix "APP_":
/// - APP_DATABASE_HOST -> database.host
/// - APP_DATABASE__CONNECTION_STRING -> database.connection_string
pub struct EnvProvider {
    prefix: Option<String>,
}

impl EnvProvider {
    pub fn new(prefix: Option<&str>) -> Self {
        Self {
            prefix: prefix.map(|s| s.to_uppercase()),
        }
    }

    fn parse_key(&self, key: &str) -> Option<String> {
        let key = match &self.prefix {
            Some(prefix) => {
                if key.starts_with(prefix) {
                    &key[prefix.len()..]
                } else {
                    return None;
                }
            }
            None => key,
        };

        if key.is_empty() {
            return None;
        }

        // Replace double underscore with placeholder, then single underscore with dot
        let normalized = key
            .replace("__", "\x00")
            .replace('_', ".")
            .replace('\x00', "_")
            .to_lowercase();

        if normalized.is_empty() {
            None
        } else {
            Some(normalized)
        }
    }

    fn parse_value(s: &str) -> Value {
        // Try to parse as various types
        if s.eq_ignore_ascii_case("true") {
            return Value::Bool(true);
        }

        if s.eq_ignore_ascii_case("false") {
            return Value::Bool(false);
        }

        if s.eq_ignore_ascii_case("null") {
            return Value::Null;
        }

        if let Ok(i) = s.parse::<i64>() {
            return Value::Number(Number::Int(i));
        }

        if let Ok(f) = s.parse::<f64>() {
            return Value::Number(Number::Float(f));
        }

        Value::String(s.to_string())
    }

    fn set_by_path(root: &mut Value, path_str: &str, value: Value) {
        let path = match FieldPath::parse(path_str) {
            Ok(p) => p,
            Err(_) => return,
        };

        use crate::path::FieldSegment;

        let segments = path.segments();
        if segments.is_empty() {
            return;
        }

        if segments.len() == 1 {
            if let (Value::Object(obj), FieldSegment::Key(key)) = (root, &segments[0]) {
                obj.insert(key.clone(), value);
            }
            return;
        }

        let mut current = root;

        for (i, segment) in segments.iter().enumerate() {
            let is_last = i == segments.len() - 1;

            if is_last {
                if let (Value::Object(obj), FieldSegment::Key(key)) = (current, segment) {
                    obj.insert(key.clone(), value);
                }
                return;
            }

            current = match (current, segment) {
                (Value::Object(obj), FieldSegment::Key(key)) => {
                    if !obj.contains_key(key) {
                        obj.insert(key.clone(), Value::Object(Object::new()));
                    }
                    obj.get_mut(key).unwrap()
                }
                _ => return,
            };
        }
    }
}

impl Default for EnvProvider {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Provider for EnvProvider {
    fn name(&self) -> &str {
        "environment"
    }

    fn optional(&self) -> bool {
        true
    }

    fn load(&self) -> Result<Option<Value>, ConfigError> {
        let mut root = Value::Object(Object::new());

        for (key, value) in env::vars() {
            if let Some(path) = self.parse_key(&key) {
                let parsed_value = Self::parse_value(&value);
                Self::set_by_path(&mut root, &path, parsed_value);
            }
        }

        if root.is_empty() {
            Ok(None)
        } else {
            Ok(Some(root))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_with_prefix() {
        let provider = EnvProvider::new(Some("APP_"));

        assert_eq!(
            provider.parse_key("APP_DATABASE_HOST"),
            Some("database.host".to_string())
        );
        assert_eq!(
            provider.parse_key("APP_DATABASE_PORT"),
            Some("database.port".to_string())
        );
        assert_eq!(provider.parse_key("OTHER_KEY"), None);
    }

    #[test]
    fn test_parse_key_double_underscore() {
        let provider = EnvProvider::new(Some("APP_"));

        // Double underscore becomes literal underscore
        // Single underscore becomes dot (hierarchy separator)
        // APP_DB__CONNECTION_STRING -> db_connection.string
        assert_eq!(
            provider.parse_key("APP_DB__CONNECTION_STRING"),
            Some("db_connection.string".to_string())
        );
    }

    #[test]
    fn test_parse_key_no_prefix() {
        let provider = EnvProvider::new(None);

        assert_eq!(
            provider.parse_key("DATABASE_HOST"),
            Some("database.host".to_string())
        );
    }

    #[test]
    fn test_parse_value_bool() {
        assert_eq!(EnvProvider::parse_value("true"), Value::Bool(true));
        assert_eq!(EnvProvider::parse_value("TRUE"), Value::Bool(true));
        assert_eq!(EnvProvider::parse_value("false"), Value::Bool(false));
        assert_eq!(EnvProvider::parse_value("FALSE"), Value::Bool(false));
    }

    #[test]
    fn test_parse_value_int() {
        assert_eq!(
            EnvProvider::parse_value("123"),
            Value::Number(Number::Int(123))
        );
        assert_eq!(
            EnvProvider::parse_value("-456"),
            Value::Number(Number::Int(-456))
        );
    }

    #[test]
    fn test_parse_value_float() {
        assert_eq!(
            EnvProvider::parse_value("3.14"),
            Value::Number(Number::Float(3.14))
        );
    }

    #[test]
    fn test_parse_value_string() {
        assert_eq!(
            EnvProvider::parse_value("hello"),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_set_by_path() {
        let mut root = Value::Object(Object::new());
        EnvProvider::set_by_path(
            &mut root,
            "database.host",
            Value::String("localhost".to_string()),
        );

        let path = FieldPath::parse("database.host").unwrap();
        assert_eq!(root.get_by_path(&path).unwrap().as_str(), Some("localhost"));
    }
}
