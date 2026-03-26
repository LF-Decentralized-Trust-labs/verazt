# Plan: Restructure `analyzer` crate — flatten `analysis/` and reorganise `detectors/`

## Motivation

The crate is named `analyzer`, so the `analysis/` sub-module is a redundant wrapper.
Every module inside it — `context`, `passes`, `frameworks`, `pipeline` (pass manager) —
is a first-class concept of the crate and belongs at the root level.

Additionally, `detectors/` mixes infrastructure modules (`id`, `traits`, `registry`)
with concrete detector implementations. Separating them into `base/`, `sir/`, and `bir/`
mirrors the same layering used in `passes/`.

---

## Target structure

```
crates/analyzer/src/
  context.rs              (was analysis/context.rs)
  frameworks/             (was analysis/frameworks/)
    cfa/
    dfa/
    datalog/
  passes/                 (was analysis/passes/)
    base/
    sir/
    bir/
    vir/
  pass_manager/           (was analysis/pipeline/)
    dependency.rs
    executor.rs
    manager.rs
    scheduler.rs
    mod.rs
  detectors/              (reorganised)
    base/                 (was detectors/id.rs, traits.rs, registry.rs)
      id.rs
      traits.rs
      registry.rs
      mod.rs
    sir/                  (all SIR structural detectors)
      cei_violation.rs
      centralization_risk.rs
      constant_state_var.rs
      dead_code.rs
      delegatecall.rs
      deprecated.rs
      floating_pragma.rs
      low_level_call.rs
      missing_access_control.rs
      reentrancy.rs
      shadowing.rs
      timestamp_dependence.rs
      tx_origin.rs
      unchecked_call.rs
      uninitialized.rs
      visibility.rs
      acquires_mismatch.rs
      missing_modifies.rs
      missing_pda_constraint.rs
      sir_missing_access_control.rs
      tx_origin_auth.rs
      unchecked_arithmetic.rs
      mod.rs
    bir/                  (all BIR dataflow detectors)
      access_control.rs
      arithmetic.rs
      reentrancy.rs         (AIRReentrancyDetector — move from sir/reentrancy.rs)
      mod.rs
    mod.rs                (updated)
  pipeline.rs             (unchanged — two-phase orchestrator)
  output/                 (unchanged)
                          (artifacts.rs removed)
  cli.rs                  (unchanged)
  config.rs               (unchanged)
  lib.rs                  (updated)
```

The key rename is `analysis/pipeline/` → `pass_manager/` to avoid collision with the
existing `pipeline.rs` (the two-phase analysis→detection orchestrator). `pass_manager`
is preferred over `scheduling` because the `PassManager` struct lives there and the
module handles more than just scheduling (dependency resolution, execution, timing).

---

## Steps

### 1. Rename `analysis/pipeline/` → `pass_manager/`

Move all files:
- `analysis/pipeline/mod.rs`       → `pass_manager/mod.rs`
- `analysis/pipeline/manager.rs`   → `pass_manager/manager.rs`
- `analysis/pipeline/scheduler.rs` → `pass_manager/scheduler.rs`
- `analysis/pipeline/executor.rs`  → `pass_manager/executor.rs`
- `analysis/pipeline/dependency.rs`→ `pass_manager/dependency.rs`

Update all `crate::analysis::pipeline` references to `crate::pass_manager`.

### 2. Move `analysis/context.rs` → `context.rs`

Update all `crate::analysis::context` / `analysis::context` references to `crate::context`.

### 3. Move `analysis/frameworks/` → `frameworks/`

Update all `crate::analysis::frameworks` references to `crate::frameworks`.

### 4. Move `analysis/passes/` → `passes/`

Update all `crate::analysis::passes` / `analysis::passes` references to `crate::passes`.

### 5. Reorganise `detectors/`

#### 5a. Create `detectors/base/`

Move:
- `detectors/id.rs`       → `detectors/base/id.rs`
- `detectors/traits.rs`   → `detectors/base/traits.rs`
- `detectors/registry.rs` → `detectors/base/registry.rs`

Create `detectors/base/mod.rs` re-exporting all public types.

Update all `crate::detectors::id`, `crate::detectors::traits`, `crate::detectors::registry`
references to `crate::detectors::base::*`.

#### 5b. Create `detectors/sir/`

