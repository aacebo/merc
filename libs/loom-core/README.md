# loom-core

Core types and enums for the Loom ecosystem.

## Features

- `json` - JSON serialization support via serde_json
- `yaml` - YAML serialization support via saphyr
- `toml` - TOML serialization support via toml

## Key Types

### Value

A generic value type supporting:
- `Null`
- `Bool`
- `Number` (Int/Float)
- `String`
- `Array`
- `Object`

### Format

Enum representing data formats:
- `Json`
- `Yaml`
- `Toml`
- `Xml`
- `Csv`
- `Markdown`
- `Html`
- `Text`
- `Binary`

### MediaType

67+ MIME type variants including text, code, images, audio, video, and archives.

### Path

Path abstractions:
- `FilePath` - File system paths
- `FieldPath` - Object field paths
- `UriPath` - URI paths

## Usage

```toml
[dependencies]
loom-core = { version = "0.0.1", features = ["json"] }
```

```rust
use loom_core::{Value, Format, MediaType};
use loom_core::value::Object;

let mut obj = Object::new();
obj.insert("key".to_string(), Value::String("value".to_string()));
let value = Value::Object(obj);
```
