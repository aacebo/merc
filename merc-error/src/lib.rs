mod builder;
mod code;
mod group;

pub use builder::*;
pub use code::*;
pub use group::*;

use std::{any::Any, backtrace::Backtrace, collections::BTreeMap, sync::Arc};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    code: ErrorCode,
    message: Option<String>,
    fields: BTreeMap<String, String>,
    backtrace: Option<Arc<Backtrace>>,
    inner: Option<Arc<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    pub fn new() -> Self {
        Self {
            code: ErrorCode::default(),
            message: None,
            fields: BTreeMap::new(),
            backtrace: None,
            inner: None,
        }
    }

    pub fn builder() -> ErrorBuilder {
        ErrorBuilder::new()
    }

    pub fn panic(info: Box<dyn Any + Send>) -> Self {
        let message = if let Some(s) = info.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        Self::builder()
            .code(ErrorCode::Unknown)
            .message(format!("Task panicked: {}", message))
            .build()
    }

    pub fn code(&self) -> &ErrorCode {
        &self.code
    }

    pub fn message(&self) -> Option<&str> {
        match &self.message {
            None => None,
            Some(v) => Some(v.as_str()),
        }
    }

    pub fn field(&self, name: &str) -> Option<&str> {
        match &self.fields.get(name) {
            None => None,
            Some(v) => Some(v),
        }
    }

    pub fn backtrace(&self) -> Option<&Backtrace> {
        match &self.backtrace {
            None => None,
            Some(v) => Some(v),
        }
    }

    pub fn inner(&self) -> Option<&dyn std::error::Error> {
        match &self.inner {
            None => None,
            Some(v) => Some(v.as_ref()),
        }
    }
}

impl<T: std::error::Error + Send + Sync + 'static> From<T> for Error {
    fn from(value: T) -> Self {
        Self {
            code: ErrorCode::default(),
            message: None,
            fields: BTreeMap::new(),
            backtrace: None,
            inner: Some(Arc::new(value)),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[ERROR::{}]", &self.code)?;

        if let Some(backtrace) = &self.backtrace {
            writeln!(f, "\tbacktrace: {}", backtrace)?;
        }

        if let Some(error) = &self.inner {
            writeln!(f, "\tinner error: {}", error)?;
        }

        if let Some(message) = &self.message {
            writeln!(f, "\tmessage: {}", message)?;
        }

        Ok(())
    }
}
