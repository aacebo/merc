# Loom Backlog

## Status Overview

| Phase | Description | Crate | Status |
|-------|-------------|-------|--------|
| [01-control-result-ops](01-control-result-ops.md) | if/then/else, and/or, retry, unwrap | pipe | PENDING |
| [02-collection-ops](02-collection-ops.md) | flatten, chunk, window | pipe | PENDING |
| [03-time-operators](03-time-operators.md) | timeout, debounce | pipe | PENDING |
| [04-multi-file-merge](04-multi-file-merge.md) | Config file includes/refs | config | PENDING |

## Priority Tiers

### Tier 1: Pipe Operators - Foundation
- **Phase 01**: Control flow & result ops - if/then/else, and/or, retry, unwrap

### Tier 2: Pipe Operators - Advanced
- **Phase 02**: Collection operators - flatten, chunk, window
- **Phase 03**: Time operators - timeout, debounce

### Tier 3: Config Enhancement
- **Phase 04**: Multi-file config merge

## Dependencies

```
Phase 01 (Control/Result) ──┬─► Phase 02 (Collection)
                            │
                            └─► Phase 03 (Time)


Phase 04 (Config) - Independent
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
- **Error Aggregation** - Hierarchical layer errors with loom_error::Result support

## Environment Variable Support

Override config via environment variables:

```bash
LOOM_CONCURRENCY=16 loom run -c config.toml ...
LOOM_BATCH__SIZE=32 loom run -c config.toml ...
LOOM_LAYERS_SCORE_THRESHOLD=0.8 loom run -c config.toml ...
```
