# Plan: Restructure Pass Modules

## Motivation

The current layout has two awkward sibling modules:

```
analysis/src/
  pass/       ← abstract infrastructure (traits, ids, metadata)
  passes/     ← concrete implementations (sir/, air/, vir/)
```

Problems:
- `pass` vs `passes` at the same level is confusing.
- `PassId` is a hand-maintained enum that must be kept in sync with actual `Pass` implementations — the compiler cannot enforce this.
- `SirPassId` and `AirPassId` sub-enums provide no real type safety and are used only internally via convenience aliases.
- `DetectionPassId` and its 17 variants live in the analysis crate, but the analysis pipeline never references them — they exist solely because `BugDetectionPass::id()` returns `PassId`.
- `pass/meta.rs` is duplicated as `pass_meta.rs` at the crate root.
- `context_new.rs` is an orphaned experimental file.

---

## Key Decisions

### 1. Replace `PassId` enum with `TypeId`

`PassId` is used as a key in `HashMap<PassId, Arc<dyn AnalysisPass>>`, for dependency tracking,
and for completion tracking in `AnalysisContext`. Every use is purely a **unique identifier** —
the enum variant values are never matched on for logic.

`std::any::TypeId` provides exactly this: a compiler-guaranteed unique ID per type, with zero
maintenance overhead. No enum to update when adding a new pass.

```rust
// Before
fn id(&self) -> PassId { PassId::SirCfg }
fn dependencies(&self) -> Vec<PassId> { vec![PassId::AirTaintPropagation] }

// After
fn id(&self) -> TypeId { TypeId::of::<Self>() }
fn dependencies(&self) -> Vec<TypeId> { vec![TypeId::of::<TaintPropagationPass>()] }
```

Pipeline internals (`DependencyGraph`, `PassManager`, `PassExecutor`, `PassScheduler`,
`AnalysisContext`) replace `PassId` with `TypeId` as the key type throughout.

For display and CLI pass selection, the existing `name() -> &'static str` on `Pass` is sufficient.
CLI lookup iterates registered passes and matches by name — no `From<&str>` on an enum needed.

**Result**: `pass/id.rs` and the `PassId` enum are deleted entirely.

### 2. Analysis crate does not need detection passes

Verified by grep: the analysis `pipeline/` has **zero references** to detection pass IDs.
`PassManager` never schedules or registers detection passes.

The only reason detection IDs were in the analysis crate was `BugDetectionPass::id() -> PassId`.
With `PassId` gone, the scanner defines its own `DetectorId` enum and `BugDetectionPass::id()`
returns `DetectorId`. The two crates are fully decoupled on this axis.

---

## Target Structure

### `analysis` crate

```
analysis/src/
  lib.rs
  context.rs
  pipeline/
    mod.rs
    manager.rs      ← HashMap<TypeId, Arc<dyn AnalysisPass>>
    scheduler.rs    ← HashSet<TypeId> for completed tracking
    executor.rs     ← TypeId keys
    dependency.rs   ← DependencyGraph<TypeId>
  passes/
    mod.rs              ← was passes/mod.rs; now also re-exports base/
    base/               ← NEW: was pass/ (the abstract infrastructure)
      mod.rs
      traits.rs         ← Pass, AnalysisPass, PassError, PassResult, PassExecutionInfo
      meta.rs           ← PassLevel, PassRepresentation
      ← id.rs DELETED
    sir/
      mod.rs
      cfg_pass.rs
    air/
      mod.rs
      taint_propagation.rs
    vir/
      mod.rs
  frameworks/
    ...                 ← unchanged
```

### `scanner` crate

```
scanner/src/
  lib.rs
  detector/             ← NEW: replaces scanner::pipeline::detector
    mod.rs
    id.rs               ← DetectorId enum (17 variants, owned entirely by scanner)
    traits.rs           ← BugDetectionPass trait with fn id(&self) -> DetectorId
  detectors/
    ...                 ← each detector's id() updated to return DetectorId
  pipeline/
    engine.rs
    registry.rs
    mod.rs
  ...
```

---

## Changes

### Step 1 — Move `pass/` → `passes/base/`

- Rename directory `analysis/src/pass/` → `analysis/src/passes/base/`.
- Update `analysis/src/passes/mod.rs` to add `pub mod base;`.
- Update `analysis/src/lib.rs`: drop `pub mod pass;`, update all re-export paths from `pass::` to `passes::base::`.
- Update all internal imports across the analysis crate from `crate::pass::` to `crate::passes::base::`.

