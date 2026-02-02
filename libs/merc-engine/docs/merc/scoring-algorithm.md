# Scoring Algorithm

A multi-dimensional text classification system for determining text importance and filtering trivial content.

## Overview

The scoring algorithm evaluates input text across four independent dimensions using zero-shot classification, then aggregates weighted predictions to produce a final importance score. Text that scores below a threshold or is detected as phatic (small talk) is rejected.

## Data Flow

```mermaid
flowchart TD
    A[Input Text] --> B[Zero-Shot Classification<br/>rust_bert pretrained model]

    B --> C[Sentiment]
    B --> D[Emotion]
    B --> E[Outcome]
    B --> F[Context]

    subgraph SentimentLabels [Sentiment Labels]
        C1[Negative<br/>w=0.35]
        C2[Positive<br/>w=0.30]
        C3[Neutral<br/>w=0.10]
    end

    subgraph EmotionLabels [Emotion Labels]
        D1[Stress<br/>w=0.45]
        D2[Fear<br/>w=0.40]
        D3[Anger<br/>w=0.40]
        D4[Sad<br/>w=0.40]
        D5[Shame<br/>w=0.35]
        D6[Pride<br/>w=0.30]
        D7[Joy<br/>w=0.30]
    end

    subgraph OutcomeLabels [Outcome Labels]
        E1[Decision<br/>w=0.80]
        E2[Progress<br/>w=0.65]
        E3[Conflict<br/>w=0.65]
        E4[Success<br/>w=0.55]
        E5[Failure<br/>w=0.55]
        E6[Reward<br/>w=0.45]
        E7[Punishment<br/>w=0.45]
    end

    subgraph ContextLabels [Context Labels]
        F1[Task<br/>w=1.00]
        F2[Plan<br/>w=0.90]
        F3[Goal<br/>w=0.90]
        F4[Preference<br/>w=0.85]
        F5[Fact<br/>w=0.80]
        F6[Entity<br/>w=0.65]
        F7[Time<br/>w=0.55]
        F8[Place<br/>w=0.55]
        F9[Phatic<br/>w=0.40]
    end

    C --> SentimentLabels
    D --> EmotionLabels
    E --> OutcomeLabels
    F --> ContextLabels

    SentimentLabels --> G[Score Aggregation<br/>S = max of category scores]
    EmotionLabels --> G
    OutcomeLabels --> G
    ContextLabels --> G

    G --> H{S >= 0.75<br/>AND<br/>Phatic < 0.80?}

    H -->|Yes| I[ACCEPT]
    H -->|No| J[REJECT]

    style I fill:#22c55e,color:#fff
    style J fill:#ef4444,color:#fff
    style F1 fill:#22c55e,color:#fff
    style F9 fill:#ef4444,color:#fff
```

## Mathematical Definitions

### Label Score

For each label prediction from the model:

```mermaid
flowchart LR
    A[Model Prediction] --> B{c >= t?}
    B -->|Yes| C["S_label = c * w"]
    B -->|No| D["S_label = 0"]
```

Where:
- `c` = model confidence (0.0 to 1.0)
- `w` = label weight (predefined per label)
- `t` = label threshold (predefined per label)

### Category Score

Each category aggregates its top-k label scores:

```mermaid
flowchart TD
    A[All Label Scores] --> B[Sort descending]
    B --> C[Take top k labels<br/>k = min 2, n]
    C --> D["S_category = sum(top k) / k"]
```

Where:
- `k = min(2, n)` where n = number of labels with non-zero scores
- Labels are sorted in descending order by score
- k is at least 1 to avoid division by zero

### Overall Score

The final score is the maximum across all categories:

```mermaid
flowchart LR
    A[S_sentiment] --> E[S_overall = max]
    B[S_emotion] --> E
    C[S_outcome] --> E
    D[S_context] --> E
```

## Label Categories

### Sentiment (3 labels)

| Label    | Weight | Threshold | Hypothesis                                      |
|----------|--------|-----------|------------------------------------------------|
| Negative | 0.35   | 0.70      | "This text expresses a negative sentiment."    |
| Positive | 0.30   | 0.70      | "This text expresses a positive sentiment."    |
| Neutral  | 0.10   | 0.70      | "This text expresses a neutral sentiment."     |

### Emotion (7 labels)

| Label  | Weight | Threshold | Hypothesis                                      |
|--------|--------|-----------|------------------------------------------------|
| Stress | 0.45   | 0.70      | "This text expresses stress or pressure."      |
| Fear   | 0.40   | 0.70      | "This text expresses fear or anxiety."         |
| Anger  | 0.40   | 0.70      | "This text expresses anger or frustration."    |
| Sad    | 0.40   | 0.70      | "This text expresses sadness or grief."        |
| Shame  | 0.35   | 0.70      | "This text expresses shame or embarrassment."  |
| Pride  | 0.30   | 0.70      | "This text expresses pride or accomplishment." |
| Joy    | 0.30   | 0.70      | "This text expresses joy or happiness."        |

