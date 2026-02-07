# Phase 02: Time Operators

**Status:** PENDING

**Crate:** loom-pipe

**Depends on:** Phase 01 (collection operators)

## Goal

Add time-based operators for timeout and rate limiting in pipelines.

## Operators

### timeout

Fail with TimeoutError after specified duration:

```rust
source
    .map(|x| slow_operation(x))
    .timeout(Duration::from_secs(5))
```

Returns `Result<T, TimeoutError>`:

```rust
match source.timeout(Duration::from_secs(5)).run() {
    Ok(value) => println!("Got: {:?}", value),
    Err(TimeoutError) => println!("Operation timed out"),
}
```

### debounce

Rate limiting - only allow one call within time window:

```rust
// If multiple values arrive within 100ms, only process the last one
source
    .debounce(Duration::from_millis(100))
```

### throttle

Limit execution rate:

```rust
// At most one execution per 100ms
source
    .throttle(Duration::from_millis(100))
```

### delay

Add delay before execution:

```rust
source
    .delay(Duration::from_millis(500))
    .map(|x| process(x))
```

## Implementation

### File: `libs/loom-pipe/src/operators/time.rs`

```rust
use std::time::Duration;

/// Timeout error
#[derive(Debug, Clone)]
pub struct TimeoutError {
    pub duration: Duration,
}

/// Timeout operator
pub struct Timeout<I> {
    input: I,
    duration: Duration,
}

impl<I> Pipe for Timeout<I>
where
    I: Pipe + Send + 'static,
    I::Output: Send + 'static,
{
    type Output = Result<I::Output, TimeoutError>;

    fn run(self) -> Self::Output {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();
        let duration = self.duration;

        thread::spawn(move || {
            let result = self.input.run();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(duration) {
            Ok(result) => Ok(result),
            Err(_) => Err(TimeoutError { duration }),
        }
    }
}

/// Debounce operator (async context)
pub struct Debounce<I> {
    input: I,
    duration: Duration,
}

/// Delay operator
pub struct Delay<I> {
    input: I,
    duration: Duration,
}

impl<I> Pipe for Delay<I>
where
    I: Pipe,
{
    type Output = I::Output;

    fn run(self) -> Self::Output {
        std::thread::sleep(self.duration);
        self.input.run()
    }
}
```

### Async Variants

For async pipelines:

```rust
impl<I> AsyncPipe for Timeout<I>
where
    I: AsyncPipe + Send + 'static,
    I::Output: Send + 'static,
{
    type Output = Result<I::Output, TimeoutError>;

    async fn run(self) -> Self::Output {
        tokio::time::timeout(self.duration, self.input.run())
            .await
            .map_err(|_| TimeoutError { duration: self.duration })
    }
}
```

### Trait Extension

```rust
pub trait TimePipe: Pipe {
    fn timeout(self, duration: Duration) -> Timeout<Self>
    where
        Self: Sized;

    fn delay(self, duration: Duration) -> Delay<Self>
    where
        Self: Sized;

    fn throttle(self, duration: Duration) -> Throttle<Self>
    where
        Self: Sized;
}
```

## Files to Create

| File | Description |
|------|-------------|
| `libs/loom-pipe/src/operators/time.rs` | Time-based operators |
| `libs/loom-pipe/src/error.rs` | TimeoutError and related types |

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-pipe/src/operators/mod.rs` | Export new module |
| `libs/loom-pipe/src/lib.rs` | Re-export operators and errors |

## Considerations

- **Sync vs Async**: Different implementations needed for sync/async contexts
- **Thread spawning**: Sync timeout requires spawning a thread
- **Debounce state**: Requires maintaining state between calls (may need wrapper type)
- **Cancellation**: Consider if running operations should be cancelled on timeout

## Verification

1. `cargo build -p loom-pipe`
2. `cargo test -p loom-pipe`
3. Test timeout with operation that exceeds duration
4. Test delay adds expected wait time
5. Test throttle limits execution rate
