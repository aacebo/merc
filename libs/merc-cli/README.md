# merc-cli

Command-line interface for the Merc scoring engine.

## Installation

```bash
cargo build --package merc-cli
```

## Usage

```bash
merc <command> [options]
```

## Commands

### `bench` - Benchmark Operations

#### `bench run` - Run benchmark against a dataset

```bash
merc bench run <path> [options]

Options:
  -t, --threshold <THRESHOLD>  Base threshold for scoring [default: 0.75]
  -d, --dynamic                Enable dynamic thresholds based on text length
```

Example:
```bash
merc bench run libs/merc-engine/benches/dataset.json
merc bench run libs/merc-engine/benches/dataset.json --threshold 0.80 --dynamic
```

#### `bench validate` - Validate a benchmark dataset

```bash
merc bench validate <path>
```

Checks for:
- Valid JSON structure
- Required fields present
- Valid label names
- Valid decision values (accept/reject)
- No duplicate sample IDs

#### `bench coverage` - Show label coverage for a dataset

```bash
merc bench coverage <path>
```

Displays:
- Total samples and accept/reject breakdown
- Samples per category (target: 50 each)
- Samples per label (target: 3+ each)
- Missing labels (if any)

## Development

Run with cargo:
```bash
cargo run --package merc-cli -- bench run libs/merc-engine/benches/dataset.json
```
