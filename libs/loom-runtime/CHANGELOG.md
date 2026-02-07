# Changelog

All notable changes to `loom-runtime` will be documented in this file.

## [Unreleased]

_No changes yet._

## Completed

- **Context Refactor** - `Context` now holds optional `Arc<Runtime>` for active runtime access; added `emit()` and `data_source()` methods; new `BatchContext` type for batch processing
- **Error Aggregation** - Hierarchical `loom_error::Result<Value>` support in `LayerResult`
- **Result Metadata** - Timing and resource metrics (`elapsed_ms`, `throughput`)
- **Dynamic Layers** - Runner removal, config simplification
