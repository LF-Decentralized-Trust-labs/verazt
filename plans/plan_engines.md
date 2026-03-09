# Plan: Restructure `scanner` to Add `engines/` Module

## Overview

Two sequential phases:

1. **Phase 1** — Move the entire `absint` stack (framework + analyses) from `scanner`
   into the `analysis` crate, so both `scanner` and `verifier` can share it.
2. **Phase 2** — Restructure `scanner` to introduce an `engines/` module containing
   `pattern/` (renamed from `grep/`), with `absint` removed (now from `analysis`).
   A placeholder for the future `datalog/` engine is added.

## Target Structure

```
analysis crate
  (existing passes: ast/, sir/, air/, pass infra)
  absint/              ← moved from scanner: lattice, solver, cfg, var, utils
    analyses/          ← moved from scanner: def_use, liveness, reaching_defs,
                                             state_mutation, taint

scanner crate
  engines/
    mod.rs
    pattern/           ← was grep/: AST/SIR structural pattern matching
      mod.rs
      builder.rs
      composite.rs
      core.rs
      matcher.rs
      primitives.rs
    datalog/           ← placeholder: future Datalog-based detection engine
      mod.rs
  detectors/           ← some use engines::pattern, some use analysis::absint
  pipeline/
  output/
  config/
  cli/
  lib.rs
```

## Detection Strategy by Engine

| Engine | Location | When to use | Examples |
|---|---|---|---|
| `engines::pattern` | `scanner-internal` | Structural/syntactic patterns visible in AST or SIR | `delegatecall`, `tx.origin`, floating pragma, visibility |
| `analysis::absint` | `analysis` crate (shared) | Requires dataflow reasoning across basic blocks | reentrancy, taint propagation, uninitialized reads |
| `engines::datalog` | `scanner-internal` (future) | Relational/logical reasoning over program facts | TBD |

---

## Phase 1: Move `absint` to the `analysis` Crate

### Step 1.1: Move Directories

```
crates/scanner/src/absint/  →  crates/analysis/src/absint/
```

### Step 1.2: Fix `use crate::absint::` Paths Inside the Moved Files

All files inside `absint/` reference `crate::absint::*`. After the move into
`analysis`, these become `crate::absint::*` again (same module name, different
crate root) — **no path changes needed** inside the absint files themselves,
since the module hierarchy is preserved exactly.

### Step 1.3: Update `analysis/src/lib.rs`

- Add `pub mod absint;`
- Add re-exports for the absint public API:
  ```rust
  pub use absint::{
      ControlFlowGraph, BasicBlock, BasicBlockId, Terminator,
      Lattice, FlatLattice, MapLattice, PowerSetLattice, ProductLattice,
      DataFlowSolver, DataFlowResult, Direction, Transfer,
      VarId, VarScope,
  };
  pub use absint::analyses::{/* taint, reaching_defs, etc. */};
  ```
- Update doc comment to mention the absint framework.

### Step 1.4: Update `scanner/src/lib.rs`

- Remove `pub mod absint;`
- Remove absint re-exports (if any).
- Update doc comment: remove mention of `absint` as a scanner module.

### Step 1.5: Update All `crate::absint::` References in `scanner`

No scanner detector currently imports `crate::absint::` directly (confirmed),
so **no scanner detector files need changes** in Phase 1.

### Step 1.6: Verify Phase 1

```bash
cargo check -p analysis
cargo check -p scanner
cargo test -p analysis
```

---

## Phase 2: Restructure `scanner` — `grep/` → `engines/pattern/`

### Step 2.1: Move Directory

```
scanner/src/grep/  →  scanner/src/engines/pattern/
```

Create `scanner/src/engines/mod.rs`.

### Step 2.2: Fix `use crate::grep::` Paths Inside `engines/pattern/`

| File | Old prefix | New prefix |
|---|---|---|
| `engines/pattern/builder.rs` | `crate::grep::` | `crate::engines::pattern::` |
| `engines/pattern/composite.rs` | `crate::grep::` | `crate::engines::pattern::` |
| `engines/pattern/matcher.rs` | `crate::grep::` | `crate::engines::pattern::` |
| `engines/pattern/primitives.rs` | `crate::grep::` | `crate::engines::pattern::` |

### Step 2.3: Fix `use crate::grep::` in Detectors

