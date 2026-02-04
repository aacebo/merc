use std::collections::BTreeMap;

use super::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Object(BTreeMap<String, Value>);

impl Object {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl std::ops::Deref for Object {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for (i, (key, entity)) in self.0.iter().enumerate() {
            write!(f, "{}: {}", key, entity)?;

            if i < self.0.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "}}")
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}
