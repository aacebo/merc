# Loom Libraries

The `loom` crate provides unified access to all framework components through feature-gated modules. It follows a layered architecture where foundational types flow upward through data, format, and processing layers, culminating in a runtime that orchestrates everything together.

## Architecture

```mermaid
flowchart TB
    cli["cli"]

    subgraph loom["loom"]
        direction TB

        subgraph Orchestration["Orchestration Layer"]
            runtime["runtime"]
        end

        subgraph ML["ML Layer"]
            cortex["cortex"]
        end

        subgraph Processing["Processing Layer"]
            pipe["pipe"]
            sync["sync"]
        end

        subgraph Format["Format Layer"]
            codec["codec"]
            signal["signal"]
        end

        subgraph Data["Data Layer"]
            io["io"]
            config["config"]
        end

        subgraph Foundation["Foundation Layer"]
            error["error"]
            assert["assert"]
            core["core"]
        end
    end

    %% Layer Dependencies
    cli --> loom
    Orchestration --> ML
    Orchestration --> Processing
    Orchestration --> Format
    Orchestration --> Data
    Orchestration --> Foundation
    ML --> Data
    Processing --> Foundation
    Format --> Data
    Format --> Foundation
    Data --> Foundation

    %% Links
    click cli "https://github.com/aacebo/loom/blob/main/libs/loom-cli/README.md" _blank
    click runtime "https://github.com/aacebo/loom/blob/main/libs/loom-runtime/README.md" _blank
    click cortex "https://github.com/aacebo/loom/blob/main/libs/loom-cortex/README.md" _blank
    click pipe "https://github.com/aacebo/loom/blob/main/libs/loom-pipe/README.md" _blank
    click sync "https://github.com/aacebo/loom/blob/main/libs/loom-sync/README.md" _blank
    click codec "https://github.com/aacebo/loom/blob/main/libs/loom-codec/README.md" _blank
    click signal "https://github.com/aacebo/loom/blob/main/libs/loom-signal/README.md" _blank
    click io "https://github.com/aacebo/loom/blob/main/libs/loom-io/README.md" _blank
    click config "https://github.com/aacebo/loom/blob/main/libs/loom-config/README.md" _blank
    click error "https://github.com/aacebo/loom/blob/main/libs/loom-error/README.md" _blank
    click assert "https://github.com/aacebo/loom/blob/main/libs/loom-assert/README.md" _blank
    click core "https://github.com/aacebo/loom/blob/main/libs/loom-core/README.md" _blank
    click loom "https://github.com/aacebo/loom/blob/main/libs/loom/README.md" _blank

    %% Styles
    style loom fill:#2a2a2a,stroke:#444,color:#fff
    style Orchestration fill:#1a1a1a,stroke:#333,color:#fff
    style ML fill:#1a1a1a,stroke:#333,color:#fff
    style Processing fill:#1a1a1a,stroke:#333,color:#fff
    style Format fill:#1a1a1a,stroke:#333,color:#fff
    style Data fill:#1a1a1a,stroke:#333,color:#fff
    style Foundation fill:#1a1a1a,stroke:#333,color:#fff
```

## Crates

### Foundation Layer

| Crate | Description |
|-------|-------------|
| [**loom-error**](./loom-error/README.md) | Error handling utilities with error codes, messages, and backtraces |
| [**loom-assert**](./loom-assert/README.md) | Assertion utilities for testing |
| [**loom-core**](./loom-core/README.md) | Core data types, value system, formats, identifiers, and paths |

### Data Layer

| Crate | Description |
|-------|-------------|
| [**loom-io**](./loom-io/README.md) | Async I/O and data source abstraction with CRUD operations |
| [**loom-config**](./loom-config/README.md) | Configuration management with multiple providers (env, file, memory) |

### Format Layer

| Crate | Description |
|-------|-------------|
| [**loom-codec**](./loom-codec/README.md) | Format encoding/decoding registry (JSON, YAML, TOML) |
| [**loom-signal**](./loom-signal/README.md) | Event/signal emission system for telemetry and observability |

### Processing Layer

| Crate | Description |
|-------|-------------|
| [**loom-pipe**](./loom-pipe/README.md) | Data pipeline and stream processing with operators (Map, Filter, FanOut, Router, Parallel) |
| [**loom-sync**](./loom-sync/README.md) | Synchronization primitives for async operations |

### ML Layer

| Crate | Description |
|-------|-------------|
| [**loom-cortex**](./loom-cortex/README.md) | Machine learning and neural network capabilities (PyTorch, BERT) |

### Orchestration Layer

| Crate | Description |
|-------|-------------|
| [**loom-runtime**](./loom-runtime/README.md) | Core runtime orchestration integrating all components |

### CLI

| Crate | Description |
|-------|-------------|
| [**loom-cli**](./loom-cli/README.md) | Command-line interface binary |
