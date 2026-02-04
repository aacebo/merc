# loom-sync

Synchronization primitives for the Loom ecosystem.

## Features

- `tokio` - Tokio async runtime support

## Modules

### chan

Channel abstractions for inter-task communication.

### tasks

Task management utilities.

## Usage

```toml
[dependencies]
loom-sync = { version = "0.0.1", features = ["tokio"] }
```

```rust
use loom_sync::chan;
use loom_sync::tasks;
```
