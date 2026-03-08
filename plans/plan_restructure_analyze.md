# Plan: Restructure `crates/analyze`

## Background

The current structure has three problems identified during review:

1. **Detectors are grouped by engine, not by bug.** `absint/detectors/` and `grep/detectors/`
   own the detectors, but a detector's engine is an implementation detail. Two detectors
   for the same bug (e.g. reentrancy) live in separate directories for arbitrary reasons.

2. **`absint` detectors don't use the absint engine.** `ReentrancyDfaDetector`,
   `CeiViolationDfaDetector`, etc. are named "Dfa" but perform manual AST walks.
   The generic DFA machinery (`lattice.rs`, `solver.rs`, `analyses/`) is unused.

3. **CFG naming mismatch.** `analysis/air/cfg.rs` builds a CFG from SIR `Stmt`s,
   not AIR `Op`s. It is misplaced under `air/`.

---

## Target Structure

```
crates/analyze/src/
├── lib.rs
├── cli.rs
├── config.rs
│
├── analysis/                    # Pass framework + analysis passes (unchanged)
│   ├── mod.rs
│   ├── context.rs               # AnalysisContext — central artifact store
│   ├── pass.rs                  # Pass + AnalysisPass traits
│   ├── pass_id.rs
│   ├── pass_level.rs
│   ├── pass_representation.rs
│   ├── manager.rs               # PassManager
│   ├── scheduler.rs
│   ├── executor.rs
│   ├── dependency.rs
│   ├── ast/                     # AST-level analysis passes
│   │   ├── symbol_table.rs
│   │   ├── call_graph.rs
│   │   ├── inheritance_graph.rs
│   │   ├── modifier_analysis.rs
│   │   └── type_index.rs
│   ├── sir/                     # SIR-level analysis passes
│   │   ├── structural.rs
│   │   └── cfg.rs               # MOVED from analysis/air/cfg.rs (operates on SIR)
│   └── air/                    # AIR-level analysis passes
│       ├── generation.rs
│       ├── taint.rs
│       ├── access_control.rs
│       └── arithmetic.rs
│
├── absint/                      # Pure DFA engine — no detectors
│   ├── mod.rs
│   ├── cfg.rs                   # Generic DFA CFG (with def/use metadata)
│   ├── lattice.rs               # Lattice framework
│   ├── solver.rs                # Worklist fixpoint solver
│   ├── var.rs
│   ├── utils.rs
│   └── analyses/                # Reusable dataflow analyses
│       ├── reaching_defs.rs
│       ├── liveness.rs
│       ├── def_use.rs
│       ├── taint.rs
│       └── state_mutation.rs
│
├── grep/                        # Pure pattern engine — no detectors
│   ├── mod.rs
│   ├── core.rs
│   ├── primitives.rs
│   ├── composite.rs
│   ├── builder.rs
│   └── matcher.rs
│
├── detectors/                   # NEW: all detectors, grouped by bug category
│   ├── mod.rs                   # register_all_detectors() + re-exports
│   ├── reentrancy.rs            # ReentrancyAstDetector + AirReentrancyDetector
│   ├── access_control.rs        # Grep + SIR + AIR variants
│   ├── arithmetic.rs            # SIR + AIR variants
│   ├── tx_origin.rs             # Grep + SIR variants
│   ├── cei_violation.rs
│   ├── dead_code.rs
│   ├── unchecked_call.rs
│   ├── uninitialized.rs
│   ├── acquires_mismatch.rs     # Move-specific
│   ├── missing_modifies.rs      # Move-specific
│   ├── missing_pda_constraint.rs # Anchor-specific
│   ├── delegatecall.rs
│   ├── deprecated.rs
│   ├── floating_pragma.rs
│   ├── low_level_call.rs
│   ├── shadowing.rs
│   ├── timestamp_dependence.rs
│   ├── visibility.rs
│   ├── constant_state_var.rs
│   └── centralization_risk.rs
│
├── pipeline/                    # Orchestration (unchanged)
│   ├── mod.rs
│   ├── engine.rs
│   ├── detector.rs              # BugDetectionPass trait
│   └── registry.rs              # DetectorRegistry (delegates to detectors/mod.rs)
│
└── output/                      # Output formatters (unchanged)
    ├── mod.rs
    ├── formatter.rs
    ├── json.rs
    ├── markdown.rs
    └── sarif.rs
```

---

## Steps

### Step 1 — Move `analysis/air/cfg.rs` → `analysis/sir/cfg.rs`

