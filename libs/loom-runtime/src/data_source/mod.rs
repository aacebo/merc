mod document;
mod entity;
mod error;
mod etag;
mod id;
pub mod sources;

pub use document::*;
pub use entity::*;
pub use error::*;
pub use etag::*;
pub use id::*;

use crate::path::Path;
use async_trait::async_trait;

#[async_trait]
pub trait DataSource {
    fn exists(&self, path: &Path) -> Result<bool, ReadError>;
    fn count(&self, path: &Path) -> Result<usize, ReadError>;
    fn find_one(&self, path: &Path) -> Result<Document, ReadError>;
    fn find(&self, path: &Path) -> Result<Vec<Document>, ReadError>;

    fn create(&self, document: Document) -> Result<(), WriteError>;
    fn update(&self, document: Document) -> Result<(), WriteError>;
    fn upsert(&self, document: Document) -> Result<(), WriteError>;
    fn delete(&self, path: &Path) -> Result<(), WriteError>;
}
