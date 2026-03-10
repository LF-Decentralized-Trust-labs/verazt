# Plan: Improve `analysis` Crate

## Overview

This plan captures a set of targeted improvements to the `crates/analysis` crate.
They are ordered by priority and grouped by theme. Implementation should follow the
order listed here to minimise merge conflicts and broken intermediate states.

---

## File Structure

### Before

```
crates/mlir/src/
├── sir/
│   ├── attrs.rs
│   ├── defs.rs
│   ├── dialect/
│   ├── exprs.rs, lits.rs, loc.rs, stmts.rs, types.rs, spec.rs
│   ├── module.rs
│   └── utils/
│       ├── mod.rs
│       ├── export.rs
│       ├── fold.rs
│       ├── map.rs
│       ├── printer.rs
│       └── visit.rs
└── air/
    ├── cfg.rs
    └── ...

crates/analysis/src/
├── lib.rs
├── pass.rs
├── pass_id.rs
├── pass_level.rs
├── pass_representation.rs
├── context.rs
├── manager.rs
├── scheduler.rs
├── executor.rs
├── dependency.rs
├── ast/                # DELETED (see step 1.5)
│   ├── mod.rs
│   ├── call_graph.rs
│   ├── inheritance_graph.rs
│   ├── modifier_analysis.rs
│   ├── symbol_table.rs
│   └── type_index.rs
├── sir/
│   ├── mod.rs
│   ├── cfg.rs          # ControlFlowGraph + BasicBlock types + CfgPass
│   └── structural.rs   # SIR tree-walking utilities
├── air/
│   ├── mod.rs
│   ├── access_control.rs
│   ├── arithmetic.rs
│   ├── generation.rs
│   └── taint.rs
└── absint/
    ├── mod.rs
    ├── cfg.rs
    ├── lattice.rs
    ├── solver.rs
    ├── utils.rs
    ├── var.rs
    └── analyses/
        ├── mod.rs
        ├── def_use.rs
        ├── liveness.rs
        ├── reaching_defs.rs
        ├── state_mutation.rs
        └── taint.rs
```

### After

```
crates/mlir/src/
├── sir/
│   ├── ...                             # unchanged
│   ├── cfg.rs                          # NEW: ControlFlowGraph, BasicBlock, Terminator, BasicBlockId
│   └── utils/
│       ├── ...                         # unchanged
│       └── query.rs                    # NEW: is_public_function, has_reentrancy_guard,
│                                       #       walk_binops, walk_dialect_exprs, etc.
└── air/
    ├── cfg.rs                          # unchanged
    └── ...

crates/analysis/src/
├── lib.rs
├── context.rs                          # shared runtime state used by both pass and pipeline
├── pass/                               # pass abstraction (what a pass IS)
│   ├── mod.rs                          # re-exports Pass, AnalysisPass, PassId, PassLevel, etc.
│   ├── traits.rs                       # Pass + AnalysisPass traits (from pass.rs)
│   ├── id.rs                           # PassId enum (from pass_id.rs)
│   └── meta.rs                         # PassLevel + PassRepresentation (from pass_meta.rs)
├── pipeline/                           # execution machinery (how passes are RUN)
│   ├── mod.rs
│   ├── manager.rs                      # pass registry + entry point; produces AnalysisReport
│   ├── scheduler.rs                    # pure fn: registered passes → ExecutionSchedule
│   ├── executor.rs                     # drives execution + timing from ExecutionSchedule
│   └── dependency.rs                   # dependency graph + topological sort
├── passes/                             # all analysis passes, organised by IR layer
│   ├── mod.rs
│   ├── sir/
│   │   ├── mod.rs
│   │   └── cfg_pass.rs                 # CfgPass; data types live in mlir::sir::cfg
│   ├── air/
│   │   ├── mod.rs
│   │   └── taint_propagation.rs        # AIRTaintPropagationPass; computes TaintMap fact
│   └── vir/                            # placeholder for future VIR-layer passes
│       └── mod.rs
└── frameworks/                         # reusable analysis infrastructure (IR-agnostic)
    ├── mod.rs
    ├── dfa/                            # dataflow analysis framework (operates on AIR)
    │   ├── mod.rs
    │   ├── annotated_cfg.rs            # enhanced CFG with def/use sets (from absint/cfg.rs)
    │   ├── lattice.rs                  # Lattice trait + PowerSet, Map, Flat, Product impls
    │   ├── solver.rs                   # DataFlowSolver, Transfer trait, Direction enum
    │   ├── utils.rs                    # shared helpers
    │   ├── var.rs                      # VarId and variable identification
    │   └── analyses/                   # concrete analyses: domain + transfer + pass stub
    │       ├── mod.rs
    │       ├── def_use.rs
    │       ├── liveness.rs
    │       ├── reaching_defs.rs
    │       ├── state_mutation.rs
    │       └── taint.rs               # TaintSource/Sink/State domain + transfer + pass stub
    └── cfa/                            # control-flow analysis utilities (operates on AIR ICFG)
        ├── mod.rs
        ├── domtree.rs                  # Lengauer-Tarjan dominator + post-dominator trees
        ├── loops.rs                    # natural loop detection from dominator tree
        ├── reachability.rs             # BFS/DFS reachability queries over AIR ICFG
        └── callgraph.rs               # inter-procedural call graph built from SIR call sites
```

