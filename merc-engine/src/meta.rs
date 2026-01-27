use std::{collections::BTreeMap, sync::Arc};

pub trait MetaValue: std::any::Any + std::fmt::Debug {}

impl<T: std::any::Any + std::fmt::Debug> MetaValue for T {}

#[derive(Clone, Default)]
pub struct Meta {
    inner: BTreeMap<String, Arc<dyn MetaValue>>,
}

impl Meta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn set<Value: MetaValue>(&mut self, key: &str, value: Value) -> &mut Self {
        self.inner.insert(key.to_string(), Arc::new(value));
        self
    }

    pub fn merge(&mut self, other: Self) -> &mut Self {
        for (key, value) in other.inner {
            self.inner.insert(key, value);
        }

        self
    }
}

impl std::ops::Deref for Meta {
    type Target = BTreeMap<String, Arc<dyn MetaValue>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Debug for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Meta");

        for (key, value) in &self.inner {
            s.field(key.as_str(), value);
        }

        s.finish()
    }
}
