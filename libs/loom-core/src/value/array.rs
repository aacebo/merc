use super::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Array(Vec<Value>);

impl Array {
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl std::ops::Deref for Array {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        for (i, value) in self.0.iter().enumerate() {
            write!(f, "{}", value)?;

            if i < self.0.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}

impl<T: Into<Value>> From<Vec<T>> for Array {
    fn from(value: Vec<T>) -> Self {
        Self(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: Into<Value>, const N: usize> From<[T; N]> for Array {
    fn from(value: [T; N]) -> Self {
        Self(value.into_iter().map(|v| v.into()).collect())
    }
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}
