# Hindsight Reference

A biomimetic agent memory system that organizes memories into epistemically-distinct networks and retrieves them using multi-pathway fusion.

> **Note:** This is a reference document for research purposes. Hindsight is an external system, not part of Merc.

## Overview

Hindsight structures agent memory like human cognition—separating facts from experiences, observations from opinions. The system uses the TEMPR (Temporal Entity Memory Priming Retrieval) architecture for storage and retrieval, and CARA (Coherent Adaptive Reasoning Agents) for preference-conditioned reasoning.

## Architecture

```mermaid
flowchart TD
    A[Agent Interaction] --> B[Hindsight API]

    B --> C{Operation}

    C -->|Retain| D[TEMPR: Store]
    C -->|Recall| E[TEMPR: Retrieve]
    C -->|Reflect| F[CARA: Reason]

    subgraph MemoryNetworks [Memory Networks]
        W[World<br/>Objective Facts]
        X[Experience<br/>Agent Actions]
        O[Opinion<br/>Beliefs + Confidence]
        S[Observation<br/>Entity Summaries]
    end

    D --> MemoryNetworks
    E --> MemoryNetworks
    F --> MemoryNetworks

    E --> G[Reciprocal Rank Fusion]
    G --> H[Cross-Encoder Reranking]
    H --> I[Token-Limited Output]

    style W fill:#3b82f6,color:#fff
    style X fill:#22c55e,color:#fff
    style O fill:#f59e0b,color:#fff
    style S fill:#8b5cf6,color:#fff
```

## Memory Networks

Hindsight organizes memories into four epistemically-distinct networks:

```mermaid
flowchart LR
    subgraph World ["World (W)"]
        W1[Objective Facts]
        W2[Third-Person]
        W3[Environmental]
    end

    subgraph Experience ["Experience (B)"]
        X1[Agent Actions]
        X2[First-Person]
        X3[Recommendations]
    end

    subgraph Opinion ["Opinion (O)"]
        O1[Agent Beliefs]
        O2[Confidence c in 0,1]
        O3[Timestamps]
    end

    subgraph Observation ["Observation (S)"]
        S1[Entity Summaries]
        S2[Synthesized]
        S3[Preference-Neutral]
    end

    style World fill:#3b82f6,color:#fff
    style Experience fill:#22c55e,color:#fff
    style Opinion fill:#f59e0b,color:#fff
    style Observation fill:#8b5cf6,color:#fff
```

| Network | Symbol | Description | Example |
|---------|--------|-------------|---------|
| World | W | Objective, third-person environmental facts | "The stove gets hot when turned on" |
| Experience | B | First-person agent actions and outcomes | "I recommended the Italian restaurant and they loved it" |
| Opinion | O | Agent beliefs with confidence scores | "User prefers morning meetings (c=0.85)" |
| Observation | S | Synthesized entity profiles | "John: Software engineer, likes coffee, works remotely" |

## Core Operations

### Retain

Converts interactions into structured, time-aware memories.

```mermaid
flowchart TD
    A[Raw Transcript] --> B[LLM Extraction]
    B --> C[Narrative Facts<br/>2-5 per turn]

    C --> D{Classify}

    D --> E[World Facts]
    D --> F[Experience Facts]
    D --> G[Entities]
    D --> H[Temporal Metadata]

    E --> I[Memory Graph]
    F --> I
    G --> I
    H --> I

    I --> J[Embedding Generation]
    J --> K[Index Updates]
```

**Fact Representation:**

Each fact `f` contains:
- `u`: Unique identifier
- `b`: Bank identifier
- `t`: Narrative text
- `v ∈ ℝᵈ`: Embedding vector (d=384 default)
- `τ`: Temporal metadata (start/end timestamps)
- Entity links and type classifications

### Recall

Retrieves relevant memories using parallel multi-pathway search.

```mermaid
flowchart TD
    A[Query] --> B[TEMPR Retrieval]

    subgraph Channels [Parallel Channels]
        C1[Semantic<br/>Cosine Similarity]
        C2[Lexical<br/>BM25]
        C3[Graph<br/>Entity Links]
        C4[Temporal<br/>Time Windows]
    end

    B --> C1
    B --> C2
    B --> C3
    B --> C4

    C1 --> D[Reciprocal Rank Fusion]
    C2 --> D
    C3 --> D
    C4 --> D

    D --> E[Cross-Encoder Reranking]
    E --> F[Token Budget Trim]
    F --> G[Context Output]

    style C1 fill:#3b82f6,color:#fff
    style C2 fill:#22c55e,color:#fff
    style C3 fill:#f59e0b,color:#fff
    style C4 fill:#8b5cf6,color:#fff
```

