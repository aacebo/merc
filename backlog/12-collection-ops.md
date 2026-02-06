# Phase 12: Collection Operators

**Status:** PENDING

**Crate:** loom-pipe

**Depends on:** Phase 06 (fork/join API patterns)

## Goal

Add operators for working with iterable collections in pipelines.

## Operators

### flatten

Flatten nested collections:

```rust
// Vec<Vec<T>> -> Vec<T>
source
    .map(|x| vec![x, x+1, x+2])
    .flatten()
```

### flat_map

Map and flatten in one step:

```rust
// Equivalent to .map(f).flatten()
source
    .flat_map(|x| vec![x, x+1, x+2])
```

### collect

Collect iterator into collection:

```rust
source
    .filter(|x| x > 0)
    .collect::<Vec<_>>()
```

### concat

Concatenate multiple collections:

```rust
source1.concat(source2)  // Combines outputs
```

### chunk

Split into fixed-size chunks:

```rust
source
    .chunk(3)  // Vec<T> -> Vec<Vec<T>> with max 3 per chunk
```

### window

Sliding window over elements:

```rust
source
    .window(3)  // Produces overlapping windows of size 3
```

## Implementation

### File: `libs/loom-pipe/src/operators/collection.rs`

```rust
/// Flatten operator for nested iterables
pub struct Flatten<I> {
    input: I,
}

impl<I, T> Pipe for Flatten<I>
where
    I: Pipe,
    I::Output: IntoIterator<Item = T>,
    T: IntoIterator,
{
    type Output = Vec<T::Item>;

    fn run(self) -> Self::Output {
        self.input
            .run()
            .into_iter()
            .flat_map(|inner| inner.into_iter())
            .collect()
    }
}

/// Chunk operator
pub struct Chunk<I> {
    input: I,
    size: usize,
}

impl<I, T> Pipe for Chunk<I>
where
    I: Pipe<Output = Vec<T>>,
{
    type Output = Vec<Vec<T>>;

    fn run(self) -> Self::Output {
        let items = self.input.run();
        items.chunks(self.size)
            .map(|c| c.to_vec())
            .collect()
    }
}
```

### Trait Extension

```rust
pub trait CollectionPipe: Pipe {
    fn flatten(self) -> Flatten<Self>
    where
        Self: Sized;

    fn flat_map<F, U>(self, f: F) -> FlatMap<Self, F>
    where
        F: Fn(Self::Output) -> U,
        U: IntoIterator,
        Self: Sized;

    fn chunk(self, size: usize) -> Chunk<Self>
    where
        Self: Sized;

    fn window(self, size: usize) -> Window<Self>
    where
        Self: Sized;
}
```

## Files to Create

| File | Description |
|------|-------------|
| `libs/loom-pipe/src/operators/collection.rs` | Collection operators |

## Files to Modify

| File | Changes |
|------|---------|
| `libs/loom-pipe/src/operators/mod.rs` | Export new module |
| `libs/loom-pipe/src/lib.rs` | Re-export operators |

## Verification

1. `cargo build -p loom-pipe`
2. `cargo test -p loom-pipe`
3. Test flatten with nested vectors
4. Test chunk/window with various sizes
