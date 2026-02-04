# loom-config

Configuration management for the Loom ecosystem.

## Features

- `json` - JSON configuration support
- `yaml` - YAML configuration support
- `toml` - TOML configuration support

## Key Types

### ConfigBuilder

Builder pattern for constructing configuration from multiple sources.

### ConfigRoot / ConfigSection

Type-safe configuration access with hierarchical paths.

### Providers

- `MemoryProvider` - In-memory configuration
- `FileProvider` - File-based configuration
- `EnvProvider` - Environment variable configuration

## Macros

- `get!(config, "path.to.value")` - Get configuration value
- `get!(config, "path", int)` - Get typed value (int, float, bool, str)
- `set!(config, "path", value)` - Set configuration value

## Usage

```toml
[dependencies]
loom-config = { version = "0.0.1", features = ["json"] }
```

```rust
use loom_config::{ConfigBuilder, ConfigRoot, get, set};

let config = ConfigBuilder::new()
    .with_provider(MemoryProvider::new())
    .build();

set!(config, "database.host", "localhost");
let host: Option<&str> = get!(config, "database.host", str);
```
