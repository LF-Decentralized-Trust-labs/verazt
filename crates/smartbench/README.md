# SmartBench

Benchmark smart contract analysis tools against the SmartBugs-curated dataset. SmartBench parses ground-truth annotations, runs your analysis tool, and reports accuracy metrics including precision, recall, and F1 score.

## Overview

SmartBench evaluates static analysis tools for Solidity smart contracts by:
- Scanning datasets with ground-truth annotations
- Running your tool (local binary or Docker image) on contract files
- Matching tool findings against ground-truth labels
- Computing accuracy metrics (TP, FP, FN, precision, recall, F1)
- Generating detailed reports in text or JSON format

## Building

Build the smartbench binary from the workspace root:

```bash
cargo build --release -p smartbench
```

The binary will be available at `target/release/smartbench`.

## Usage

SmartBench requires two main inputs:
1. A dataset directory containing annotated Solidity contracts
2. Either a local tool binary (`--tool-path`) or a Docker image (`--docker-image`)

### Basic Usage

**With a local binary:**
```bash
smartbench --tool-path /path/to/your/tool --dataset /path/to/dataset
```

**With a Docker image:**
```bash
smartbench --docker-image your-tool:latest --dataset /path/to/dataset
```

### Advanced Options

**Filter by bug category:**
```bash
smartbench --tool-path ./tool --dataset ./dataset --category reentrancy
```

**Save results to JSON:**
```bash
smartbench --tool-path ./tool --dataset ./dataset --format json --output results.json
```

**Docker mode with resource limits:**
```bash
smartbench \
  --docker-image mythril:latest \
  --dataset ./smartbugs-curated \
  --timeout 60 \
  --mem-limit 4g \
  --cpus 2.0
```

**Verbose output with per-file details:**
```bash
smartbench --tool-path ./tool --dataset ./dataset --verbose
```

## Command-Line Options

### Required (one of):
- `--tool-path <PATH>` - Path to local analysis tool binary
- `--docker-image <IMAGE>` - Docker image for the analysis tool

### Required:
- `--dataset <PATH>` - Path to dataset directory containing annotated contracts

### Optional:
- `--format <FORMAT>` - Output format: `text` (default) or `json`
- `--output <FILE>` - Output file (default: stdout)
- `--category <NAME>` - Filter to specific bug category (e.g., reentrancy, overflow)
- `--timeout <SECS>` - Per-file timeout in seconds (Docker mode only)
- `--mem-limit <LIMIT>` - Memory limit per container, e.g., "4g" (Docker mode only)
- `--cpus <NUM>` - CPU limit per container (Docker mode only)
- `--verbose` - Show per-file details in output

## Dataset Format

SmartBench expects datasets to follow this structure:
```
dataset/
├── contract1.sol          # Solidity source files
├── contract1.sol.json     # Ground-truth annotations
├── contract2.sol
└── contract2.sol.json
```

Annotation files (`.json`) should contain bug markers that identify known vulnerabilities in the corresponding contract.

## Tool Requirements

### Local Binary Mode
Your tool should:
- Accept a Solidity file path as an argument
- Output findings in a parseable format (JSON recommended)
- Exit with status code 0 on success

### Docker Mode
Your Docker image should:
- Accept a mounted Solidity file
- Process the file and output findings
- Support standard Docker resource limits

## Example Output

**Text format (default):**
```
SmartBench Report
=================
Tool: binary: ./smarthunt
Dataset: ./smartbugs-curated

Overall Metrics:
  True Positives:  45
  False Positives: 12
  False Negatives: 8

  Precision: 78.95%
  Recall:    84.91%
  F1 Score:  81.82%

Files with errors: 3
```

**JSON format:**
```json
{
  "overall": {
    "true_positives": 45,
    "false_positives": 12,
    "false_negatives": 8,
    "precision": 0.7895,
    "recall": 0.8491,
    "f1": 0.8182
  },
  "per_category": {...},
  "per_file": [...]
}
```

## Examples

### Benchmark SmartHunt
```bash
cargo run --release -p smartbench -- \
  --tool-path ./target/release/smarthunt \
  --dataset ./smartbugs-curated \
  --verbose
```

### Benchmark Slither (Docker)
```bash
smartbench \
  --docker-image trailofbits/eth-security-toolbox:latest \
  --dataset ./smartbugs-curated \
  --timeout 120 \
  --mem-limit 8g \
  --category reentrancy
```

### Generate JSON Report
```bash
smartbench \
  --tool-path ./smarthunt \
  --dataset ./dataset \
  --format json \
  --output benchmark-results.json
```

## Interpreting Results

- **True Positives (TP)**: Bugs correctly identified by the tool
- **False Positives (FP)**: Tool reported bugs that don't exist (false alarms)
- **False Negatives (FN)**: Real bugs the tool missed
- **Precision**: TP / (TP + FP) - accuracy of positive predictions
- **Recall**: TP / (TP + FN) - coverage of actual bugs
- **F1 Score**: Harmonic mean of precision and recall

Higher precision means fewer false alarms. Higher recall means fewer missed bugs. F1 score balances both metrics.
