# Plan: Test Verazt Analyzer on SmartBugs-Curated Dataset

## 1. Goal

Create a `benchmark` crate (CLI binary) that:

1. Parses ground-truth bug labels from each `.sol` file in the dataset.
2. Runs the Verazt analyzer on each file and collects detected bugs.
3. Matches detected bugs against expected bugs.
4. Reports **True Positives (TP)**, **False Positives (FP)**, and **False Negatives (FN)** per file, per category, and overall.

Usage:
```bash
cargo run -p benchmark -- --dataset solidity/smartbugs-curated
```

---

## 2. Existing Code to Reuse

### 2.1 Annotation Parser (`crates/bugs/src/datasets/smartbugs.rs`) — REUSE AS-IS

Already fully implemented:
- `parse_annotations(file_path) -> Vec<AnnotatedBug>` — parses `// <yes> <report> BUG_NAME`
- `scan_dataset(dataset_dir) -> Vec<AnnotatedBug>` — recursive scan
- `collect_sol_files(dataset_dir) -> Vec<PathBuf>` — file collection
- `AnnotatedBug { file_path, line_number, bug_line, bug_name, category }`

### 2.2 Bug Categories (`crates/bugs/src/bug.rs`) — REUSE AS-IS

- `BugCategory` enum maps 1:1 with SmartBugs labels
- `BugCategory::from_annotation("REENTRANCY")` → `Some(BugCategory::Reentrancy)`
- Each detector has `bug_category() -> BugCategory` for matching

### 2.3 Existing Benchmark Test (`crates/analyzer/tests/benchmark.rs`) — REUSE & MIGRATE

Already has the matching logic we need:
- `DetectedBug` struct: `{ name, category, start_line, severity }`
- `MatchResult` struct: `{ true_positives, false_positives, false_negatives }`
- `MatchedBug` struct: `{ annotation, detection }`
- `match_file()` function: matches detections to annotations by category + exact line

**Issues to fix during migration:**
- `run_analyze_on_file()` is a **stub** — creates an empty `AnalysisContext` without actually compiling the `.sol` file
- Dataset paths point to wrong location (`../../benchmarks/smartbugs-curated/dataset/` instead of `datasets/solidity/smartbugs-curated/`)

### 2.4 Analyzer Pipeline (`crates/analyzer/`) — REUSE AS-IS

- `parse_input_file()` — Solidity compilation
- `AnalysisContext::new()` — context creation
- `PipelineEngine::run()` → `PipelineResult { bugs: Vec<Bug> }`
- All 22 detectors registered via `register_all_detectors()`

---

## 3. Dataset Info

- **Location**: `datasets/solidity/smartbugs-curated/`
- **143 `.sol` files** across **10 categories**:
  `access_control`, `arithmetic`, `bad_randomness`, `denial_of_service`, `front_running`, `other`, `reentrancy`, `short_addresses`, `time_manipulation`, `unchecked_low_level_calls`
- **206 total ground-truth annotations**

### Annotation Format

```solidity
// <yes> <report> BUG_NAME
vulnerable_code_line;
```

### Detector Coverage

| SmartBugs Category      | Matching Detectors                                             | Covered? |
|-------------------------|---------------------------------------------------------------|----------|
| `REENTRANCY`            | `ReentrancySirDetector`, `AIRReentrancyDetector`, `CeiViolationSirDetector` | ✅ |
| `ARITHMETIC`            | `SirUncheckedArithmeticDetector`, `AIRArithmeticDetector`     | ✅ |
| `ACCESS_CONTROL`        | `MissingAccessControlSirDetector`, `VisibilitySirDetector`, `DelegatecallSirDetector`, `TxOriginSirDetector` | ✅ |
| `UNCHECKED_LL_CALLS`    | `UncheckedCallSirDetector`, `LowLevelCallSirDetector`         | ✅ |
| `TIME_MANIPULATION`     | `TimestampDependenceSirDetector`                               | ✅ |
| `OTHER`                 | `UninitializedSirDetector`, `ShadowingSirDetector`            | partial |
| `DENIAL_OF_SERVICE`     | —                                                              | ❌ |
| `BAD_RANDOMNESS`        | —                                                              | ❌ |
| `FRONT_RUNNING`         | —                                                              | ❌ |
| `SHORT_ADDRESSES`       | —                                                              | ❌ |

---

## 4. Implementation Plan

### 4.1 Create `crates/benchmark/` crate

#### `crates/benchmark/Cargo.toml`

