# Loom Backlog

## Status Overview

| Phase | Description | Crate | Status |
|-------|-------------|-------|--------|
| [01-error-aggregation](01-error-aggregation.md) | Hierarchical layer errors | runtime | PENDING |
| [02-control-flow-ops](02-control-flow-ops.md) | if/then/else, and/or | pipe | PENDING |
| [03-result-operators](03-result-operators.md) | retry, expect/unwrap | pipe | PENDING |
| [04-collection-ops](04-collection-ops.md) | flatten, chunk, window | pipe | PENDING |
| [05-time-operators](05-time-operators.md) | timeout, debounce | pipe | PENDING |
| [06-multi-file-merge](06-multi-file-merge.md) | Config file includes/refs | config | PENDING |

## Priority Tiers

### Tier 1: Runtime Infrastructure
- **Phase 01**: Error aggregation - Hierarchical error support

### Tier 2: Pipe Operators - Foundation
- **Phase 02**: Control flow - if/then/else, and/or
- **Phase 03**: Result operators - retry, expect/unwrap

### Tier 3: Pipe Operators - Advanced
- **Phase 04**: Collection operators - flatten, chunk, window
- **Phase 05**: Time operators - timeout, debounce

### Tier 4: Config Enhancement
- **Phase 06**: Multi-file config merge

## Dependencies

```
Phase 01 (Errors) - Independent


Phase 02 (Control) ──┬─► Phase 03 (Result)
                     │
                     └─► Phase 04 (Collection)
                              │
                              └──► Phase 05 (Time)


Phase 06 (Config) - Independent
```

## Completed Work Summary

The following phases have been completed and their documentation archived:

- **Config Integration** - loom-config crate integrated with env var support
- **Validation** - Config validation with garde derive macros
- **Pipeline Rewrite** - Pipeline infrastructure with Layer trait
- **Dynamic Layers** - Runner removal, config simplification
- **Output Behavior** - CLI output path handling (auto-naming)
- **Fork/Join** - Renamed spawn→fork, added .join()
- **Simplify Structure** - Merged modules, flattened CLI
- **Result Metadata** - Timing and resource metrics (elapsed_ms, throughput)

## Environment Variable Support

Override config via environment variables:

```bash
LOOM_CONCURRENCY=16 loom run -c config.toml ...
LOOM_BATCH__SIZE=32 loom run -c config.toml ...
LOOM_LAYERS_SCORE_THRESHOLD=0.8 loom run -c config.toml ...
```
