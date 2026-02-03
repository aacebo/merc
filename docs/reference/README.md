# 6.2 Reference

Research documentation on external memory systems that inform Merc's design.

<pre>
├── <a href="../README.md">..</a>
├── <a href="../1.memory.md">▸ 1. Memory</a>
├── <a href="../2.ingestion.md">▸ 2. Ingestion</a>
├── <a href="../3.guards.md">▸ 3. Guards</a>
├── <a href="../4.recall.md">▸ 4. Recall</a>
├── <a href="../5.classification.md">▸ 5. Classification</a>
└── <a href="../README.md">▾ 6. Research/</a>
    ├── <a href="../merc/README.md">▸ 6.1 Merc/</a>
    ├── <span><a href="./README.md"><b>▾ 6.2 Reference/</b></a> 👈</span>
    │   ├── <a href="./1.hindsight.md">6.2.1 Hindsight</a> — Biomimetic agent memory
    │   ├── <a href="./2.zep.md">6.2.2 Zep</a> — Knowledge graph memory
    │   └── <a href="./3.enterprise.md">6.2.3 Enterprise</a> — Enterprise memory patterns
    └── <a href="../analysis/README.md">▸ 6.3 Analysis/</a>
</pre>

---

## Overview

These documents analyze external memory systems to understand different approaches to agent memory:

| System | Approach | Key Concepts |
|--------|----------|--------------|
| **Hindsight** | Epistemic networks | TEMPR/CARA, World/Experience/Opinion/Observation |
| **Zep** | Knowledge graph | Temporal awareness, entity extraction, bi-temporal storage |
| **Enterprise** | Compliance-focused | Audit trails, PII handling, retention policies |

---

## Why Study These Systems?

Understanding external systems helps:
1. **Identify patterns** — Common approaches to memory challenges
2. **Avoid pitfalls** — Learn from others' design decisions
3. **Find gaps** — Opportunities for Merc to differentiate
4. **Inform design** — Borrow proven concepts where appropriate

See [analysis/](../analysis/) for direct comparisons with Merc.