```toml
[package]
name = "benchmark"
version.workspace = true
edition.workspace = true

[[bin]]
name = "benchmark"
path = "src/main.rs"

[dependencies]
analyzer = { workspace = true }
bugs = { workspace = true }
frontend = { workspace = true }
clap = { workspace = true }
```

#### Add to workspace `Cargo.toml`

```toml
members = [
  ...,
  "crates/benchmark",
]
```

### 4.2 Source Files

#### `crates/benchmark/src/main.rs` — CLI entry point

```rust
/// CLI usage:
///   cargo run -p benchmark -- --dataset solidity/smartbugs-curated
///   cargo run -p benchmark -- --dataset solidity/smartbugs-curated --category reentrancy
///   cargo run -p benchmark -- --dataset solidity/smartbugs-curated --verbose

#[derive(Parser)]
struct Args {
    /// Path to dataset relative to the datasets/ folder
    /// (e.g. "solidity/smartbugs-curated")
    #[arg(long)]
    dataset: String,

    /// Filter to a specific category subfolder
    #[arg(long)]
    category: Option<String>,

    /// Show per-file details
    #[arg(long)]
    verbose: bool,

    /// Solc version override
    #[arg(long, default_value = "0.4.26")]
    solc_version: String,
}
```

#### `crates/benchmark/src/evaluate.rs` — Matching & evaluation logic

**Migrate and fix** the matching logic from `crates/analyzer/tests/benchmark.rs`:

- `DetectedBug`, `MatchResult`, `MatchedBug` — reuse structs
- `match_file()` — reuse, but also add a **line tolerance** parameter (default ±3 lines) for fuzzy matching as a fallback
- `run_analyze_on_file()` — **fix the stub**: actually compile the `.sol` file with `parse_input_file()`, build `AnalysisContext` with the parsed source units, then run `PipelineEngine::run()`

```rust
fn run_analyze_on_file(file_path: &Path, solc_version: &str) -> Vec<DetectedBug> {
    // 1. Compile .sol file
    let source_units = parse_input_file(file_path_str, None, &[], Some(solc_version));

    // 2. Build analysis context (currently SIR-based, so we convert)
    let context = AnalysisContext::new(ir_units, AnalysisConfig::default());

    // 3. Run pipeline
    let result = PipelineEngine::new(PipelineConfig::default()).run(&mut context);

    // 4. Convert Bug -> DetectedBug
    result.bugs.iter().map(|b| DetectedBug { ... }).collect()
}
```

#### `crates/benchmark/src/report.rs` — Report formatting

Use `common::utils::print_header` and `common::utils::print_subheader` for section headers.

Print results as a structured list:

```
===========================================================================
*** SmartBugs-Curated Evaluation Report ***
===========================================================================

Files: 143 total, 140 compiled, 3 skipped

---------------------------------------------------
*** Per-Category Results ***
---------------------------------------------------

- Reentrancy (31 files, 42 expected)
  TP: ..  FP: ..  FN: ..

- Arithmetic (16 files, 18 expected)
  TP: ..  FP: ..  FN: ..

- Access Control (16 files, 28 expected)
  TP: ..  FP: ..  FN: ..

- ...

---------------------------------------------------
*** Overall ***
---------------------------------------------------

  Total Expected:  206
  True Positives:  ...
  False Positives: ...
  False Negatives: ...
```

### 4.3 Flow

```
main():
  1. Parse CLI args
  2. Resolve dataset path: workspace_root/datasets/{args.dataset}/
  3. Collect .sol files (optionally filtered by --category)
  4. For each .sol file:
     a. Parse annotations → Vec<AnnotatedBug>
     b. Print expected bugs (if --verbose)
     c. Compile with solc and run analyzer → Vec<DetectedBug>
     d. Print detected bugs (if --verbose)
     e. Match → MatchResult { TP, FP, FN }
     f. Print per-file summary (if --verbose)
  5. Aggregate per-category and overall stats
  6. Print evaluation report
```

### 4.4 Matching Strategy

Reuse `match_file()` from existing `benchmark.rs` with one enhancement:

**Primary match**: exact category + exact `bug_line` match (existing behavior)
**Fallback match**: exact category + `|detected.start_line - expected.bug_line| <= 3`

This handles cases where the AST location is slightly offset from the annotated source line.

### 4.5 Error Handling

- If compilation fails for a file → log warning, skip, count as "skipped"
- The binary should **not** exit with error on individual file failures
- Track and report: `compiled`, `skipped`, `compile_errors`

---

