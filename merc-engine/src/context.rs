use std::collections::HashMap;

pub struct Context {
    meta: Meta,
    text: String,
}

impl Context {
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Default)]
pub struct Meta {
    step: usize,
    inner: HashMap<String, Box<dyn std::any::Any>>,
}

impl Meta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn step(&self) -> &usize {
        &self.step
    }

    pub fn set<Value: std::any::Any>(&mut self, key: &str, value: Value) -> &mut Self {
        self.inner.entry(key.to_string()).or_insert(Box::new(value));
        self
    }

    pub fn merge(&mut self, other: Self) -> &mut Self {
        for (key, value) in other.inner {
            self.set(&key, value);
        }

        self
    }
}

impl std::ops::Deref for Meta {
    type Target = HashMap<String, Box<dyn std::any::Any>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
