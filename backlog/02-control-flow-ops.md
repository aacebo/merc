# Phase 02: Control Flow Operators

**Status:** PENDING

**Crate:** loom-pipe

**Depends on:** (none)

## Goal

Add conditional branching and logical control flow operators to loom-pipe.

## Operators

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

## Implementation

### File: `libs/loom-pipe/src/operators/branch.rs`

```rust
use crate::{Pipe, Source};

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

### Trait Extension

```rust
pub trait BranchPipe: Pipe {
    fn if_then_else<C, T, E, O>(self, cond: C, then_fn: T, else_fn: E) -> IfThenElse<Self, C, T, E>
    where
        C: Fn(&Self::Output) -> bool,
        T: Fn(Self::Output) -> O,
        E: Fn(Self::Output) -> O,
        Self: Sized,
    {
        IfThenElse {
            input: self,
            condition: cond,
            then_fn,
            else_fn,
        }
    }

    fn when<C, F>(self, cond: C, f: F) -> When<Self, C, F>
    where
        C: Fn(&Self::Output) -> bool,
        F: Fn(Self::Output) -> Self::Output,
        Self: Sized;
}
```

## Files to Create

| File | Description |
|------|-------------|
| `libs/loom-pipe/src/operators/branch.rs` | IfThenElse, When operators |
| `libs/loom-pipe/src/operators/logical.rs` | And, Or operators |

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
