mod builder;

pub use builder::*;

use std::{backtrace::Backtrace, collections::BTreeMap};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: Option<String>,
    fields: BTreeMap<String, String>,
    backtrace: Option<Backtrace>,
    inner: Option<Box<dyn std::error::Error + 'static>>,
}

impl Error {
    pub fn new() -> Self {
        Self {
            message: None,
            fields: BTreeMap::new(),
            backtrace: None,
            inner: None,
        }
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

impl<T: std::error::Error + 'static> From<T> for Error {
    fn from(value: T) -> Self {
        Self {
            message: None,
            fields: BTreeMap::new(),
            backtrace: None,
            inner: Some(value.into()),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(backtrace) = &self.backtrace {
            write!(f, "backtrace: {}", backtrace)?;
        }

        if let Some(error) = &self.inner {
            write!(f, "inner error: {}", error)?;
        }

        if let Some(message) = &self.message {
            write!(f, "message: {}", message)?;
        }

        Ok(())
    }
}
