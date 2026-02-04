mod document;
mod entity;
mod error;
mod etag;
mod id;
mod record;
pub mod sources;

pub use document::*;
pub use entity::*;
pub use error::*;
pub use etag::*;
pub use id::*;
pub use record::*;

use async_trait::async_trait;

use crate::path::Path;

#[async_trait]
pub trait DataSource: Send + Sync {
    async fn exists(&self, path: &Path) -> Result<bool, ReadError>;
    async fn count(&self, path: &Path) -> Result<usize, ReadError>;
    async fn find_one(&self, path: &Path) -> Result<Record, ReadError>;
    async fn find(&self, path: &Path) -> Result<Vec<Record>, ReadError>;

    async fn create(&self, record: Record) -> Result<(), WriteError>;
    async fn update(&self, record: Record) -> Result<(), WriteError>;
    async fn upsert(&self, record: Record) -> Result<(), WriteError>;
    async fn delete(&self, path: &Path) -> Result<(), WriteError>;
}