Move all SIR structural detector files into `detectors/sir/`:
- `cei_violation`, `centralization_risk`, `constant_state_var`, `dead_code`,
  `delegatecall`, `deprecated`, `floating_pragma`, `low_level_call`,
  `missing_access_control`, `reentrancy` (SIR variant), `shadowing`,
  `timestamp_dependence`, `tx_origin`, `unchecked_call`, `uninitialized`, `visibility`
- Multi-chain SIR: `acquires_mismatch`, `missing_modifies`, `missing_pda_constraint`,
  `sir_missing_access_control`, `tx_origin_auth`, `unchecked_arithmetic`

Create `detectors/sir/mod.rs` re-exporting all SIR detector types.

#### 5c. Create `detectors/bir/`

Move all BIR dataflow detector files into `detectors/bir/`:
- `access_control`, `arithmetic`
- Move the `AIRReentrancyDetector` from `reentrancy.rs` here (currently in the same
  file as `ReentrancySirDetector` — split the file if needed)

Create `detectors/bir/mod.rs` re-exporting all BIR detector types.

#### 5d. Update `detectors/mod.rs`

Replace flat module declarations with:
```rust
pub mod base;
pub mod sir;
pub mod bir;

pub use base::{BugDetectionPass, ConfidenceLevel, DetectorError, DetectorResult,
               DetectorId, DetectorRegistry, register_all_detectors, create_bug,
               create_bug_with_details};
```

### 6. Update `lib.rs`

- Remove `pub mod analysis;`
- Add `pub mod context;`, `pub mod frameworks;`, `pub mod passes;`, `pub mod pass_manager;`
- Update all `pub use crate::analysis::...` re-exports to point to the new paths.
- Remove the `pass` compatibility alias module (or keep it pointing at `passes::base` if
  downstream crates still use it).

### 7. Delete `analysis/mod.rs` and `analysis/` directory

Once all references are updated and tests pass, remove the now-empty directory.

### 8. Delete dead code: GREP framework, `SourceUnitsArtifact`, `artifacts.rs`

- Delete `analysis/frameworks/grep/` (6 files, ~30 KB) — no consumers exist.
- Delete `artifacts.rs` and remove `pub mod artifacts;` from `lib.rs`.
- Remove `context.store::<SourceUnitsArtifact>(...)` calls from `cli.rs` and
  `tests/benchmark.rs`.
- Remove the `SourceUnitsArtifact` import from `cli.rs`.

Rationale: the GREP framework was scaffolded but never wired to any detector.
`SourceUnitsArtifact` is write-only — stored in the context but never retrieved.
All SIR detectors operate on `scirs::sir::Module`, not `SourceUnit`.

### 9. Rename `ArtifactKey` → `ContextKey`

- In `context.rs`: rename trait `ArtifactKey` → `ContextKey`.
- In `context.rs`: rename field `typed_artifacts` → `typed_data` (and related methods).
- Update all `impl ArtifactKey for …` → `impl ContextKey for …` across the crate.

Rationale: `ArtifactKey` is a typed context slot, not a build artifact.
Renaming avoids collision with SARIF's `artifacts` concept.

### 10. Refactor SIR detectors to use existing `Visit<'a>` trait

The `Visit<'a>` trait in `scirs::sir::utils::visit` already provides the full
visitor pattern with default traversals for every SIR node type (`visit_expr`,
`visit_stmt`, `visit_binop_expr`, `visit_call_expr`, `visit_dialect_expr`,
`visit_contract_decl`, `visit_function_decl`, etc.).

Convert each SIR detector's `detect()` body from ad-hoc `structural::walk_*`
calls to a `Visit<'a>` impl. For example, `TxOriginSirDetector`:

```rust
struct TxOriginVisitor<'a> { bugs: &'a mut Vec<Bug>, contract: &'a str, func: &'a str }
impl<'a> Visit<'a> for TxOriginVisitor<'a> {
    fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
        if matches!(d, DialectExpr::Evm(EvmExpr::TxOrigin)) {
            self.bugs.push(/* ... */);
        }
    }
}
```

Detectors affected (18 files): `cei_violation`, `centralization_risk`,
`constant_state_var`, `dead_code`, `delegatecall`, `deprecated`,
`floating_pragma`, `low_level_call`, `missing_access_control`,
`missing_modifies`, `reentrancy` (SIR variant), `shadowing`,
`sir_missing_access_control`, `timestamp_dependence`, `tx_origin`,
`unchecked_call`, `unchecked_arithmetic`, `acquires_mismatch`.

### 11. Trim `query` module — remove walkers, keep domain helpers

