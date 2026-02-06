# Phase 6: Fork/Join

**Status: PENDING**

## Overview

Rename parallel pipeline operations to use "fork" terminology and add `.join()` for collecting results.

## Current Structure

In `loom-pipe/src/operators/`:

- `spawn.rs` - `Spawn` operator, `SpawnPipe` trait with `.spawn(f)` → returns `Task<O>`
- `parallel.rs` - `Parallel` operator, `ParallelBuilder` with `.spawn(f)` → returns `Vec<TaskResult<O>>`
- `wait.rs` - `Await` operator, `AwaitPipe` trait with `.wait()` → waits for single task

## Goal State

```rust
// Before
let results = Source::from(10)
    .parallel()
    .spawn(|x| x * 2)
    .spawn(|x| x + 5)
    .build();  // implicitly waits

// After
let results = Source::from(10)
    .parallel()
    .fork(|x| x * 2)
    .fork(|x| x + 5)
    .join();  // explicitly waits, returns Vec<TaskResult<O>>
```

## Implementation Steps

### Step 1: Rename spawn.rs → fork.rs

**File:** `libs/loom-pipe/src/operators/spawn.rs` → `fork.rs`

```rust
// Rename Spawn → Fork
pub struct Fork<Input, Output> {
    f: Box<dyn FnOnce(Input) -> Output + Send>,
}

impl<Input, Output> Fork<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Input) -> Output + Send + 'static,
    {
        Self { f: Box::new(f) }
    }
}

// Rename SpawnPipe → ForkPipe
pub trait ForkPipe<T>: Pipe<T> + Sized
where
    T: Send + 'static,
{
    fn fork<O, F>(self, f: F) -> Source<Task<O>>
    where
        O: Send + 'static,
        F: FnOnce(T) -> O + Send + 'static,
    {
        self.pipe(Fork::new(f))
    }
}

impl<T: Send + 'static, P: Pipe<T> + Sized> ForkPipe<T> for P {}
```

### Step 2: Update ParallelBuilder

**File:** `libs/loom-pipe/src/operators/parallel.rs`

```rust
impl<T, O, P> ParallelBuilder<T, O, P>
where
    T: Clone + Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
{
    /// Fork a new parallel branch (renamed from spawn)
    pub fn fork<F>(mut self, f: F) -> Self
    where
        F: FnOnce(T) -> O + Send + 'static,
    {
        self.parallel = self.parallel.add(f);
        self
    }

    /// Wait for all forked threads and collect results
    pub fn join(self) -> Vec<TaskResult<O>> {
        self.build()
    }
}
```

### Step 3: Update Exports

**File:** `libs/loom-pipe/src/operators/mod.rs`

```rust
mod fork;  // renamed from spawn
mod parallel;
mod wait;

pub use fork::*;  // exports Fork, ForkPipe
pub use parallel::*;
pub use wait::*;
```

**File:** `libs/loom-pipe/src/lib.rs`

Update re-exports to use new names.

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-pipe/src/operators/spawn.rs` | Rename to `fork.rs`, `Spawn` → `Fork`, `SpawnPipe` → `ForkPipe`, `.spawn()` → `.fork()` |
| `libs/loom-pipe/src/operators/parallel.rs` | `.spawn()` → `.fork()`, add `.join()` method |
| `libs/loom-pipe/src/operators/mod.rs` | Update module declaration and exports |
| `libs/loom-pipe/src/lib.rs` | Update re-exports |

## API Changes

| Before | After |
|--------|-------|
| `Spawn` | `Fork` |
| `SpawnPipe` | `ForkPipe` |
| `.spawn(f)` | `.fork(f)` |
| `ParallelBuilder::spawn(f)` | `ParallelBuilder::fork(f)` |
| `.build()` (on ParallelBuilder) | `.join()` (explicit, more intuitive) |

## Verification

1. `cargo build -p loom-pipe`
2. `cargo test -p loom-pipe`
3. Update any usages in loom-runtime/loom-cli
4. Ensure all tests pass with new naming
