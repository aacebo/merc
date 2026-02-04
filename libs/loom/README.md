# loom

Unified re-export crate for the Loom ecosystem.

## Features

### Crate Features

Enable individual crates as needed:

- `core` - Core types (Value, Path, Format, MediaType)
- `config` - Configuration management
- `io` - Data source abstractions
- `codec` - Encoding/decoding (JSON, YAML, TOML)
- `pipe` - Pipeline/operator traits
- `error` - Error types
- `sync` - Synchronization primitives
- `signal` - Observability abstractions
- `runtime` - Main runtime orchestrator
- `full` - Enable all crates

### Format Features

- `json` - JSON support (default)
- `yaml` - YAML support
- `toml` - TOML support

### Async Features

- `tokio` - Tokio async runtime support

## Usage

```toml
[dependencies]
loom = { version = "0.0.1", features = ["full", "json", "yaml", "toml"] }
```

```rust
use loom::core::{Value, Format, MediaType};
use loom::codec::{Codec, JsonCodec};
use loom::config::ConfigBuilder;
```
