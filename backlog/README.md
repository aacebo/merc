# Loom Backlog

## Status Overview

| Phase | Description | Crate | Status |
|-------|-------------|-------|--------|
| [01-collection-ops](01-collection-ops.md) | flatten, chunk, window | pipe | PENDING |
| [02-time-operators](02-time-operators.md) | timeout, debounce | pipe | PENDING |
| [03-multi-file-merge](03-multi-file-merge.md) | Config file includes/refs | config | PENDING |

## Priority Tiers

### Tier 1: Pipe Operators - Advanced
- **Phase 01**: Collection operators - flatten, chunk, window
- **Phase 02**: Time operators - timeout, debounce

### Tier 2: Config Enhancement
- **Phase 03**: Multi-file config merge

## Dependencies

```
Phase 01 (Collection) ─► Phase 02 (Time)

Phase 03 (Config) - Independent
```

## Completed Work Summary

The following phases have been completed and their documentation archived:

- **Control Flow & Result Ops** - branch, and/or, retry, unwrap/expect operators
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
