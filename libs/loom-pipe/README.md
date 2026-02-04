# loom-pipe

Pipeline and operator traits for the Loom ecosystem.

## Key Traits

### Operator

Transform a source into a new source:

```rust
pub trait Operator<Input> {
    type Output;
    fn apply(self, src: Source<Input>) -> Source<Self::Output>;
}
```

### Pipe

Chain operators together:

```rust
pub trait Pipe<Input> {
    fn pipe<Op: Operator<Input>>(self, op: Op) -> Source<Op::Output>;
}
```

### Build

Execute the pipeline and produce a result:

```rust
pub trait Build {
    type Output;
    fn build(self) -> Self::Output;
}
```

## Key Types

### Source

Wrapper around a lazy computation:

```rust
let source = Source::from(value);
let source = Source::new(|| compute_value());
```

### Transformer

Transform input to output:

```rust
let transformer = Transformer::new(source, |input| transform(input));
```

## Built-in Operators

### MapOperator

Transform values with a function:

```rust
let result = Source::from(1)
    .map(|x| x * 2)
    .build();
```

## Usage

```toml
[dependencies]
loom-pipe = "0.0.1"
```

```rust
use loom_pipe::{Source, Build, Pipe};

let result = Source::from(42)
    .map(|x| x * 2)
    .build();

assert_eq!(result, 84);
```