### Reflect

Performs deeper analysis and belief updating.

```mermaid
flowchart TD
    A[Reflection Query] --> B[Recall Relevant Memories]
    B --> C[CARA Reasoning]

    C --> D{Evidence Type}

    D -->|Supporting| E[Increase Confidence]
    D -->|Weak| F[Decrease Confidence]
    D -->|Contradicting| G[Reduce Both<br/>c and Content]

    E --> H[Update Opinion Network]
    F --> H
    G --> H

    H --> I[Generate Response]
```

## TEMPR System

**T**emporal **E**ntity **M**emory **P**riming **R**etrieval

### Retrieval Channels

```mermaid
flowchart LR
    subgraph Semantic [Semantic Channel]
        S1[Query Embedding]
        S2[Cosine Similarity]
        S3[Vector Index]
    end

    subgraph Lexical [Lexical Channel]
        L1[Query Tokens]
        L2[BM25 Scoring]
        L3[GIN Index]
    end

    subgraph Graph [Graph Channel]
        G1[Seed Entities]
        G2[Spreading Activation]
        G3[Multi-hop Traversal]
    end

    subgraph Temporal [Temporal Channel]
        T1[Time Expression]
        T2[Hybrid Parsing]
        T3[Window Filter]
    end

    style Semantic fill:#3b82f6,color:#fff
    style Lexical fill:#22c55e,color:#fff
    style Graph fill:#f59e0b,color:#fff
    style Temporal fill:#8b5cf6,color:#fff
```

| Channel | Method | Index Type | Use Case |
|---------|--------|------------|----------|
| Semantic | Cosine similarity on embeddings | Vector (pgvector) | Conceptual similarity, paraphrasing |
| Lexical | BM25 ranking | GIN full-text | Names, technical terms, exact matches |
| Graph | Spreading activation | Entity links | Related entities, indirect connections |
| Temporal | Window filtering | B-tree timestamps | "Last spring", "yesterday", time ranges |

### Memory Graph

The memory graph `G = (V, E)` connects facts through multiple edge types:

```mermaid
flowchart TD
    subgraph Vertices [Vertices V]
        F1[Fact 1]
        F2[Fact 2]
        F3[Fact 3]
        E1[Entity: John]
    end

    F1 -->|temporal| F2
    F1 -->|semantic| F3
    F1 -->|entity| E1
    F2 -->|entity| E1
    F2 -->|causal| F3
```

| Edge Type | Connection Criteria | Weight Function |
|-----------|---------------------|-----------------|
| Temporal | Close-in-time pairs | Time decay |
| Semantic | cosine(v1, v2) > threshold | Similarity score |
| Entity | Shared entity reference | 1.0 (binary) |
| Causal | Extracted cause-effect | LLM confidence |

## Scoring Pipeline

```mermaid
flowchart TD
    subgraph Stage1 [Stage 1: Channel Scoring]
        A1[Semantic: cosine]
        A2[Lexical: BM25]
        A3[Graph: activation]
        A4[Temporal: decay]
    end

    subgraph Stage2 [Stage 2: Rank Fusion]
        B1["RRF: sum 1/(k + rank)"]
    end

    subgraph Stage3 [Stage 3: Neural Rerank]
        C1[Cross-Encoder]
    end

    subgraph Stage4 [Stage 4: Output]
        D1[Token-Limited Context]
    end

    Stage1 --> Stage2
    Stage2 --> Stage3
    Stage3 --> Stage4

    style Stage1 fill:#3b82f6,color:#fff
    style Stage2 fill:#22c55e,color:#fff
    style Stage3 fill:#f59e0b,color:#fff
    style Stage4 fill:#8b5cf6,color:#fff
```

### Channel Scoring

#### Semantic Scoring

Measures conceptual similarity using embedding vectors.

```mermaid
flowchart LR
    A[Query] --> B[Embed: v_q]
    C[Memory] --> D[Embed: v_m]
    B --> E["score = cos(v_q, v_m)"]
    D --> E
    E --> F["score in -1, 1"]
```

