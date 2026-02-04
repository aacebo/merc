mod array;
mod number;
mod object;

pub use array::*;
pub use number::*;
pub use object::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Array),
    Object(Object),
}

impl Value {
    pub fn kind(&self) -> &str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
        }
    }

    // Type checking methods

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Number(Number::Int(_)))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Number(Number::Float(_)))
    }

    // Value extraction methods

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Number(Number::Int(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Number(Number::Float(v)) => Some(*v),
            Self::Number(Number::Int(v)) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Array> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Self::Object(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match self {
            Self::Object(v) => Some(v),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::String(v) => v.len(),
            Self::Array(v) => v.len(),
            Self::Object(v) => v.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::String(v) => v.is_empty(),
            Self::Array(v) => v.is_empty(),
            Self::Object(v) => v.is_empty(),
            _ => true,
        }
    }

    /// Deep merge another Value into self
    pub fn deep_merge(&mut self, source: Value) {
        match (self, source) {
            (Value::Object(target), Value::Object(source)) => {
                for (key, source_value) in source.iter() {
                    match target.get_mut(key) {
                        Some(target_value) => target_value.deep_merge(source_value.clone()),
                        None => {
                            target.insert(key.clone(), source_value.clone());
                        }
                    }
                }
            }
            (target, source) => *target = source,
        }
    }

    /// Get value by FieldPath, traversing objects and arrays
    pub fn get_by_path(&self, path: &crate::path::FieldPath) -> Option<&Value> {
        use crate::path::FieldSegment;

        let mut current = self;

        for segment in path.segments() {
            current = match (current, segment) {
                (Value::Object(obj), FieldSegment::Key(key)) => obj.get(key)?,
                (Value::Array(arr), FieldSegment::Index(idx)) => arr.get(*idx)?,
                _ => return None,
            };
        }

        Some(current)
    }

    /// Get mutable value by FieldPath
    pub fn get_by_path_mut(&mut self, path: &crate::path::FieldPath) -> Option<&mut Value> {
        use crate::path::FieldSegment;

        let mut current = self;

        for segment in path.segments() {
            current = match (current, segment) {
                (Value::Object(obj), FieldSegment::Key(key)) => obj.get_mut(key)?,
                (Value::Array(arr), FieldSegment::Index(idx)) => arr.get_mut(*idx)?,
                _ => return None,
            };
        }

        Some(current)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Array(v) => write!(f, "{}", v),
            Self::Object(v) => write!(f, "{}", v),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Number(Number::Int(value))
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<isize> for Value {
    fn from(value: isize) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Number(Number::Int(value as i64))
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Number(Number::Float(value as f64))
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(Number::Float(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Array> for Value {
    fn from(value: Array) -> Self {
        Self::Array(value)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(value: Vec<T>) -> Self {
        Self::Array(Array::from(value))
    }
}

impl<T: Into<Value>, const N: usize> From<[T; N]> for Value {
    fn from(value: [T; N]) -> Self {
        Self::Array(Array::from(value))
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl std::ops::Index<&str> for Value {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        static NULL: Value = Value::Null;
        match self {
            Self::Object(obj) => obj.get(key).unwrap_or(&NULL),
            _ => &NULL,
        }
    }
}

impl std::ops::Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        static NULL: Value = Value::Null;
        match self {
            Self::Array(arr) => arr.get(index).unwrap_or(&NULL),
            _ => &NULL,
        }
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Value> for Value {
    fn from(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::Number(Number::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Self::Number(Number::Float(f))
                } else {
                    Self::Null
                }
            }
            serde_json::Value::String(s) => Self::String(s),
            serde_json::Value::Array(arr) => Self::Array(Array::from(
                arr.into_iter().map(Self::from).collect::<Vec<_>>(),
            )),
            serde_json::Value::Object(obj) => {
                let mut map = Object::new();

                for (k, v) in obj {
                    map.insert(k, Self::from(v));
                }

                Self::Object(map)
            }
        }
    }
}

#[cfg(feature = "json")]
impl From<&Value> for serde_json::Value {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Bool(*b),
            Value::Number(Number::Int(i)) => Self::Number((*i).into()),
            Value::Number(Number::Float(f)) => serde_json::Number::from_f64(*f)
                .map(Self::Number)
                .unwrap_or(Self::Null),
            Value::String(s) => Self::String(s.clone()),
            Value::Array(arr) => Self::Array(arr.iter().map(Self::from).collect()),
            Value::Object(obj) => {
                let map: serde_json::Map<String, Self> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::from(v)))
                    .collect();
                Self::Object(map)
            }
        }
    }
}

#[cfg(feature = "json")]
impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Bool(b),
            Value::Number(Number::Int(i)) => Self::Number(i.into()),
            Value::Number(Number::Float(f)) => serde_json::Number::from_f64(f)
                .map(Self::Number)
                .unwrap_or(Self::Null),
            Value::String(s) => Self::String(s),
            Value::Array(arr) => Self::Array(arr.into_iter().map(Self::from).collect()),
            Value::Object(obj) => {
                let map: serde_json::Map<String, Self> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::from(v)))
                    .collect();
                Self::Object(map)
            }
        }
    }
}

