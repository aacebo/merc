# loom-codec

Encoding and decoding implementations for the Loom ecosystem.

## Features

- `json` - JSON codec via serde_json
- `yaml` - YAML codec via saphyr
- `toml` - TOML codec via toml

## Codec Trait

```rust
pub trait Codec: Send + Sync {
    fn format(&self) -> Format;
    fn decode(&self, record: Record) -> Result<Document, CodecError>;
    fn encode(&self, document: Document) -> Result<Record, CodecError>;
}
```

## Built-in Codecs

### JsonCodec

```rust
let codec = JsonCodec::new();         // Compact output
let codec = JsonCodec::pretty();       // Pretty-printed output
```

### YamlCodec

```rust
let codec = YamlCodec::new();
```

### TomlCodec

```rust
let codec = TomlCodec::new();          // Compact output
let codec = TomlCodec::pretty();       // Pretty-printed output
```

### TextCodec

Plain text handling (always available).

```rust
let codec = TextCodec::new();
```

## Usage

```toml
[dependencies]
loom-codec = { version = "0.0.1", features = ["json", "yaml", "toml"] }
```

```rust
use loom_codec::{Codec, JsonCodec, Record, Document};

let codec = JsonCodec::new();
let document = codec.decode(record)?;
let record = codec.encode(document)?;
```