---

## Phase 1 — Low-effort, high-clarity wins (no logic changes)

### 1.1 Restructure into `passes/` + `frameworks/`

Introduce two top-level modules to replace the scattered `sir/`, `air/`, and
`absint/` directories:

- `passes/` — all analysis passes organised by IR layer (`sir/`, `air/`, `vir/`)
- `frameworks/` — reusable analysis infrastructure; currently `dfa/` (dataflow),
  with `cfa/` as a future placeholder for control-flow analysis

`absint/` is deleted as a top-level module. Its framework machinery
(`lattice.rs`, `solver.rs`, `var.rs`, `utils.rs`) moves flat into
`frameworks/dfa/`. Its concrete analyses move into `frameworks/dfa/analyses/`.
`air/taint.rs` (the pipeline pass) moves to `passes/air/taint_propagation.rs`.

**Changes:**
- Create `src/passes/mod.rs`, `src/passes/sir/mod.rs`, `src/passes/air/mod.rs`,
  `src/passes/vir/mod.rs`
- Move `src/sir/cfg_pass.rs` → `src/passes/sir/cfg_pass.rs`
- Move `src/air/taint.rs` → `src/passes/air/taint_propagation.rs`
- Create `src/frameworks/mod.rs`, `src/frameworks/dfa/mod.rs`
- Move `src/absint/lattice.rs`, `solver.rs`, `var.rs`, `utils.rs` flat into
  `src/frameworks/dfa/`
- Move `src/absint/analyses/` → `src/frameworks/dfa/analyses/`
- Remove top-level `mod absint;`, `mod sir;`, `mod air;` from `src/lib.rs`
- Add `pub mod passes;` and `pub mod frameworks;` to `src/lib.rs`
- Update all `use crate::absint::` paths to `use crate::frameworks::dfa::`
- Update any cross-crate paths that reference `analysis::absint`, `analysis::sir`,
  or `analysis::air`

### 1.2 Rename `absint/cfg.rs` → `frameworks/dfa/annotated_cfg.rs`

Both `sir/cfg.rs` and `absint/cfg.rs` exist. The `absint` one builds an enhanced
CFG annotated with def/use sets — a distinct concept from the raw CFG in `sir/`.
Renaming it removes the naming collision and clarifies its role inside `dfa/`.

**Changes:**
- Rename `src/absint/cfg.rs` → `src/frameworks/dfa/annotated_cfg.rs`
- Update `frameworks/dfa/mod.rs` to declare `mod annotated_cfg;`
- Update all internal `use super::cfg::` / `use crate::absint::cfg::` references

