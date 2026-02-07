# Changelog

All notable changes to `loom-pipe` will be documented in this file.

## [Unreleased]

- **Sequence Operators** - `.flatten()`, `.flat_map()`, `.chunk()`, `.window()`, `.concat()`
- **Branch Operator** - `.branch().when().then().or_else()` conditional branching with builder pattern
- **Logical Operators** - `.and()`, `.or()`, `.or_else_map()` for Result short-circuiting
- **Retry Operator** - `.retry().attempts().delay().backoff().run()` with exponential backoff
- **Result Operators** - `.unwrap()`, `.expect()`, `.unwrap_or()`, `.unwrap_or_else()`, `.ok()`
- **Option Operators** - `.unwrap()`, `.expect()`, `.unwrap_or()`, `.unwrap_or_else()`, `.ok_or()`

## Completed

- **Pipeline Rewrite** - Pipeline infrastructure with Layer trait
- **Fork/Join** - Renamed spawn to fork, added `.join()` operator
