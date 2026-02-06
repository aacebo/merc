# Phase 06: Multi-File Config Merge

**Status:** PENDING

**Crate:** loom-config

**Depends on:** (none)

## Goal

Allow complex pipeline configurations to reference each other across multiple
smaller files that get merged at parse time.

## Use Case

Split large configs into modular pieces:

```
config/
  base.yaml         # shared settings
  models/
    bert.yaml       # model-specific config
    gpt.yaml
  pipelines/
    score.yaml      # references models/bert.yaml
```

## Requirements

1. **File references** - `$include` or `$ref` syntax to include other files
2. **Recursive resolution** - Nested includes resolved automatically
3. **Conflict detection** - Error when same keys appear in multiple files
4. **Relative paths** - Paths resolved relative to the including file's location

## Example

**base.yaml:**
```yaml
concurrency: 4
batch_size: 32
```

**score.yaml:**
```yaml
$include: ./base.yaml

layers:
  score:
    threshold: 0.7
    model: bert-base
```

**Merged result:**
```yaml
concurrency: 4
batch_size: 32
layers:
  score:
    threshold: 0.7
    model: bert-base
```

## Implementation TBD

Detailed implementation to be designed when phase begins. Key considerations:

- Syntax choice: `$include`, `$ref`, or YAML anchors
- Deep vs shallow merge behavior
- Circular reference detection
- Error messages with file/line context