### 1.3 Consolidate tiny pass-metadata files

`pass_level.rs` and `pass_representation.rs` each define a single small enum.
Merge them into a new `pass_meta.rs` to reduce file-navigation noise.

```
src/pass.rs          # core Pass / AnalysisPass traits (unchanged)
src/pass_id.rs       # PassId enum (unchanged for now; see Phase 2)
src/pass_meta.rs     # PassLevel + PassRepresentation (merged from two files)
```

**Changes:**
- Create `src/pass_meta.rs` with the contents of both deleted files
- Delete `src/pass_level.rs` and `src/pass_representation.rs`
- Update `src/lib.rs` module declarations
- Update all `use crate::pass_level::` and `use crate::pass_representation::` paths

### 1.4 Move SIR CFG types and structural utilities to `mlir`

`ControlFlowGraph`, `BasicBlock`, `Terminator`, and `BasicBlockId` are IR
representation types — they belong in `mlir` alongside the SIR node types they
are built from, mirroring how `mlir::air::cfg` already holds the AIR CFG types.
The tree-walking utilities in `structural.rs` (`walk_binops`, `walk_dialect_exprs`,
`is_public_function`, etc.) are generic SIR query helpers that any crate working
with SIR could use — they fit naturally into `mlir::sir::utils`.

**Changes:**
- Add `crates/mlir/src/sir/cfg.rs` containing `ControlFlowGraph`, `BasicBlock`,
  `Terminator`, `BasicBlockId` (moved verbatim from `analysis/sir/cfg.rs`)
- Add `crates/mlir/src/sir/utils/query.rs` containing all functions from
  `analysis/sir/structural.rs`; re-export from `mlir::sir::utils`
- Update `crates/mlir/src/sir/mod.rs` to declare `pub mod cfg;` and add
  `query` to `utils/mod.rs`
- Rename `analysis/src/sir/cfg.rs` → `analysis/src/sir/cfg_pass.rs`; remove the
  moved types and replace with `use mlir::sir::cfg::*;` in `CfgPass`
- Delete `analysis/src/sir/structural.rs`; update all callers to
  `use mlir::sir::utils::query::*;`

### 1.5 Delete `ast/` module and remove `frontend` dependency from `analysis`

All 5 passes in `ast/` (`symbol_table`, `type_index`, `call_graph`,
`inheritance_graph`, `modifier_analysis`) import directly from
`frontend::solidity::ast` — they are Solidity-specific and conflict with the
goal of language-agnostic analysis through SIR. The information they extract
(contracts, functions, call edges, inheritance) is already encoded in the SIR
during lowering and should be queried from `mlir::sir` types instead.

**Changes:**
- Delete `src/ast/` directory and all files within it
- Remove `mod ast;` from `src/lib.rs`
- Remove all `PassId` variants for AST passes (`SymbolTable`, `TypeIndex`,
  `CallGraph`, `InheritanceGraph`, `ModifierAnalysis`, and any other
  `PassRepresentation::Ast` variants) from `src/pass_id.rs`
- Remove `source_units` field and any `frontend::solidity` imports from
  `src/context.rs`; `AnalysisContext` input is henceforth a SIR `Module`
- Update `src/manager.rs` to no longer accept or pass AST source units
- In `air/dfa/analyses/taint.rs` and `state_mutation.rs`, replace
  `frontend::solidity::ast::Loc` with `mlir::sir::Loc`
- Remove `frontend` from `[dependencies]` in `Cargo.toml`

### 1.6 Move bug-detection passes from `analysis/air/` to `scanner/detectors/`

`analysis/air/access_control.rs` and `analysis/air/arithmetic.rs` produce `Bug`
artifacts and depend on the `bugs` crate — they are detectors, not analyses.
The scanner already has `detectors/access_control.rs` and `detectors/arithmetic.rs`
as counterparts. The AIR-based logic should be consolidated there.

