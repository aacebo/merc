# Loom Architecture & Scaling Backlog

## Pipeline Architecture Overview

```
┌─────────────────────────────────────┐
│   HTTP API Layer (Actix-web)        │ bins/api
├─────────────────────────────────────┤
│   Event/Message Queue (RabbitMQ)    │ crates/events
├─────────────────────────────────────┤
│   Worker/Job Processing             │ bins/worker
├─────────────────────────────────────┤
│   Runtime Orchestration             │ loom-runtime
├─────────────────────────────────────┤
│   Pipeline (Lazy, Type-Erased)      │ loom-pipe
├─────────────────────────────────────┤
│   ML Scoring & Traits               │ loom-cortex
├─────────────────────────────────────┤
│   Task Execution Abstraction        │ loom-sync
├─────────────────────────────────────┤
│   Storage (PostgreSQL + sqlx)       │ crates/storage
├─────────────────────────────────────┤
│   Codecs & I/O Abstraction          │ loom-codec, loom-io
└─────────────────────────────────────┘
```

## Layer Descriptions

### 1. Runtime Layer (`loom-runtime`)
- Central orchestration point
- Manages data sources (FileSystem, etc.)
- Handles codecs (JSON, YAML, TOML)
- Provides pipeline builder factory

### 2. Pipeline/Processing Layer (`loom-pipe`)
- Lazy/pull-based pipeline system
- Type-erased stage execution
- Operators: Spawn, Parallel, FanOut, Router

### 3. ML/Scoring Layer (`loom-cortex`)
- Traits: `Scorer` (sync interface) and `ScorerOutput`
- Platt calibration for probability calibration
- Machine learning model implementations

### 4. Event/Message Layer (`crates/events`)
- RabbitMQ-based async event queue
- Producer/Consumer pattern
- Topic-based routing via exchange bindings

### 5. Worker Layer (`bins/worker`)
- Consumes events from RabbitMQ
- Processes long-running work asynchronously
- Configurable via environment variables

### 6. Storage Layer (`crates/storage`)
- PostgreSQL with sqlx
- Migrations-based schema management

---

## Horizontal Scaling Assessment

### Current Readiness

| Component | Status | Notes |
|-----------|--------|-------|
| API Server | Ready | Actix multi-worker, stateless |
| Message Queue | Ready | RabbitMQ topic routing supports N workers |
| Worker Process | Partial | Stub implementation, doesn't process events yet |
| ML Inference | Bottleneck | Sync, sequential, no batching |
| Database | Tight | Pool max=5, needs tuning for N workers |

### Target Topology

```
Load Balancer
    ↓
[API Server 1] ────┐
[API Server 2] ──→ RabbitMQ ← [Worker 1]
[API Server 3] ────┤  (amqp)    [Worker 2]
                   ↓            [Worker N]
                PostgreSQL
```

---

## Critical Bottlenecks

### 1. Synchronous Scoring Pipeline
**Severity: MAJOR**
- Benchmark runner loops sequentially through samples
- Each sample calls `scorer.score()` (blocking ML inference)
- No parallelization of inference across samples
- No batch processing support

**Location:** `libs/loom-runtime/src/bench/runner.rs`

### 2. Scorer Trait is Synchronous
```rust
pub trait Scorer {
    fn score(&self, text: &str) -> Result<Self::Output, Self::Error>;
}
```
- No `async fn score()` available
- Forces blocking waits during evaluation

### 3. RabbitMQ Worker is Incomplete
- Consumer dequeue loop exists but discards events
- No actual event processing implemented
- One event processed at a time

**Location:** `bins/worker/src/main.rs`

### 4. No Distributed State Management
- No distributed caching for model results
- No request deduplication
- No batch size optimization for ML inference

### 5. Database Pool is Tight
- Only 5 connections max
- Would need tuning for N workers

---

## Backlog Items

### High Priority

- [ ] **Make Scorer trait async** - Enable non-blocking inference coordination
- [ ] **Complete worker implementation** - Actually process queued jobs
- [ ] **Add batch inference support** - Accumulate requests, process in parallel
- [ ] **Increase database connection pool** - Dynamic based on worker count

### Medium Priority

- [ ] **Add model caching/sharing** - Redis or shared model server across workers
- [ ] **Implement request deduplication** - In API layer
- [ ] **Add distributed tracing** - OpenTelemetry for observability
- [ ] **Add metrics collection** - API/Worker/Queue depth monitoring

### Low Priority

- [ ] **Pipeline DAG execution** - Type-erased system could support parallel stage execution
- [ ] **Model loading optimization** - Shared model server to avoid per-worker loading
- [ ] **Rate limiting / backpressure** - Prevent worker overload

---

## Key Insights

1. **Scoring is the bottleneck** - Not the architecture. ML inference (rust-bert) is CPU-bound.

2. **Architecture is ready for horizontal scaling** - But implementation is incomplete.

3. **Pipeline is clever** - Type-erased, lazy evaluation, composable operators.

4. **Event-driven foundation is solid** - RabbitMQ integration correct for multi-worker pattern.

5. **Single-machine constraint** - Benchmark runner has no distributed execution.

---

## File References

| Layer | Location |
|-------|----------|
| Pipeline | `libs/loom-pipe/` |
| Runtime | `libs/loom-runtime/` |
| Events | `crates/events/` |
| API | `bins/api/` |
| Worker | `bins/worker/` |
| Task Sync | `libs/loom-sync/` |
| ML Scoring | `libs/loom-cortex/` |
