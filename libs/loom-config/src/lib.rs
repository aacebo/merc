mod builder;
mod error;
pub mod providers;
mod root;
mod section;

pub use builder::*;
pub use error::*;
pub use providers::{EnvProvider, FileProvider, MemoryProvider, Provider};
pub use root::*;
pub use section::*;

// Re-export loom-core types for convenience
pub use loom_core::{Format, Map, MediaType, path, value};

/// Get a value from a ConfigRoot by path string.
///
/// # Examples
///
/// ```ignore
/// use loom_runtime::{get, config::ConfigRoot};
///
/// let config: ConfigRoot = /* ... */;
///
/// // Get a string value (default)
/// let host: Option<&str> = crate::get!(config, "database.host");
///
/// // Get an integer value
/// let port: Option<i64> = crate::get!(config, "database.port", int);
///
/// // Get a float value
/// let rate: Option<f64> = crate::get!(config, "limits.rate", float);
///
/// // Get a boolean value
/// let debug: Option<bool> = crate::get!(config, "debug", bool);
///
/// // Get a raw Value reference
/// let value: Option<&Value> = crate::get!(config, "servers[0]", value);
/// ```
#[macro_export]
macro_rules! get {
    ($config:expr, $path:expr) => {{
        $crate::path::FieldPath::parse($path)
            .ok()
            .and_then(|p| $config.get_str(&p))
    }};
    ($config:expr, $path:expr, int) => {{
        $crate::path::FieldPath::parse($path)
            .ok()
            .and_then(|p| $config.get_int(&p))
    }};
    ($config:expr, $path:expr, float) => {{
        $crate::path::FieldPath::parse($path)
            .ok()
            .and_then(|p| $config.get_float(&p))
    }};
    ($config:expr, $path:expr, bool) => {{
        $crate::path::FieldPath::parse($path)
            .ok()
            .and_then(|p| $config.get_bool(&p))
    }};
    ($config:expr, $path:expr, value) => {{
        $crate::path::FieldPath::parse($path)
            .ok()
            .and_then(|p| $config.get(&p))
    }};
}

/// Set a value in a ConfigRoot by path string.
///
/// # Examples
///
/// ```ignore
/// use loom_runtime::{set, config::ConfigRoot, value::Value};
///
/// let mut config = ConfigRoot::default();
///
/// // Set a string value
/// crate::set!(config, "database.host", "localhost");
///
/// // Set an integer value
/// crate::set!(config, "database.port", 5432);
///
/// // Set a boolean value
/// crate::set!(config, "debug", true);
///
/// // Set a Value directly
/// crate::set!(config, "data", Value::Null);
/// ```
#[macro_export]
macro_rules! set {
    ($config:expr, $path:expr, $value:expr) => {{
        if let Ok(p) = $crate::path::FieldPath::parse($path) {
            $config.set(&p, $value.into());
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_str() {
        let config = ConfigBuilder::new()
            .add_in_memory([("database.host", "localhost")])
            .build()
            .unwrap();

        assert_eq!(crate::get!(config, "database.host"), Some("localhost"));
    }

    #[test]
    fn test_get_int() {
        let mut config = ConfigRoot::default();
        let path = path::FieldPath::parse("database.port").unwrap();
        config.set(&path, value::Value::from(5432i64));

        assert_eq!(crate::get!(config, "database.port", int), Some(5432));
    }

    #[test]
    fn test_get_bool() {
        let mut config = ConfigRoot::default();
        let path = path::FieldPath::parse("debug").unwrap();
        config.set(&path, value::Value::from(true));

        assert_eq!(crate::get!(config, "debug", bool), Some(true));
    }

    #[test]
    fn test_get_float() {
        let mut config = ConfigRoot::default();
        let path = path::FieldPath::parse("rate").unwrap();
        config.set(&path, value::Value::from(3.14f64));

        assert_eq!(crate::get!(config, "rate", float), Some(3.14));
    }

    #[test]
    fn test_get_value() {
        let mut config = ConfigRoot::default();
        let path = path::FieldPath::parse("data").unwrap();
        config.set(&path, value::Value::Null);

        assert!(crate::get!(config, "data", value).is_some());
    }

    #[test]
    fn test_get_nonexistent() {
        let config = ConfigRoot::default();
        assert_eq!(crate::get!(config, "nonexistent"), None);
    }

    #[test]
    fn test_set_str() {
        let mut config = ConfigRoot::default();
        crate::set!(config, "database.host", "localhost");

        assert_eq!(crate::get!(config, "database.host"), Some("localhost"));
    }

    #[test]
    fn test_set_int() {
        let mut config = ConfigRoot::default();
        crate::set!(config, "database.port", 5432i64);

        assert_eq!(crate::get!(config, "database.port", int), Some(5432));
    }

    #[test]
    fn test_set_bool() {
        let mut config = ConfigRoot::default();
        crate::set!(config, "debug", true);

        assert_eq!(crate::get!(config, "debug", bool), Some(true));
    }

    #[test]
    fn test_set_nested() {
        let mut config = ConfigRoot::default();
        crate::set!(config, "a.b.c", "deep");

        assert_eq!(crate::get!(config, "a.b.c"), Some("deep"));
    }
}