Remove the 5 duplicated walker functions from `query.rs` (`walk_exprs`,
`walk_stmts`, `walk_binops`, `walk_dialect_exprs`, `walk_function_calls`
and all their `_stmt` / `_expr` helpers) — these are now superseded by
the `Visit<'a>` trait.

**Keep the domain-specific helpers** — they encode smart-contract semantics
that the generic visitor doesn't cover:

| Helper | Used by | Purpose |
|---|---|---|
| `storage_names` | 7 detectors + 1 pass | Collect storage var names |
| `is_public_function` | 3 detectors | Check visibility attr |
| `has_reentrancy_guard` | 2 detectors | Check guard attr |
| `has_storage_write` | 4 detectors | Recursive storage-write check |
| `expr_references_storage` | 3 detectors + 1 pass | Storage reference check |
| `is_evm_external_call` | 3 detectors | Classify call as external |
| `is_constant_storage` | 2 detectors | Check constant/immutable attr |
| `has_assert_before_storage_write` | 2 detectors | Assert-before-write check |

Rename the module from `query` → `helpers` (or `predicates`) to reflect that
it now contains only domain predicates, not tree walkers.

### 12. Update documentation and doc-comments

- Update `lib.rs` crate-level doc comment (Architecture section).
- Update any `//!` module doc-comments that reference `analysis::`, the old
  detector paths, `ArtifactKey`/`artifacts`, or `query::walk_*`.

---

## Reference: key types and their new paths

| Type | Old path | New path |
|---|---|---|
| `AnalysisContext` | `analysis::context` | `context` |
| `ContextKey` (was `ArtifactKey`) | `analysis::context` | `context` |
| `PassManager` | `analysis::pipeline::manager` | `pass_manager::manager` |
| `PassScheduler` | `analysis::pipeline::scheduler` | `pass_manager::scheduler` |
| `PassExecutor` | `analysis::pipeline::executor` | `pass_manager::executor` |
| `DependencyGraph` | `analysis::pipeline::dependency` | `pass_manager::dependency` |
| `Pass` / `AnalysisPass` | `analysis::passes::base` | `passes::base` |
| `WriteSetPass` | `analysis::passes::sir` | `passes::sir` |
| `ICFGPass`, `TaintPass`, … | `analysis::passes::bir` | `passes::bir` |
| `DataFlowSolver`, lattices | `analysis::frameworks::dfa` | `frameworks::dfa` |
| `DomTree`, `LoopInfo`, … | `analysis::frameworks::cfa` | `frameworks::cfa` |
| `Visit<'a>` | `scirs::sir::utils::visit` | *(unchanged)* |
| Domain helpers | `scirs::sir::utils::query` | `scirs::sir::utils::helpers` |
| `BugDetectionPass`, `DetectorId`, … | `detectors::{traits,id}` | `detectors::base` |
| `DetectorRegistry` | `detectors::registry` | `detectors::base` |
| SIR detector types | `detectors::*` (flat) | `detectors::sir` |
| BIR detector types | `detectors::*` (flat) | `detectors::bir` |

All of these are re-exported from `lib.rs` and `detectors/mod.rs`, so the public API
of the crate does not change — only internal paths need updating.

---

## Checklist

### Step 1 — Rename `analysis/pipeline/` → `pass_manager/`
- [x] Move `analysis/pipeline/mod.rs` → `pass_manager/mod.rs`
- [x] Move `analysis/pipeline/manager.rs` → `pass_manager/manager.rs`
- [x] Move `analysis/pipeline/scheduler.rs` → `pass_manager/scheduler.rs`
- [x] Move `analysis/pipeline/executor.rs` → `pass_manager/executor.rs`
- [x] Move `analysis/pipeline/dependency.rs` → `pass_manager/dependency.rs`
- [x] Update all `crate::analysis::pipeline` references → `crate::pass_manager`
- [x] `cargo check` passes

### Step 2 — Move `analysis/context.rs` → `context.rs`
- [x] Move file
- [x] Update all `crate::analysis::context` references → `crate::context`
- [x] `cargo check` passes

### Step 3 — Move `analysis/frameworks/` → `frameworks/`
- [x] Move directory
- [x] Update all `crate::analysis::frameworks` references → `crate::frameworks`
- [x] `cargo check` passes

### Step 4 — Move `analysis/passes/` → `passes/`
- [x] Move directory
- [x] Update all `crate::analysis::passes` references → `crate::passes`
- [x] `cargo check` passes

