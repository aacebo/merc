use std::{backtrace::Backtrace, collections::BTreeMap, rc::Rc};

use crate::{Error, ErrorCode};

pub struct ErrorBuilder {
    code: ErrorCode,
    message: Option<String>,
    fields: BTreeMap<String, String>,
    backtrace: Option<Rc<Backtrace>>,
    inner: Option<Rc<dyn std::error::Error + 'static>>,
}

impl ErrorBuilder {
    pub fn new() -> Self {
        Self {
            code: ErrorCode::default(),
            message: None,
            fields: BTreeMap::new(),
            backtrace: None,
            inner: None,
        }
    }

    pub fn code(mut self, code: ErrorCode) -> Self {
        self.code = code;
        self
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
        self.backtrace = Some(Rc::new(Backtrace::force_capture()));
        self
    }

    pub fn inner<TError: std::error::Error + 'static>(mut self, inner: TError) -> Self {
        self.inner = Some(Rc::new(inner));
        self
    }

    pub fn build(self) -> Error {
        Error {
            code: self.code,
            message: self.message,
            fields: self.fields,
            backtrace: self.backtrace,
            inner: self.inner,
        }
    }
}
