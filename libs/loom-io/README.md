# loom-io

Data source abstractions for the Loom ecosystem.

## Features

- `json` - JSON serialization support
- `yaml` - YAML format support
- `toml` - TOML format support

## Key Types

### DataSource Trait

Async trait for data source operations:

```rust
#[async_trait]
pub trait DataSource: Send + Sync {
    fn name(&self) -> &str;
    async fn exists(&self, path: &Path) -> Result<bool, ReadError>;
    async fn count(&self, path: &Path) -> Result<usize, ReadError>;
    async fn find_one(&self, path: &Path) -> Result<Record, ReadError>;
    async fn find(&self, path: &Path) -> Result<Vec<Record>, ReadError>;
    async fn create(&self, record: Record) -> Result<(), WriteError>;
    async fn update(&self, record: Record) -> Result<(), WriteError>;
    async fn upsert(&self, record: Record) -> Result<(), WriteError>;
    async fn delete(&self, path: &Path) -> Result<(), WriteError>;
}
```

### Built-in Sources

- `FileSystemSource` - File system backed storage
- `MemorySource` - In-memory storage

### Supporting Types

- `Record` - Raw data record with path, media type, and content
- `Document` - Decoded document with entities
- `Entity` - Individual data entity with field path and value
- `ETag` - Content hash for change detection
- `Id` - Unique identifier

## Usage

```toml
[dependencies]
loom-io = { version = "0.0.1", features = ["json"] }
```

```rust
use loom_io::{DataSource, Record, MemorySource};

let source = MemorySource::new();
let record = Record::from_str(path, MediaType::TextJson, r#"{"key": "value"}"#);
source.create(record).await?;
```
