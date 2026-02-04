# loom-runtime

Core runtime library for Loom providing data source abstractions, codec transformations, and content type handling.

## Architecture

```mermaid
erDiagram
    Runtime ||--o{ Codec : owns
    Runtime ||--o{ DataSource : owns

    Codec ||--|| Format : returns
    Codec ||--|{ Record : decodes
    Codec ||--|{ Document : encodes

    DataSource ||--|{ Record : manages

    Record ||--|| MediaType : has
    Record ||--|| Path : has
    Record ||--|| Id : has
    Record ||--|| ETag : has

    Document ||--|| MediaType : has
    Document ||--|| Path : has
    Document ||--|| Id : has
    Document ||--|| ETag : has
    Document ||--o{ Entity : contains

    MediaType ||--|| Format : converts_to

    Entity ||--|| Value : contains

    Runtime {
        Vec codecs
        Vec sources
    }

    Codec {
        format() Format
        decode() Document
        encode() Record
    }

    DataSource {
        async exists() bool
        async count() usize
        async find_one() Record
        async find() Records
        async create() void
        async update() void
        async upsert() void
        async delete() void
    }

    Record {
        Id id
        ETag etag
        Path path
        usize size
        MediaType media_type
        bytes content
    }

    Document {
        Id id
        ETag etag
        Path path
        usize size
        MediaType media_type
        Vec entities
    }

    MediaType {
        TextJson variant
        TextYaml variant
        TextPlain variant
        CodeRust variant
    }

    Format {
        Json variant
        Yaml variant
        Text variant
        Binary variant
    }
```

## Key Types

| Type | Description |
|------|-------------|
| **Runtime** | Top-level container holding codecs and data sources |
| **DataSource** | Async trait for storage backends (file system, memory, etc.) |
| **Codec** | Trait for encoding/decoding between Record and Document |
| **Record** | Raw bytes with metadata (storage layer) |
| **Document** | Parsed content with entities (application layer) |
| **MediaType** | Content type classification (78 variants) |
| **Format** | High-level format grouping (Json, Yaml, Text, Binary, etc.) |

## Data Flow

```
Read:  DataSource.find_one().await -> Record -> Codec.decode() -> Document
Write: Document -> Codec.encode() -> Record -> DataSource.create().await
```

## Usage

```rust
use loom_runtime::{Builder, FileSystemSource, JsonCodec, TextCodec};

let runtime = Builder::new()
    .codec(JsonCodec::new())
    .codec(TextCodec::new())
    .source(FileSystemSource::new())
    .build();
```