**Formula:**
```
semantic_score(q, m) = (v_q · v_m) / (||v_q|| × ||v_m||)
```

Where:
- `v_q` = query embedding vector (ℝ³⁸⁴)
- `v_m` = memory embedding vector (ℝ³⁸⁴)

#### Lexical Scoring (BM25)

Measures term-based relevance using Okapi BM25.

```mermaid
flowchart LR
    A[Query Terms] --> B[Term Frequency]
    C[Memory Text] --> D[Document Frequency]
    B --> E[BM25 Score]
    D --> E
    E --> F["score >= 0"]
```

**Formula:**
```
BM25(q, m) = Σ IDF(t) × (f(t,m) × (k₁ + 1)) / (f(t,m) + k₁ × (1 - b + b × |m|/avgdl))
```

Where:
- `t` = query term
- `f(t,m)` = term frequency in memory m
- `|m|` = memory length
- `avgdl` = average document length
- `k₁ = 1.2`, `b = 0.75` (standard parameters)

#### Graph Scoring

Spreading activation across memory graph edges.

```mermaid
flowchart TD
    A[Seed Entities] --> B[Initial Activation = 1.0]
    B --> C{Edge Type}

    C -->|Temporal| D["score * decay"]
    C -->|Semantic| E["score * similarity"]
    C -->|Entity| F["score * 1.0"]
    C -->|Causal| G["score * confidence"]

    D --> H[Propagate to Neighbors]
    E --> H
    F --> H
    G --> H

    H --> I{Iterations < max?}
    I -->|Yes| C
    I -->|No| J[Final Activation Scores]
```

**Activation Decay:**
```
activation(n, t+1) = Σ activation(m, t) × edge_weight(m, n) × damping
```

Where `damping = 0.85` (prevents infinite propagation)

#### Temporal Scoring

Filters and scores by time relevance.

```mermaid
flowchart LR
    A[Query Time Expression] --> B[Parse to Window]
    B --> C["[start, end]"]
    C --> D{Memory timestamp in window?}
    D -->|Yes| E[score = recency_decay]
    D -->|No| F[score = 0]
```

**Recency Decay:**
```
temporal_score(m) = e^(-λ × age(m))
```

Where:
- `λ` = decay rate
- `age(m)` = time since memory creation

### Reciprocal Rank Fusion (RRF)

Merges results from multiple retrieval channels without score normalization:

```mermaid
flowchart LR
    subgraph Inputs [Channel Rankings]
        R1["Semantic<br/>rank(d)"]
        R2["Lexical<br/>rank(d)"]
        R3["Graph<br/>rank(d)"]
        R4["Temporal<br/>rank(d)"]
    end

    R1 --> F["RRF(d) = sum of 1/(k + rank)"]
    R2 --> F
    R3 --> F
    R4 --> F

    F --> O[Fused Ranking]
```

**Formula:**
```
RRF(d) = Σᵢ 1 / (k + rankᵢ(d))
```

Where:
- `d` = document/fact
- `k = 60` (regularization constant)
- `rankᵢ(d)` = position of d in channel i's results

**Example Calculation:**

For memory `m₁` with ranks: Semantic=2, Lexical=1, Graph=2, Temporal=3

```
RRF(m₁) = 1/(60+2) + 1/(60+1) + 1/(60+2) + 1/(60+3)
        = 1/62 + 1/61 + 1/62 + 1/63
        = 0.0161 + 0.0164 + 0.0161 + 0.0159
        = 0.0645
```

**Why k = 60?**
- Balances weight between top-ranked and lower-ranked documents
- Prevents single-channel dominance
- Industry-standard value for production systems

| k Value | Behavior | Use Case |
|---------|----------|----------|
| Small (1-10) | Top-1 dominates | High precision needed |
| Medium (60) | Balanced | General retrieval |
| Large (100+) | Flattened ranks | Diversity prioritized |

### Cross-Encoder Reranking

After RRF fusion, a neural reranker refines the ordering:

```mermaid
flowchart LR
    A[RRF Candidates<br/>max 300] --> B[Cross-Encoder<br/>ms-marco-MiniLM]
    B --> C[Reranked Results]
    C --> D[Token Budget Trim]
    D --> E[Final Context]
```

**Scoring:**
```
rerank_score(q, m) = sigmoid(model([CLS] q [SEP] m [SEP]))
```

Output: `score ∈ [0, 1]` representing relevance probability

## CARA System