## 5. File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` (workspace) | **MODIFY** | Add `"crates/benchmark"` to `members` |
| `crates/benchmark/Cargo.toml` | **NEW** | Crate manifest with deps on `analyzer`, `bugs`, `frontend`, `clap` |
| `crates/benchmark/src/main.rs` | **NEW** | CLI entry point with `--dataset`, `--category`, `--verbose` flags |
| `crates/benchmark/src/evaluate.rs` | **NEW** | Matching logic (migrated from `analyzer/tests/benchmark.rs`) + fixed `run_analyze_on_file()` |
| `crates/benchmark/src/report.rs` | **NEW** | Evaluation report formatter |

The existing `crates/analyzer/tests/benchmark.rs` can be left as-is or removed later — the new crate supersedes it.

---

## 6. Running

```bash
# Full smartbugs-curated evaluation
cargo run -p benchmark -- --dataset solidity/smartbugs-curated

# Single category
cargo run -p benchmark -- --dataset solidity/smartbugs-curated --category reentrancy

# Verbose (per-file details)
cargo run -p benchmark -- --dataset solidity/smartbugs-curated --verbose

# With specific solc version
cargo run -p benchmark -- --dataset solidity/smartbugs-curated --solc-version 0.4.26
```

---

## 7. Task Checklist

### 7.1 Workspace Setup

- [x] Add `"crates/benchmark"` to `members` in workspace `Cargo.toml`
- [x] Create `crates/benchmark/Cargo.toml` with dependencies: `analyzer`, `bugs`, `frontend`, `common`, `clap`

### 7.2 CLI Entry Point (`crates/benchmark/src/main.rs`)

- [x] Define `Args` struct with `--dataset`, `--category`, `--verbose`, `--solc-version` flags
- [x] Implement `main()` that parses args and resolves dataset path: `workspace_root/datasets/{args.dataset}/`
- [x] Validate that the resolved dataset path exists, exit with error if not
- [x] Collect `.sol` files using `bugs::datasets::smartbugs::collect_sol_files()`
- [x] If `--category` is set, filter to files under that subdirectory only
- [x] Wire up the evaluate → report flow

### 7.3 Evaluation Logic (`crates/benchmark/src/evaluate.rs`)

- [x] Migrate `DetectedBug` struct from `crates/analyzer/tests/benchmark.rs`
- [x] Migrate `MatchResult` struct from `crates/analyzer/tests/benchmark.rs`
- [x] Migrate `MatchedBug` struct from `crates/analyzer/tests/benchmark.rs`
- [x] Migrate `match_file()` function from `crates/analyzer/tests/benchmark.rs`
- [x] Add line tolerance (±3) as fallback matching in `match_file()`
- [x] Implement `run_analyze_on_file()` — actually compile the `.sol` file:
  - [x] Call `frontend::solidity::parsing::parse_input_file()` with solc version
  - [x] Build `AnalysisContext` from the parsed source units
  - [x] Run `PipelineEngine::new(PipelineConfig::default()).run(&mut context)`
  - [x] Convert `Bug` → `DetectedBug`
  - [x] Handle compilation failures gracefully (return empty vec + log warning)
- [x] Implement per-file evaluation function:
  - [x] Parse annotations using `bugs::datasets::smartbugs::parse_annotations()`
  - [x] Run analyzer via `run_analyze_on_file()`
  - [x] Match via `match_file()`
  - [x] Return `FileResult` with TP/FP/FN counts
- [x] Implement dataset-wide evaluation loop:
  - [x] Iterate over all `.sol` files
  - [x] Accumulate per-category stats (keyed by `BugCategory`)
  - [x] Track file counts: total, compiled, skipped

### 7.4 Report Formatting (`crates/benchmark/src/report.rs`)

- [x] Use `common::utils::print_header` for top-level report header
- [x] Use `common::utils::print_subheader` for section headers (Per-Category, Overall)
- [x] Print file summary line: total / compiled / skipped
- [x] Print per-category results as list: `- Category (N files, M expected) TP: .. FP: .. FN: ..`
- [x] Print overall totals: Total Expected, TP, FP, FN
- [x] If `--verbose`, print per-file details:
  - [x] File path
  - [x] Expected bugs list (category @ line)
  - [x] Detected bugs list (name(category) @ line)
  - [x] Per-file TP/FP/FN

### 7.5 Build & Verify

- [x] Run `cargo check` to verify compilation
- [x] Run `cargo build -p benchmark` to build the binary
- [x] Run `cargo run -p benchmark -- --dataset solidity/smartbugs-curated` end-to-end
- [x] Verify output format matches the expected report layout
