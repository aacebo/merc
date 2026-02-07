# Phase 03: CLI Command Structs

**Status:** PENDING

**Crate:** loom-cli

**Depends on:** (none)

## Goal

Refactor CLI command execution to use dedicated struct types with clap-based validation instead of loose function parameters.

## Current State

Commands use `exec` methods with multiple parameters:

```rust
// Current approach
fn exec(config: Option<PathBuf>, input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    // ...
}
```

## Target State

Each command has a dedicated struct with clap derives:

```rust
#[derive(Debug, clap::Args)]
pub struct RunCommand {
    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Input file path
    #[arg(required = true)]
    input: PathBuf,

    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,
}

impl RunCommand {
    pub fn exec(self) -> Result<()> {
        // ...
    }
}
```

## Benefits

1. **Type Safety** - Parameters bundled in validated struct
2. **Clap Integration** - Validation via derive macros
3. **Self-Documenting** - Struct fields describe the command
4. **Testability** - Easy to construct command structs in tests
5. **Consistency** - Uniform pattern across all commands

## Commands to Refactor

| Command | Current | Target Struct |
|---------|---------|---------------|
| `run` | `exec(config, input, output)` | `RunCommand` |
| `bench` | `exec(config, dataset, ...)` | `BenchCommand` |
| `validate` | `exec(config)` | `ValidateCommand` |

## Implementation TBD

Detailed implementation to be designed when phase begins. Key considerations:

- Subcommand struct composition with clap
- Error handling patterns
- Config loading consolidation
- Progress/output handling