**C**oherent **A**daptive **R**easoning **A**gents

### Behavioral Profiles

Each agent has a profile `Θ = (S, L, E, β)`:

```mermaid
flowchart TD
    subgraph Profile ["Behavioral Profile Θ"]
        S["S: Skepticism<br/>Cautious ↔ Trusting"]
        L["L: Literalism<br/>Strict ↔ Interpretive"]
        E["E: Empathy<br/>Feeling-aware ↔ Neutral"]
        B["β: Bias Strength<br/>Profile influence intensity"]
    end

    Profile --> R[Preference-Conditioned<br/>Reasoning]
    R --> O[Response Generation]
```

| Parameter | Low Value | High Value |
|-----------|-----------|------------|
| S (Skepticism) | Trusting, accepts claims | Cautious, requires evidence |
| L (Literalism) | Interpretive, infers intent | Strict, follows exactly |
| E (Empathy) | Neutral, task-focused | Feeling-aware, emotional |
| β (Bias) | Weak profile influence | Strong profile influence |

### Opinion Confidence Updates

Opinions maintain a confidence score `c ∈ [0, 1]` that updates with new evidence:

```mermaid
flowchart TD
    A[Opinion: c = 0.7] --> B[New Evidence]

    B --> C{Evidence Type}

    C -->|Strong Support| D["c' = 0.76"]
    C -->|Weak Support| E["c' = 0.715"]
    C -->|Neutral| F["c' = 0.7"]
    C -->|Contradiction| G["c' = 0.35"]

    style D fill:#22c55e,color:#fff
    style E fill:#3b82f6,color:#fff
    style F fill:#6b7280,color:#fff
    style G fill:#ef4444,color:#fff
```

**Update Rules:**

| Evidence | Formula | Effect |
|----------|---------|--------|
| Strong support | `c' = c + α(1 - c)` | Approaches 1.0 |
| Weak support | `c' = c + α'(1 - c)` | Slow increase |
| Contradiction | `c' = c × γ` | Multiplicative decay |
| Strong contradiction | Content also reduced | Opinion weakened |

Where:
- `α` = learning rate (strong) ≈ 0.2
- `α'` = learning rate (weak) ≈ 0.05
- `γ` = contradiction decay ≈ 0.5

## Retrieval Priority

During reflection, memory tiers are prioritized:

```mermaid
flowchart LR
    A["Mental Models<br/>(User-curated)"] --> B["Observations<br/>(Synthesized)"]
    B --> C["Raw Facts<br/>(World + Experience)"]

    style A fill:#22c55e,color:#fff
    style B fill:#3b82f6,color:#fff
    style C fill:#6b7280,color:#fff
```

## Configuration Defaults

| Parameter | Default Value | Description |
|-----------|---------------|-------------|
| Embedding Model | BAAI/bge-small-en-v1.5 | Local embedding model |
| Embedding Dimensions | 384 | Vector size |
| Reranker | cross-encoder/ms-marco-MiniLM-L-6-v2 | Neural reranking model |
| Max Reranker Candidates | 300 | Pre-filter limit |
| RRF Constant k | 60 | Rank fusion regularization |
| Graph Retriever | link_expansion | Traversal algorithm |
| MPFP Top-K Neighbors | 20 | Graph expansion limit |
| Recall Connection Budget | 4 | Max concurrent retrievals |
| Recall Max Concurrent | 32 | Global concurrency limit |
| Retain Chunk Size | 3000 chars | Text chunking |
| Reflect Max Iterations | 10 | Tool call limit |

| Operation | Latency |
|-----------|---------|
| Write-time (LLM extraction) | ~500-2000ms |
| Read-time (retrieval) | ~100-500ms |

## Benchmark Claims

> **Caveat:** These scores are self-reported by Hindsight's creators and have not been independently peer-reviewed. The benchmark methodology was also designed by the same team. Treat these numbers as claims, not verified facts.

| Benchmark | Claimed Score | Comparison |
|-----------|---------------|------------|
| LongMemEval | 91.4% | vs. 39% baseline (self-reported) |
| LoCoMo | 89.61% | vs. GPT-4o full-context (self-reported) |

## Sources

- [GitHub Repository](https://github.com/vectorize-io/hindsight)
- [Research Paper (arXiv:2512.12818)](https://arxiv.org/abs/2512.12818)
- [Official Documentation](https://hindsight.vectorize.io/)