**Changes:**
- Delete `analysis/src/air/access_control.rs` and `analysis/src/air/arithmetic.rs`
- Remove `PassId::AIRAccessControl` and `PassId::AIRArithmetic` from `pass_id.rs`
- Merge AIR-specific detection logic into the existing
  `scanner/detectors/access_control.rs` and `scanner/detectors/arithmetic.rs`
  (or add new `scanner/detectors/air_access_control.rs` / `air_arithmetic.rs`
  if the implementations differ enough to warrant separation)
- Remove `bugs` from `[dependencies]` in `analysis/Cargo.toml` — after this step
  `analysis` should have **no dependency on `bugs`**

### 1.7 Group core infrastructure into `pass/` and `pipeline/`

The 8 top-level infrastructure files split cleanly into two concerns:

- **`pass/`** — defines what a pass *is*: the `Pass`/`AnalysisPass` traits, the
  `PassId` enum, and the `PassLevel`/`PassRepresentation` metadata.
- **`pipeline/`** — defines how passes are *run*: registration, scheduling,
  execution, and dependency resolution.

`context.rs` remains at the top level because it is the shared runtime state
consumed by both groups and by pass implementations.

**Changes:**
- Create `src/pass/mod.rs`, `src/pass/traits.rs`, `src/pass/id.rs`,
  `src/pass/meta.rs`
- Move content of `src/pass.rs` → `src/pass/traits.rs`
- Move content of `src/pass_id.rs` → `src/pass/id.rs`
- Move content of `src/pass_meta.rs` → `src/pass/meta.rs`
  (this step depends on 1.3 which creates `pass_meta.rs`)
- Create `src/pipeline/mod.rs`
- Move `src/manager.rs`, `src/scheduler.rs`, `src/executor.rs`,
  `src/dependency.rs` into `src/pipeline/`
- Update `src/lib.rs` module declarations
- Update all `use crate::pass::`, `use crate::pass_id::`, `use crate::pass_meta::`,
  `use crate::manager::`, `use crate::scheduler::`, `use crate::executor::`,
  `use crate::dependency::` paths throughout the crate

### 1.8 Delete `air/generation.rs` — lower SIR→AIR eagerly in `AnalysisContext`

`AIRGenerationPass::run()` is a single call to `mlir::air::lower::lower_module`.
It is not a real analysis pass — it is a mandatory infrastructure step that must
always run before any AIR pass. Modelling it as a pass adds artificial dependency
declarations (`PassId::AIRGeneration`) and forces the scheduler to detect a special
"IR generation point" in the execution schedule.

Instead, `AnalysisContext` should call `mlir::air::lower::lower_module` eagerly
when the SIR module is provided, making AIR immediately available to all passes
without any explicit dependency.

**Changes:**
- Delete `src/air/generation.rs`
- Remove `mod generation;` from `src/air/mod.rs`
- Remove `PassId::AIRGeneration` from `pass_id.rs`
- In `src/context.rs`, call `mlir::air::lower::lower_module` eagerly when
  constructing `AnalysisContext` (or when `set_ir_units` is called), populating
  `air_units` automatically
- Remove `dependencies: vec![PassId::AIRGeneration]` from all AIR passes
- Remove `needs_ir`, `ir_generation_level` special-casing from `scheduler.rs`

---

## Phase 2 — Structural / type-safety improvements (moderate effort)

### 2.1 Split `PassId` by representation layer

With `ast/` deleted, all remaining passes operate on SIR or AIR. Introduce
layered sub-enums to make cross-layer dependency violations a compile error:

```rust
// src/pass_id.rs
pub enum PassId {
    Sir(SirPassId),
    Air(AirPassId),
}

pub enum SirPassId { Cfg, MissingAccessControl, UncheckedArithmetic, TxOriginAuth, ... }
pub enum AirPassId { TaintPropagation, ... }
```

