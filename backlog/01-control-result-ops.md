# Phase 01: Control Flow & Result Operators

**Status:** PENDING

**Crate:** loom-pipe

**Depends on:** (none)

## Goal

Add conditional branching, logical control flow, and Result/Option operators to loom-pipe.

---

## Control Flow Operators

### if_then_else

Conditional branching based on predicate:

```rust
source
    .if_then_else(
        |x| x > 0,      // condition
        |x| x * 2,      // then branch
        |x| x.abs(),    // else branch
    )
```

### when (if without else)

Execute only when condition is true:

```rust
source
    .when(|x| x > 0, |x| x * 2)  // Only transforms when positive
```

### and

Short-circuit logical AND - fails fast if condition fails:

```rust
source
    .and(|x| validate(x))  // Returns Err if validation fails
```

### or

Short-circuit logical OR - provides fallback:

```rust
source
    .or(|| default_value())  // Uses fallback if source fails
```

---

## Result Operators

### retry(n)

Retry failed operations up to n times:

```rust
source
    .map(|x| fallible_operation(x))
    .retry(3)  // up to 3 attempts
```

With exponential backoff:

```rust
source
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

---

## Implementation

### File: `libs/loom-pipe/src/operators/branch.rs`

```rust
use crate::Pipe;

/// Conditional branching operator
pub struct IfThenElse<I, C, T, E> {
    input: I,
    condition: C,
    then_fn: T,
    else_fn: E,
}

impl<I, C, T, E, O> Pipe for IfThenElse<I, C, T, E>
where
    I: Pipe,
    C: Fn(&I::Output) -> bool,
    T: Fn(I::Output) -> O,
    E: Fn(I::Output) -> O,
{
    type Output = O;

    fn run(self) -> Self::Output {
        let value = self.input.run();
        if (self.condition)(&value) {
            (self.then_fn)(value)
        } else {
            (self.else_fn)(value)
        }
    }
}
```

### File: `libs/loom-pipe/src/operators/result.rs`

```rust
use std::time::Duration;

/// Retry configuration
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub backoff_multiplier: f64,
}

/// Retry operator
pub struct Retry<I> {
    input: I,
    config: RetryConfig,
}

impl<I, T, E> Pipe for Retry<I>
where
    I: Pipe<Output = Result<T, E>>,
    I: Clone,
{
    type Output = Result<T, E>;

    fn run(self) -> Self::Output {
        let mut attempts = 0;
        let mut delay = self.config.initial_delay;

        loop {
            match self.input.clone().run() {
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

---

## Files to Create

| File | Description |
|------|-------------|
| `libs/loom-pipe/src/operators/branch.rs` | IfThenElse, When operators |
| `libs/loom-pipe/src/operators/logical.rs` | And, Or operators |
| `libs/loom-pipe/src/operators/result.rs` | Retry, Unwrap, Expect operators |

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-pipe/src/operators/mod.rs` | Export new modules |
| `libs/loom-pipe/src/lib.rs` | Re-export operators |

## Verification

1. `cargo build -p loom-pipe`
2. `cargo test -p loom-pipe`
3. Test conditional branching with various predicates
4. Test short-circuit behavior of and/or
5. Test retry with failing operation
6. Test unwrap variants with success/failure cases
