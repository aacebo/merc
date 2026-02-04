# loom-error

Error types and handling for the Loom ecosystem.

## Key Types

### Error

Structured error type with:
- `code` - Error classification
- `message` - Human-readable description
- `fields` - Additional context fields
- `backtrace` - Optional stack trace
- `inner` - Nested error chain

### ErrorCode

Error classification enum:
- `Unknown`
- `Cancel`
- `Internal`
- `NotFound`
- `InvalidArgument`
- `Unauthorized`
- `PermissionDenied`
- And more...

### ErrorBuilder

Builder pattern for constructing errors:

```rust
Error::builder()
    .code(ErrorCode::NotFound)
    .message("Resource not found")
    .field("id", "123")
    .build()
```

### Result Type

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Usage

```toml
[dependencies]
loom-error = "0.0.1"
```

```rust
use loom_error::{Error, ErrorCode, Result};

fn my_function() -> Result<()> {
    Err(Error::builder()
        .code(ErrorCode::InvalidArgument)
        .message("Invalid input")
        .build())
}
```
