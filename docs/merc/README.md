# 6.1 Merc

Documentation for the Merc scoring engine—a write-time memory gating system using zero-shot classification.

<pre>
├── <a href="../README.md">..</a>
├── <span><a href="./README.md"><b>▾ 6.1 Merc/</b></a> 👈</span>
├── <a href="./scoring-algorithm.md">6.1.1 Scoring Algorithm</a>
└── <a href="./roadmap/README.md">▸ 6.1.2 Roadmap/</a>
</pre>

---

## What is Merc?

Merc is a **fast write-time gating filter** for AI memory systems:

- **Accept/reject in <200ms** — Local inference, no LLM API calls
- **Interpretable scores** — Clear 26-label breakdown across 4 categories
- **Zero LLM cost** — Uses zero-shot classification (rust_bert)
- **Pluggable** — Output feeds downstream systems (Zep, Hindsight, etc.)

## What Merc Is NOT

Merc focuses on one thing well—scoring. Other concerns are handled downstream:

| Concern | Where It's Handled |
|---------|-------------------|
| Retrieval | Zep, Hindsight |
| Entity extraction | Downstream LLM |
| Knowledge graph | Downstream storage |
| Contradiction resolution | Downstream temporal systems |
| PII masking | Downstream compliance layer |

---

## Documentation

| Document | Description |
|----------|-------------|
| [Scoring Algorithm](./scoring-algorithm.md) | How Merc scores text for memory worthiness |
| [Improvement Roadmap](./roadmap/) | Phased improvement plan (MERC-001 through MERC-014) |

---

## Design Philosophy

| Principle | Description |
|-----------|-------------|
| **Write-time gating** | Filter at write time, not read time |
| **Stateless** | No memory of previous interactions |
| **Fast** | Local inference, ~50-100ms per score |
| **Composable** | Provides signals for downstream systems |

---

## Research Context

These improvements are informed by analysis of production memory systems:

| System | Key Insight | How Merc Applies |
|--------|-------------|------------------|
| **Zep** | Bi-temporal timestamps | Temporal labels flag time-sensitive content |
| **Zep** | Automatic contradiction detection | `Temporal_Update` label flags changes |
| **Hindsight** | Epistemic networks | Similar to Context/Emotion/Outcome/Sentiment categories |
| **Hindsight** | Opinion confidence tracking | Platt calibration achieves similar goal |
| **Enterprise** | BERT local inference | Already using local zero-shot classification |
| **Enterprise** | Sensitivity tagging | Sensitivity labels (Sensitive, Confidential) |

See [reference docs](../reference/) and [analysis docs](../analysis/) for detailed comparisons.
