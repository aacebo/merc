# Phase 11: Result Operators

**Status:** PENDING

**Crate:** loom-pipe

**Depends on:** Phase 06 (fork/join API patterns)

## Goal

Add operators for working with Result<T, E> and Option<T> types in pipelines.

## Operators

### retry(n)

Retry failed operations up to n times:

```rust
// Only available when pipeline returns Result<T, E>
source
    .map(|x| fallible_operation(x))
    .retry(3)  // up to 3 attempts
```

With exponential backoff:

```rust
source
    .map(|x| fallible_operation(x))
    .retry_with(RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
    })
```

### expect

Unwrap Result/Option with custom error message (panics on failure):

```rust
source
    .map(|x| parse(x))
    .expect("parsing should not fail")
```

### unwrap

Unwrap Result/Option (panics on failure with default message):

```rust
source
    .map(|x| parse(x))
    .unwrap()
```

### unwrap_or

Provide default value on failure:

```rust
source
    .map(|x| parse(x))
    .unwrap_or(default_value)
```

### unwrap_or_else

Provide default via closure:

```rust
source
    .map(|x| parse(x))
    .unwrap_or_else(|| compute_default())
```

### ok

Convert Result to Option (discarding error):

```rust
source
    .map(|x| parse(x))
    .ok()  // Result<T, E> -> Option<T>
```

## Implementation

### File: `libs/loom-pipe/src/operators/result.rs`

```rust
/// Retry configuration
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub backoff_multiplier: f64,
}

/// Retry operator
pub struct Retry<I, F> {
    input: I,
    operation: F,
    config: RetryConfig,
}

impl<I, F, T, E> Pipe for Retry<I, F>
where
    I: Pipe,
    F: Fn(I::Output) -> Result<T, E>,
{
    type Output = Result<T, E>;

    fn run(self) -> Self::Output {
        let value = self.input.run();
        let mut attempts = 0;
        let mut delay = self.config.initial_delay;

        loop {
            match (self.operation)(value.clone()) {
                Ok(v) => return Ok(v),
                Err(e) if attempts < self.config.max_attempts => {
                    attempts += 1;
                    std::thread::sleep(delay);
                    delay = Duration::from_secs_f64(
                        delay.as_secs_f64() * self.config.backoff_multiplier
                    );
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

### Trait Extension

```rust
pub trait ResultPipe: Pipe {
    fn retry(self, max_attempts: usize) -> Retry<Self>
    where
        Self: Sized,
        Self::Output: Clone;

    fn expect(self, msg: &str) -> Expect<Self>
    where
        Self: Sized;

    fn unwrap(self) -> Unwrap<Self>
    where
        Self: Sized;

    fn unwrap_or<T>(self, default: T) -> UnwrapOr<Self, T>
    where
        Self: Sized;
}
```

## Files to Create

| File | Description |
|------|-------------|
| `libs/loom-pipe/src/operators/result.rs` | Result operators |
| `libs/loom-pipe/src/operators/option.rs` | Option operators (if separate) |

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-pipe/src/operators/mod.rs` | Export new modules |
| `libs/loom-pipe/src/lib.rs` | Re-export operators |

## Verification

1. `cargo build -p loom-pipe`
2. `cargo test -p loom-pipe`
3. Test retry with failing operation
4. Test unwrap variants with success/failure cases