#[cfg(feature = "yaml")]
impl From<saphyr::Yaml> for Value {
    fn from(yaml: saphyr::Yaml) -> Self {
        match yaml {
            saphyr::Yaml::Null => Self::Null,
            saphyr::Yaml::Boolean(b) => Self::Bool(b),
            saphyr::Yaml::Integer(i) => Self::Number(Number::Int(i)),
            saphyr::Yaml::Real(s) => {
                if let Ok(f) = s.parse::<f64>() {
                    Self::Number(Number::Float(f))
                } else {
                    Self::String(s)
                }
            }
            saphyr::Yaml::String(s) => Self::String(s),
            saphyr::Yaml::Array(arr) => Self::Array(Array::from(
                arr.into_iter().map(Self::from).collect::<Vec<_>>(),
            )),
            saphyr::Yaml::Hash(hash) => {
                let mut map = Object::new();
                for (k, v) in hash {
                    let key = match k {
                        saphyr::Yaml::String(s) => s,
                        saphyr::Yaml::Integer(i) => i.to_string(),
                        saphyr::Yaml::Real(s) => s,
                        saphyr::Yaml::Boolean(b) => b.to_string(),
                        _ => continue,
                    };
                    map.insert(key, Self::from(v));
                }
                Self::Object(map)
            }
            saphyr::Yaml::Alias(_) => Self::Null,
            saphyr::Yaml::BadValue => Self::Null,
        }
    }
}

#[cfg(feature = "yaml")]
impl From<&Value> for saphyr::Yaml {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Boolean(*b),
            Value::Number(Number::Int(i)) => Self::Integer(*i),
            Value::Number(Number::Float(f)) => Self::Real(f.to_string()),
            Value::String(s) => Self::String(s.clone()),
            Value::Array(arr) => Self::Array(arr.iter().map(Self::from).collect()),
            Value::Object(obj) => {
                let hash: saphyr::Hash = obj
                    .iter()
                    .map(|(k, v)| (Self::String(k.clone()), Self::from(v)))
                    .collect();
                Self::Hash(hash)
            }
        }
    }
}

#[cfg(feature = "yaml")]
impl From<Value> for saphyr::Yaml {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Boolean(b),
            Value::Number(Number::Int(i)) => Self::Integer(i),
            Value::Number(Number::Float(f)) => Self::Real(f.to_string()),
            Value::String(s) => Self::String(s),
            Value::Array(arr) => Self::Array(arr.into_iter().map(Self::from).collect()),
            Value::Object(obj) => {
                let hash: saphyr::Hash = obj
                    .iter()
                    .map(|(k, v)| (Self::String(k.clone()), Self::from(v)))
                    .collect();
                Self::Hash(hash)
            }
        }
    }
}

#[cfg(feature = "toml")]
impl From<toml::Value> for Value {
    fn from(toml_val: toml::Value) -> Self {
        match toml_val {
            toml::Value::Boolean(b) => Self::Bool(b),
            toml::Value::Integer(i) => Self::Number(Number::Int(i)),
            toml::Value::Float(f) => Self::Number(Number::Float(f)),
            toml::Value::String(s) => Self::String(s),
            toml::Value::Array(arr) => Self::Array(Array::from(
                arr.into_iter().map(Self::from).collect::<Vec<_>>(),
            )),
            toml::Value::Table(table) => {
                let mut map = Object::new();
                for (k, v) in table {
                    map.insert(k, Self::from(v));
                }
                Self::Object(map)
            }
            toml::Value::Datetime(dt) => Self::String(dt.to_string()),
        }
    }
}

#[cfg(feature = "toml")]
impl From<&Value> for toml::Value {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Self::String(String::new()),
            Value::Bool(b) => Self::Boolean(*b),
            Value::Number(Number::Int(i)) => Self::Integer(*i),
            Value::Number(Number::Float(f)) => Self::Float(*f),
            Value::String(s) => Self::String(s.clone()),
            Value::Array(arr) => Self::Array(arr.iter().map(Self::from).collect()),
            Value::Object(obj) => {
                let table: toml::Table = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::from(v)))
                    .collect();
                Self::Table(table)
            }
        }
    }
}

#[cfg(feature = "toml")]
impl From<Value> for toml::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Self::String(String::new()),
            Value::Bool(b) => Self::Boolean(b),
            Value::Number(Number::Int(i)) => Self::Integer(i),
            Value::Number(Number::Float(f)) => Self::Float(f),
            Value::String(s) => Self::String(s),
            Value::Array(arr) => Self::Array(arr.into_iter().map(Self::from).collect()),
            Value::Object(obj) => {
                let table: toml::Table = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::from(v)))
                    .collect();
                Self::Table(table)
            }
        }
    }
}