Five detectors import from `crate::grep::` directly:

| File | Old | New |
|---|---|---|
| `detectors/delegatecall.rs:10` | `crate::grep::` | `crate::engines::pattern::` |
| `detectors/deprecated.rs:10` | `crate::grep::` | `crate::engines::pattern::` |
| `detectors/low_level_call.rs:11` | `crate::grep::` | `crate::engines::pattern::` |
| `detectors/timestamp_dependence.rs:11` | `crate::grep::` | `crate::engines::pattern::` |
| `detectors/tx_origin.rs:11` | `crate::grep::` | `crate::engines::pattern::` |

### Step 2.4: Update `scanner/src/lib.rs`

- Remove `pub mod grep;`
- Add `pub mod engines;`
- Update doc comment: replace mention of `grep` with `engines::pattern`.

### Step 2.5: Create `scanner/src/engines/mod.rs`

```rust
//! Detection Engines
//!
//! Pluggable engines that power the scanner's detectors:
//!
//! - `pattern`: AST/SIR structural pattern matching. Detectors implement a
//!   condition and the engine handles traversal and result collection.
//! - `datalog`: (future) Relational reasoning over program facts expressed
//!   as Datalog rules.
//!
//! Dataflow analysis is provided by the shared `analysis` crate
//! (`analysis::absint`) and is used directly by detectors that require
//! reasoning across basic blocks.

pub mod pattern;
pub mod datalog;
```

### Step 2.6: Create `scanner/src/engines/datalog/mod.rs` (placeholder)

```rust
//! Datalog Detection Engine (placeholder)
//!
//! Future engine for expressing detection logic as Datalog rules,
//! enabling relational reasoning over program facts extracted from AST/SIR.
//!
//! # Planned Design
//!
//! - Extract facts from AST/SIR (nodes, edges, properties)
//! - Evaluate Datalog rules over the fact base
//! - Return findings as detector results
```

### Step 2.7: Update `scanner/src/engines/pattern/mod.rs` (was `grep/mod.rs`)

- Update module doc: replace "grep" references with "pattern".
- Describe the intended usage: detectors implement a `Condition` and register
  it with the engine; the engine drives traversal.
- Update example `use` path: `use scanner::grep::` → `use scanner::engines::pattern::`.

### Step 2.8: Verify Phase 2

```bash
cargo check -p scanner
cargo test -p scanner
cargo build
```

---

## Summary of All Changed Files

### Phase 1 — Move `absint` to `analysis`

| File | Change |
|---|---|
| `crates/scanner/src/absint/` (directory) | Moved to `crates/analysis/src/absint/` |
| `crates/analysis/src/lib.rs` | Add `pub mod absint;` + re-exports + doc update |
| `crates/scanner/src/lib.rs` | Remove `pub mod absint;` + doc update |

### Phase 2 — `grep/` → `engines/pattern/` + datalog placeholder

| File | Change |
|---|---|
| `scanner/src/grep/` (directory) | Moved to `scanner/src/engines/pattern/` |
| `scanner/src/engines/mod.rs` | New file |
| `scanner/src/engines/datalog/mod.rs` | New placeholder file |
| `scanner/src/lib.rs` | `pub mod grep` → `pub mod engines` + doc update |
| `scanner/src/engines/pattern/mod.rs` | Doc comment updated |
| `scanner/src/engines/pattern/builder.rs` | `crate::grep::` → `crate::engines::pattern::` |
| `scanner/src/engines/pattern/composite.rs` | `crate::grep::` → `crate::engines::pattern::` |
| `scanner/src/engines/pattern/matcher.rs` | `crate::grep::` → `crate::engines::pattern::` |
| `scanner/src/engines/pattern/primitives.rs` | `crate::grep::` → `crate::engines::pattern::` |
| `scanner/src/detectors/delegatecall.rs` | `crate::grep::` → `crate::engines::pattern::` |
| `scanner/src/detectors/deprecated.rs` | `crate::grep::` → `crate::engines::pattern::` |
| `scanner/src/detectors/low_level_call.rs` | `crate::grep::` → `crate::engines::pattern::` |
| `scanner/src/detectors/timestamp_dependence.rs` | `crate::grep::` → `crate::engines::pattern::` |
| `scanner/src/detectors/tx_origin.rs` | `crate::grep::` → `crate::engines::pattern::` |
