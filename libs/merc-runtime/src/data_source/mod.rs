mod error;

pub use error::*;

use crate::Document;

pub trait DataSource {
    fn read(&self) -> Result<Document, ReadError>;
    fn write(&self, document: Document) -> Result<(), WriteError>;
}