The file imports `mlir::sir::{Expr, Stmt}` and builds a CFG from SIR statements.
It does not operate on AIR `Op`s. Moving it corrects the naming mismatch.

- Move file
- Update `analysis/air/mod.rs` to remove the export
- Update `analysis/sir/mod.rs` to add the export
- Update `PassId::IrCfg` usages in `pipeline/engine.rs`

### Step 2 — Create `detectors/` module

Create `crates/analyze/src/detectors/` with one file per bug category.

Each file:
- Imports the engine it needs (`crate::absint`, `crate::grep`, `mlir::air`, etc.)
- Defines all tier variants of that detector (AST / SIR / AIR) in one place
- Exports them from `detectors/mod.rs`

Mapping of existing detectors to new files:

| New file | Moved from |
|---|---|
| `reentrancy.rs` | `absint/detectors/reentrancy.rs` + `absint/detectors/air/reentrancy.rs` |
| `access_control.rs` | `grep/detectors/missing_access_control.rs` + `absint/detectors/sir/missing_access_control.rs` + `absint/detectors/air/access_control.rs` |
| `arithmetic.rs` | `absint/detectors/sir/unchecked_arithmetic.rs` + `absint/detectors/air/arithmetic.rs` |
| `tx_origin.rs` | `grep/detectors/tx_origin.rs` + `absint/detectors/sir/tx_origin_auth.rs` |
| `cei_violation.rs` | `absint/detectors/cei_violation.rs` |
| `dead_code.rs` | `absint/detectors/dead_code.rs` |
| `unchecked_call.rs` | `absint/detectors/unchecked_call.rs` |
| `uninitialized.rs` | `absint/detectors/uninitialized.rs` |
| `acquires_mismatch.rs` | `absint/detectors/sir/acquires_mismatch.rs` |
| `missing_modifies.rs` | `absint/detectors/sir/missing_modifies.rs` |
| `missing_pda_constraint.rs` | `absint/detectors/sir/missing_pda_constraint.rs` |
| `delegatecall.rs` | `grep/detectors/delegatecall.rs` |
| `deprecated.rs` | `grep/detectors/deprecated.rs` |
| `floating_pragma.rs` | `grep/detectors/floating_pragma.rs` |
| `low_level_call.rs` | `grep/detectors/low_level_call.rs` |
| `shadowing.rs` | `grep/detectors/shadowing.rs` |
| `timestamp_dependence.rs` | `grep/detectors/timestamp_dependence.rs` |
| `visibility.rs` | `grep/detectors/visibility.rs` |
| `constant_state_var.rs` | `grep/detectors/constant_state_var.rs` |
| `centralization_risk.rs` | `grep/detectors/centralization_risk.rs` |

### Step 3 — Remove detector subdirs from `absint/` and `grep/`

- Delete `absint/detectors/`
- Delete `grep/detectors/`
- Update `absint/mod.rs` to remove `pub mod detectors`
- Update `grep/mod.rs` to remove `pub mod detectors`

### Step 4 — Update `pipeline/registry.rs`

`register_all_detectors()` currently imports from `absint::detectors` and `grep::detectors`.
Update all imports to come from `crate::detectors`.

### Step 5 — Update `lib.rs`

Add `pub mod detectors;` and re-export from `detectors`.

### Step 6 — Rename AST-walking "Dfa" detectors

The detectors in `absint/detectors/` that don't use the DFA framework should be
renamed to drop the misleading `Dfa` suffix:

| Old name | New name |
|---|---|
| `ReentrancyDfaDetector` | `ReentrancyAstDetector` |
| `CeiViolationDfaDetector` | `CeiViolationAstDetector` |
| `DeadCodeDfaDetector` | `DeadCodeAstDetector` |
| `UncheckedCallDfaDetector` | `UncheckedCallAstDetector` |
| `UninitializedDfaDetector` | `UninitializedAstDetector` |

---

## What does NOT change

- `analysis/` pass framework (`AnalysisContext`, `Pass`, `PassManager`) — untouched
- `absint/` DFA engine (`lattice.rs`, `solver.rs`, `analyses/`) — kept as-is, ready
  to be wired up when detectors are upgraded to use real dataflow
- `grep/` pattern engine — kept as-is
- `pipeline/` orchestration — only import paths change
- `output/`, `config/`, `cli/` — untouched
- The `mlir` crate — untouched

---

## Out of scope

- Implementing the DFA solver in actual detectors (separate task)
- Wiring `analysis/ast/` pass artifacts into detectors (separate task)
- Unifying the three CFG types (separate task)