### Outcome (7 labels)

| Label      | Weight | Threshold | Hypothesis                                                          |
|------------|--------|-----------|---------------------------------------------------------------------|
| Decision   | 0.80   | 0.70      | "This text describes making a decision or choice."                  |
| Progress   | 0.65   | 0.70      | "This text describes progress, completion, or forward movement."    |
| Conflict   | 0.65   | 0.70      | "This text describes disagreement, conflict, argument, or tension." |
| Success    | 0.55   | 0.70      | "This text describes achieving a goal or success."                  |
| Failure    | 0.55   | 0.70      | "This text describes a failure or setback."                         |
| Reward     | 0.45   | 0.70      | "This text describes receiving a reward or benefit."                |
| Punishment | 0.45   | 0.70      | "This text describes a punishment or consequence."                  |

### Context (9 labels)

| Label      | Weight | Threshold | Hypothesis                                                           |
|------------|--------|-----------|----------------------------------------------------------------------|
| Task       | 1.00   | 0.65      | "This text describes a task, todo item, or reminder."                |
| Plan       | 0.90   | 0.65      | "This text describes a plan, commitment, or intention."              |
| Goal       | 0.90   | 0.65      | "This text describes a goal, objective, or aspiration."              |
| Preference | 0.85   | 0.65      | "This text expresses a preference, like, dislike, or opinion."       |
| Fact       | 0.80   | 0.70      | "This text states a factual piece of information."                   |
| Entity     | 0.65   | 0.75      | "This text mentions a specific named person, organization, or entity."|
| Time       | 0.55   | 0.70      | "This text references a specific time or date."                      |
| Place      | 0.55   | 0.70      | "This text references a specific location or place."                 |
| Phatic     | 0.40   | 0.80      | "This text is a greeting, thanks, farewell, or polite small talk."   |

## Rejection Criteria

Text is rejected (returns Cancel status) if **either** condition is met:

1. **Low Score:** `S_overall < 0.75`
2. **Phatic Detection:** `S_phatic >= 0.80`

The phatic filter ensures greetings and small talk ("hi", "thanks", "bye") are filtered out regardless of other detected signals.

## Weight Design Rationale

The weight hierarchy reflects the system's optimization for capturing actionable information:

```mermaid
flowchart LR
    A["Context<br/>max 1.00"] --> B["Outcome<br/>max 0.80"]
    B --> C["Emotion<br/>max 0.45"]
    C --> D["Sentiment<br/>max 0.35"]

    style A fill:#22c55e,color:#fff
    style B fill:#3b82f6,color:#fff
    style C fill:#f59e0b,color:#fff
    style D fill:#6b7280,color:#fff
```

**Context labels are prioritized** because they capture information most useful for memory/reminder systems:
- Tasks (1.00) and Plans/Goals (0.90) are weighted highest as they represent explicit actionable items
- Preferences (0.85) and Facts (0.80) capture important personal information
- Entity/Time/Place (0.55-0.65) provide supporting context
- Phatic (0.40) is weighted low but has a high threshold (0.80) for rejection

**Outcome labels** capture significant life events and decisions that may be worth remembering.

**Emotion labels** weight negative emotions (stress, fear, anger) slightly higher than positive ones, as distress signals may warrant attention.

**Sentiment labels** have the lowest weights since raw sentiment provides less actionable information than the other dimensions.

## Example Scoring

### Example 1: Accepted

**Input:** "oh my god, I'm going to be late for work!"

```mermaid
flowchart LR
    subgraph Categories
        A["Emotion<br/>Stress → ~0.45"]
        B["Context<br/>Task-like → varies"]
        C["Sentiment<br/>Negative → ~0.35"]
        D["Outcome<br/>none → 0.0"]
    end

    A --> E["max() >= 0.75"]
    B --> E
    C --> E
    D --> E

    E --> F[ACCEPT]
    style F fill:#22c55e,color:#fff
```

### Example 2: Rejected

**Input:** "hi how are you?"

```mermaid
flowchart LR
    subgraph Categories
        A["Context<br/>Phatic → 0.80+"]
        B["Sentiment<br/>Neutral → ~0.10"]
        C["Emotion<br/>none → 0.0"]
        D["Outcome<br/>none → 0.0"]
    end

    A --> E{"Phatic >= 0.80?"}
    E -->|Yes| F[REJECT]

    style F fill:#ef4444,color:#fff
```