### Step 2 — Replace `PassId` with `TypeId`; delete `pass/id.rs`

**In `passes/base/traits.rs`**:

```rust
use std::any::TypeId;

pub trait Pass: Send + Sync {
    fn id(&self) -> TypeId;           // was: fn id(&self) -> PassId
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn level(&self) -> PassLevel;
    fn representation(&self) -> PassRepresentation;
    fn dependencies(&self) -> Vec<TypeId>;   // was: Vec<PassId>
    fn invalidates(&self) -> Vec<TypeId> { vec![] }
    fn enabled_by_default(&self) -> bool { true }
}
```

**In each `Pass` impl** (currently `CfgPass` and `TaintPropagationPass`):
```rust
fn id(&self) -> TypeId { TypeId::of::<Self>() }
fn dependencies(&self) -> Vec<TypeId> { vec![] }
```

**In `pipeline/`**: replace all `PassId` with `TypeId` in:
- `DependencyGraph`: `HashMap<TypeId, HashSet<TypeId>>`, `HashSet<TypeId>`
- `PassManager`: `HashMap<TypeId, Arc<dyn AnalysisPass>>`
- `PassExecutor`: `HashMap<TypeId, Arc<dyn AnalysisPass>>`
- `PassScheduler`: `HashSet<TypeId>` for completion tracking

**In `context.rs`**: replace `HashSet<PassId>` with `HashSet<TypeId>` for `completed_passes`.

**In `lib.rs`**: remove re-exports of `PassId`, `SirPassId`, `AirPassId`, `DetectionPassId`.

**Delete**: `analysis/src/passes/base/id.rs` (was `pass/id.rs`).

**CLI pass selection**: replace `PassId::from(str)` with iterating `PassManager::registered_passes()`
and matching on `pass.name()`.

### Step 3 — Remove detection IDs from analysis; introduce `DetectorId` in scanner

**In the analysis crate**: nothing left to do after Step 2 — detection was only in `PassId`.

**In the scanner crate**:
- Create `scanner/src/detector/id.rs` with `DetectorId` enum (17 variants from `DetectionPassId`).
- Create `scanner/src/detector/traits.rs`: move `BugDetectionPass` from
  `scanner/src/pipeline/detector.rs`, change `fn id(&self) -> PassId` → `fn id(&self) -> DetectorId`.
- Create `scanner/src/detector/mod.rs` re-exporting both.
- Update all 28 detectors: `fn id() -> PassId` → `fn id() -> DetectorId`, update returned
  variant (e.g. `PassId::TxOrigin` → `DetectorId::TxOrigin`).
- Update `scanner/src/pipeline/registry.rs` and `engine.rs` to use `DetectorId`.
- Remove any scanner imports of `analysis::PassId` that were only needed for detector identity.

### Step 4 — Clean up orphaned files

- Delete `analysis/src/pass_meta.rs` — exact duplicate of `passes/base/meta.rs`.
- Evaluate `analysis/src/context_new.rs` — merge into `context.rs` or delete.

---

## File-level Summary

| Action | From | To |
|--------|------|----|
| Move dir | `analysis/src/pass/` | `analysis/src/passes/base/` |
| Delete | `analysis/src/passes/base/id.rs` (was `pass/id.rs`) | — |
| Replace type | `PassId` throughout `pipeline/` and `context.rs` | `std::any::TypeId` |
| Update trait sig | `Pass::id() -> PassId`, `dependencies() -> Vec<PassId>` | `TypeId` variants |
| Remove re-exports | `PassId`, `SirPassId`, `AirPassId`, `DetectionPassId` from `lib.rs` | — |
| Create | — | `scanner/src/detector/id.rs` (`DetectorId`) |
| Move + change sig | `BugDetectionPass` (`id() -> PassId`) | `scanner/src/detector/traits.rs` (`id() -> DetectorId`) |
| Update (×28) | each detector `fn id() -> PassId` | `fn id() -> DetectorId` |
| Delete | `analysis/src/pass_meta.rs` | — |
| Evaluate | `analysis/src/context_new.rs` | merge or delete |

---

## Confirmed

- Only `analysis` and `scanner` import pass ID types — `verifier`, `verazt`, `bugs` crates have
  no references.
- The analysis `pipeline/` has zero references to detection pass IDs — `PassManager` never
  schedules detection passes.

## Open Questions

1. Does `scanner/src/pipeline/registry.rs` or `engine.rs` use `PassId` only for detector
   identity, or also for cross-referencing analysis pass results? (Determines whether any
   `analysis::PassId` imports remain in the scanner after Step 3.)