**Changes:**
- Rewrite `src/pass_id.rs` with the two sub-enums and the outer `PassId` wrapper
- Update all `PassId::*` usages in pass implementations, scheduler, and executor
- Add a helper `PassId::layer()` returning a `PassRepresentation` so the scheduler
  can validate cross-layer dependencies at registration time

### 2.2 Typed artifact keys (replace stringly-typed artifact storage)

`AnalysisContext::store_artifact(&str, T)` / `get_artifact::<T>(&str)` are
stringly-typed; a typo silently returns `None`. Replace with a marker-type key
pattern:

```rust
// Example definition (lives alongside the pass that produces the artifact)
pub struct CallGraphArtifact;
impl ArtifactKey for CallGraphArtifact {
    type Value = CallGraph;
    const NAME: &'static str = "call_graph"; // kept for serialisation / debug only
}

// Usage
ctx.store::<CallGraphArtifact>(graph);
ctx.get::<CallGraphArtifact>()   // returns Option<&CallGraph>
```

**Changes:**
- Add `ArtifactKey` trait to `src/context.rs` (or a new `src/artifact.rs`)
- Add typed `store<K: ArtifactKey>` / `get<K: ArtifactKey>` methods to `AnalysisContext`
- Keep the old stringly-typed methods behind a `#[deprecated]` gate during migration
- Migrate each pass to define its own `*Artifact` marker type and use the typed API
- Remove deprecated methods once all passes are migrated

---

## Phase 3 — Architecture / boundary clarification (lower urgency)

### 3.1 Clarify `manager` / `scheduler` / `executor` responsibilities

Add module-level doc comments to each file explaining what it owns and what it
does not do. If there is coupling that crosses the stated boundaries, refactor
it — but do not merge the three files unless the coupling turns out to be
fundamental.

Proposed boundary documentation:

- `manager.rs` — owns the pass registry; entry point for callers; delegates to
  scheduler then executor; produces `AnalysisReport`
- `scheduler.rs` — pure function: takes registered passes, returns
  `ExecutionSchedule`; must not mutate `AnalysisContext`
- `executor.rs` — takes `ExecutionSchedule` + `AnalysisContext`; drives execution
  and timing; must not mutate the pass registry

### 3.2 Consider `petgraph` for `dependency.rs`

If the dependency graph implementation is hand-rolled, evaluate replacing it with
`petgraph` (cycle detection, DFS/BFS, topological sort for free). Only do this if
it genuinely reduces code; if the current implementation is small and clear, leave
it.

---

## Phase 4 — New analysis infrastructure (implement after Phase 1 is stable)

### 4.1 Implement `frameworks/cfa/` — control-flow analysis utilities

CFA is a **utility collection** built on `petgraph`, not a parameterizable
framework like DFA. Each file is a standalone algorithm that produces a result
consumed by downstream passes or by `frameworks/dfa/`.

```
frameworks/cfa/
├── mod.rs
├── domtree.rs       # Lengauer-Tarjan over petgraph::DiGraph → DomTree, PostDomTree
├── loops.rs         # back-edge detection on DomTree → LoopInfo (header, body, exits)
├── reachability.rs  # BFS/DFS over AIR ICFG → ReachabilitySet
└── callgraph.rs     # scan SIR call-site ops → CallGraph (FunctionId nodes, call edges)
```

**Algorithm summary:**

| File | Input | Output | Algorithm |
|------|-------|--------|-----------|
| `domtree.rs` | AIR ICFG | `DomTree` / `PostDomTree` | Lengauer-Tarjan (DFS + union-find) |
| `loops.rs` | `DomTree` | `LoopInfo` | Back-edge identification + loop body walk |
| `reachability.rs` | AIR ICFG + start node | `ReachabilitySet` | BFS via `petgraph` |
| `callgraph.rs` | SIR `Module` | `CallGraph` | Structural scan of call-site ops |

**How CFA improves existing detectors:**

