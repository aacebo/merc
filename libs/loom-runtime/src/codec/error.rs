use crate::MediaType;
use std::fmt;

#[derive(Debug)]
pub enum CodecError {
    UnsupportedMediaType(MediaType),
    Decode(String),
    Encode(String),
}

impl CodecError {
    pub fn decode<E: std::error::Error>(e: E) -> Self {
        Self::Decode(e.to_string())
    }

    pub fn encode<E: std::error::Error>(e: E) -> Self {
        Self::Encode(e.to_string())
    }

    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::UnsupportedMediaType(_))
    }

    pub fn is_decode(&self) -> bool {
        matches!(self, Self::Decode(_))
    }

    pub fn is_encode(&self) -> bool {
        matches!(self, Self::Encode(_))
    }
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMediaType(mt) => write!(f, "unsupported media type: {}", mt),
            Self::Decode(msg) => write!(f, "decode error: {}", msg),
            Self::Encode(msg) => write!(f, "encode error: {}", msg),
        }
    }
}

impl std::error::Error for CodecError {}

impl From<std::str::Utf8Error> for CodecError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Decode(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for CodecError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Decode(e.to_string())
    }
}
