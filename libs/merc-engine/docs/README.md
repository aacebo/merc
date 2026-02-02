# Merc Engine Documentation

Documentation for the Merc scoring engine—a write-time memory gating system using zero-shot classification.

<pre>
├── <a href="./README.md"><b>&lt;&lt;docs&gt;&gt;</b></a> 👈
├── <a href="./merc/">merc/</a> — Scoring algorithm & improvement roadmap
├── <a href="./analysis/">analysis/</a> — Comparative analysis documents
└── <a href="./research/">research/</a> — External system reference docs
</pre>

---

## Directory Overview

| Directory | Purpose |
|-----------|---------|
| [merc/](./merc/) | Core scoring algorithm documentation and phased improvement roadmap |
| [analysis/](./analysis/) | Comparative analysis between Merc and other memory systems |
| [research/](./research/) | Reference documentation for external systems (Zep, Hindsight, Enterprise) |

---

## Quick Links

### Merc Core

- [Scoring Algorithm](./merc/scoring-algorithm.md) — How Merc scores text for memory worthiness
- [Improvement Roadmap](./merc/README.md) — Phased improvement plan (MERC-001 through MERC-014)

### Analysis

- [Memory System Comparison](./analysis/memory-system-comparison.md) — Merc vs Hindsight architectural comparison
- [Merc vs Zep](./analysis/merc-vs-zep-comparison.md) — Comparison with Zep's graph-based memory
- [Merc vs Enterprise](./analysis/merc-vs-enterprise-comparison.md) — Comparison with enterprise memory model

### Research

- [Hindsight Reference](./research/hindsight-reference.md) — Biomimetic memory with epistemic networks
- [Zep Reference](./research/zep-reference.md) — Graph-based memory with bi-temporal model
- [Enterprise Memory Reference](./research/enterprise-memory-reference.md) — Enterprise-scale memory architecture

---

## What is Merc?

Merc is a **write-time memory gating** system that decides whether text is worth storing as a memory. It uses:

- **Zero-shot classification** with BART-large-MNLI for label scoring
- **26 labels** across 4 categories (Sentiment, Emotion, Outcome, Context)
- **Weighted scoring** to prioritize memory-bearing content
- **Negative filters** to reject small talk and filler

### What Merc Does

- Scores text for memory worthiness (accept/reject)
- Provides label breakdown and confidence scores
- Filters out noise (greetings, acknowledgments, filler)

### What Merc Doesn't Do

- Store memories (downstream systems handle storage)
- Retrieve memories (stateless scoring only)
- Parse temporal information (flags for downstream systems)
- Handle contradictions (flags for downstream systems)

---

## Design Philosophy

| Principle | Description |
|-----------|-------------|
| **Write-time gating** | Filter at write time, not read time |
| **Stateless** | No memory of previous interactions |
| **Fast** | Local inference, ~50-100ms per score |
| **Composable** | Provides signals for downstream systems |