- **Access control**: Instead of checking whether *any* signer guard exists in the
  function body, use `DomTree` to verify the guard *dominates* every storage write.
  Eliminates false negatives from guards inside conditional branches.
- **Reentrancy**: Use `reachability.rs` to check whether an external call can
  *reach* a storage write on any path — a direct reachability query, no full DFA
  pass required.
- **Taint precision**: `LoopInfo` enables the DFA solver to process loop headers
  correctly and avoid redundant re-propagation over unreachable back-edges.

**Dependency:**
`dfa/annotated_cfg.rs` should be updated to accept a `DomTree` from `cfa/` when
available, enabling dominance-aware def-use annotation.

**Changes:**
- Add `petgraph` to `[dependencies]` in `analysis/Cargo.toml`
- Implement `src/frameworks/cfa/domtree.rs`, `loops.rs`, `reachability.rs`,
  `callgraph.rs`
- Add `pub mod cfa;` to `src/frameworks/mod.rs`
- Update `dfa/annotated_cfg.rs` to accept an optional `&DomTree`
- Update `passes/air/` detectors to query `DomTree` / `ReachabilitySet` from
  context instead of conservative approximations

### 4.2 Delete `passes/sir/cfg_pass.rs` once CFA is in place

`CfgPass` builds a SIR-level CFG that currently has no consumers (it is a stub).
Once `frameworks/cfa/` operates on the AIR ICFG (always available after step 1.8),
the SIR-level CFG pass becomes dead code.

**Changes:**
- Verify no pass reads the `CfgPass` artifact from `AnalysisContext`
- Delete `src/passes/sir/cfg_pass.rs`
- Remove `PassId::SirCfg` (or equivalent) from `pass/id.rs`
- Keep `mlir::sir::cfg` types in place — they may still be useful for future
  SIR-level analyses or tooling outside the pass pipeline

---

## Execution Order

| Phase | Step | Depends on |
|-------|------|------------|
| 1 | 1.1 Restructure into `passes/` + `frameworks/`; move `absint/` → `frameworks/dfa/`; move `sir/`, `air/` → `passes/sir/`, `passes/air/`; rename `taint.rs` → `taint_propagation.rs` | — |
| 1 | 1.2 Rename `absint/cfg.rs` → `frameworks/dfa/annotated_cfg.rs` | 1.1 |
| 1 | 1.3 Merge `pass_level` + `pass_representation` → `pass_meta` | — |
| 1 | 1.4 Move SIR CFG types + structural utils to `mlir` | — |
| 1 | 1.5 Delete `ast/`, remove `frontend` dep, fix `Loc` usages in dataflow passes | — |
| 1 | 1.6 Move `air/access_control.rs` + `air/arithmetic.rs` to `scanner`; drop `bugs` dep | 1.5 |
| 1 | 1.7 Group infrastructure into `pass/` + `pipeline/` sub-modules | 1.3 |
| 1 | 1.8 Delete `air/generation.rs`; lower SIR→AIR eagerly in `AnalysisContext` | 1.5 |
| 2 | 2.1 Split `PassId` into `Sir`/`Air` sub-enums | 1.3, 1.5, 1.6, 1.8 |
| 2 | 2.2 Typed artifact keys | — |
| 3 | 3.1 Document manager/scheduler/executor boundaries | 2.1 |
| 3 | 3.2 Evaluate `petgraph` for dependency graph | — |
| 4 | 4.1 Implement `frameworks/cfa/` (domtree, loops, reachability, callgraph) | 1.1, 1.8 |
| 4 | 4.2 Delete `passes/sir/cfg_pass.rs`; verify no consumers | 4.1 |

Phases 1 steps can be done in any order relative to each other and are safe to
batch in a single PR. Phase 2 steps are independent of each other but should come
after Phase 1 is merged. Phase 3 can be done incrementally. Phase 4 is new
implementation work and should follow Phase 1 stabilising.