### Step 5 — Reorganise `detectors/`
- [x] **5a** Move `detectors/id.rs` → `detectors/base/id.rs`
- [x] **5a** Move `detectors/traits.rs` → `detectors/base/traits.rs`
- [x] **5a** Move `detectors/registry.rs` → `detectors/base/registry.rs`
- [x] **5a** Create `detectors/base/mod.rs` with re-exports
- [x] **5a** Update all `crate::detectors::{id,traits,registry}` → `crate::detectors::base::*`
- [x] **5b** Move 22 SIR detector files into `detectors/sir/`
- [x] **5b** Create `detectors/sir/mod.rs` with re-exports
- [x] **5c** Move BIR detector files into `detectors/bir/`
- [x] **5c** Split `AIRReentrancyDetector` out of `reentrancy.rs` into `detectors/bir/reentrancy.rs`
- [x] **5c** Create `detectors/bir/mod.rs` with re-exports
- [x] **5d** Update `detectors/mod.rs` — replace flat declarations with `base`/`sir`/`bir` modules
- [x] `cargo check` passes

### Step 6 — Update `lib.rs`
- [x] Remove `pub mod analysis;`
- [x] Add `pub mod context;`, `pub mod frameworks;`, `pub mod passes;`, `pub mod pass_manager;`
- [x] Update all `pub use crate::analysis::...` re-exports → new paths
- [x] Decide: keep or remove `pass` compatibility alias
- [x] `cargo check` passes

### Step 7 — Delete `analysis/` directory
- [x] Verify `analysis/` is empty (all files moved)
- [x] Delete `analysis/mod.rs` and `analysis/` directory
- [x] `cargo check` passes
- [x] `cargo test` passes (full verification gate)

### Step 8 — Delete dead code
- [x] Delete `analysis/frameworks/grep/` (6 files)
- [x] Delete `artifacts.rs`
- [x] Remove `pub mod artifacts;` from `lib.rs`
- [x] Remove `SourceUnitsArtifact` import from `cli.rs`
- [x] Remove `context.store::<SourceUnitsArtifact>(...)` from `cli.rs`
- [x] Remove `SourceUnitsArtifact` usage from `tests/benchmark.rs`
- [x] `cargo check` passes

### Step 9 — Rename `ArtifactKey` → `ContextKey`
- [x] Rename trait `ArtifactKey` → `ContextKey` in `context.rs`
- [x] Rename field `typed_artifacts` → `typed_data` in `context.rs`
- [x] Update all `impl ArtifactKey` → `impl ContextKey` across crate
- [x] Update all `ArtifactKey` references in tests
- [x] `cargo check` passes

### Step 10 — Refactor SIR detectors to use `Visit<'a>`
- [x] `cei_violation.rs` — replace `structural::walk_*` with `Visit<'a>` impl
- [x] `centralization_risk.rs`
- [x] `constant_state_var.rs`
- [x] `dead_code.rs`
- [x] `delegatecall.rs`
- [x] `deprecated.rs`
- [x] `floating_pragma.rs`
- [x] `low_level_call.rs`
- [x] `missing_access_control.rs`
- [x] `missing_modifies.rs`
- [x] `reentrancy.rs` (SIR variant only)
- [x] `shadowing.rs`
- [x] `sir_missing_access_control.rs`
- [x] `timestamp_dependence.rs`
- [x] `tx_origin.rs`
- [x] `unchecked_call.rs`
- [x] `unchecked_arithmetic.rs`
- [x] `acquires_mismatch.rs`
- [x] `cargo check` passes
- [x] `cargo test` passes (full verification gate)

### Step 11 — Trim `query` module
- [x] Remove `walk_exprs` + helpers from `query.rs`
- [x] Remove `walk_stmts` + helpers from `query.rs`
- [x] Remove `walk_binops` + helpers from `query.rs`
- [x] Remove `walk_dialect_exprs` + helpers from `query.rs`
- [x] Remove `walk_function_calls` from `query.rs`
- [x] Verify 8 domain helpers are retained
- [x] Rename module `query` → `helpers`
- [x] Update all `use scirs::sir::utils::query` → `use scirs::sir::utils::helpers`
- [x] `cargo check` passes
- [x] `cargo test` passes (full verification gate)

### Step 12 — Update documentation
- [x] Update `lib.rs` crate-level doc comment
- [x] Update `//!` doc-comments referencing old `analysis::` paths
- [x] Update doc-comments referencing `ArtifactKey` / `artifacts`
- [x] Update doc-comments referencing `query::walk_*`
- [x] Final `cargo test` — all tests pass
