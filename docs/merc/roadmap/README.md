# 6.1.2 Roadmap

Roadmap for improving Merc's write-time scoring accuracy while maintaining <200ms latency.

<pre>
├── <a href="../README.md">..</a>
├── <span><a href="./README.md"><b>▾ 6.1.2 Roadmap/</b></a> 👈</span>
├── <a href="./1.foundation.md">6.1.2.1 Foundation</a>
├── <a href="./2.labels.md">6.1.2.2 Label Expansion</a>
├── <a href="./3.context.md">6.1.2.3 Context & Ensemble</a>
├── <a href="./4.learning.md">6.1.2.4 Learning Infrastructure</a>
└── <a href="./5.output.md">6.1.2.5 Output Enrichment</a>
</pre>

---

## Phase Overview

| Phase | File | Latency Impact | Expected Gain | Status |
|-------|------|----------------|---------------|--------|
| 1. Foundation | [1.foundation.md](./1.foundation.md) | 0% | 20-35% | Partial |
| 2. Label Expansion | [2.labels.md](./2.labels.md) | +10-15% | 30-45% | Not Started |
| 3. Context & Ensemble | [3.context.md](./3.context.md) | +50-100% | 25-45% | Not Started |
| 4. Learning | [4.learning.md](./4.learning.md) | 0% runtime | Continuous | Partial |
| 5. Output | [5.output.md](./5.output.md) | 0% | Downstream compat | Not Started |

---

## Label Summary

| Category | Current | Proposed | Change |
|----------|---------|----------|--------|
| Sentiment | 3 | 3 | — |
| Emotion | 7 | 8 | +1 (Concern) |
| Outcome | 7 | 8 | +1 (Commitment) |
| Context | 9 | 17 | +8 |
| Negative | 1 | 4 | +3 |
| Temporal | 0 | 3 | +3 (new category) |
| **Total** | **27** | **43** | **+16** |

---

## Success Metrics

| Metric | Current | Phase 1 | Phase 2 | Phase 3 |
|--------|---------|---------|---------|---------|
| Accuracy | Baseline | +20% | +35% | +50% |
| False Positives | Baseline | -15% | -30% | -40% |
| Noise Stored | Baseline | — | -25% | -30% |
| Latency | ~50ms | ~50ms | ~60ms | ~100-150ms |

---

## Quick Links

- [Current Scoring Algorithm](../scoring-algorithm.md) — How scoring works today
- [Phase 1: Foundation](./1.foundation.md) — Quick wins with zero latency cost
- [Phase 2: Labels](./2.labels.md) — Expanded label taxonomy
- [Phase 3: Context](./3.context.md) — Context window and ensemble
- [Phase 4: Learning](./4.learning.md) — Feedback and tuning infrastructure
- [Phase 5: Output](./5.output.md) — Structured output for downstream systems
