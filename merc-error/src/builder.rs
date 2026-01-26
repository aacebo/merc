use std::{backtrace::Backtrace, collections::BTreeMap};

use crate::Error;

pub struct ErrorBuilder {
    message: Option<String>,
    fields: BTreeMap<String, String>,
    backtrace: Option<Backtrace>,
    inner: Option<Box<dyn std::error::Error + 'static>>,
}

impl ErrorBuilder {
    pub fn new() -> Self {
        Self {
            message: None,
            fields: BTreeMap::new(),
            backtrace: None,
            inner: None,
        }
    }

    pub fn message<T: ToString>(mut self, message: T) -> Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn field<Value: ToString>(mut self, name: &str, value: Value) -> Self {
        self.fields.insert(name.to_string(), value.to_string());
        self
    }

    pub fn backtrace(mut self) -> Self {
        self.backtrace = Some(Backtrace::force_capture());
        self
    }

    pub fn inner<TError: std::error::Error + 'static>(mut self, inner: TError) -> Self {
        self.inner = Some(inner.into());
        self
    }

    pub fn build(self) -> Error {
        Error {
            message: self.message,
            fields: self.fields,
            backtrace: self.backtrace,
            inner: self.inner,
        }
    }
}
