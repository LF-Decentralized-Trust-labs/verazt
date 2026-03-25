# Plan: Rename `scanner` to `analyzer` and Merge `analysis` Into It

## Goal

1. Rename the `scanner` crate to `analyzer`
2. Absorb the `analysis` crate into `analyzer` as a nested submodule (`analyzer::analysis`)

---

## Current State

```
crates/
├── analysis/src/          ← standalone framework crate
│   ├── lib.rs
│   ├── context.rs
│   ├── pipeline/
│   ├── passes/
│   └── frameworks/
└── scanner/src/           ← vulnerability detection tool
    ├── lib.rs
    ├── cli.rs
    ├── config.rs
    ├── artifacts.rs
    ├── detector/
    ├── detectors/
    ├── engines/
    ├── pipeline/
    └── output/
```

`scanner` depends on `analysis` and re-exports a subset of it via `pub use analysis::...`.

---

## Target Structure

### Module grouping rationale

- `analysis/` stays as a **nested submodule** — it is a reusable framework (passes, pipeline
  orchestration, DFA/CFA) that is conceptually separate from the detection tool.
  Flattening it to the root would mix unrelated concerns at the same level.

- `detector/` and `detectors/` are **merged into one `detectors/` module** — `detector/`
  only contains infrastructure (`id.rs`, `traits.rs`) that belongs alongside the concrete
  detector implementations. Having two near-identical names for a single concept is confusing.

- `engines/` is **dissolved into `analysis/frameworks/`** — both engines are reusable
  analysis techniques, not bug detectors:
  - `engines/pattern/` → `analysis/frameworks/grep/` — renamed to `grep` to match the
    project's established terminology ("GREP detectors" in docs and comments).
  - `engines/datalog/` → `analysis/frameworks/datalog/` — relational reasoning framework.
  - `engines/` directory is then deleted.
  - This gives `analysis/frameworks/` four consistent technique-named modules:
    `dfa/`, `cfa/`, `grep/`, `datalog/`.

- `pipeline/` is **split**:
  - `pipeline/registry.rs` → `detectors/registry.rs` — purely about detector registration.
  - `pipeline/engine.rs` → `pipeline.rs` — top-level orchestrator (single file module).
  - `pipeline/` directory is then deleted.

```
crates/
└── analyzer/src/
    ├── lib.rs
    ├── cli.rs
    ├── config.rs
    ├── artifacts.rs
    ├── pipeline.rs                ← was pipeline/engine.rs (top-level orchestrator)
    ├── analysis/                  ← moved from crates/analysis/src/
    │   ├── mod.rs
    │   ├── context.rs
    │   ├── pipeline/              ← PassManager, scheduler, executor (unchanged)
    │   ├── passes/
    │   └── frameworks/
    │       ├── dfa/
    │       ├── cfa/
    │       ├── grep/              ← moved + renamed from engines/pattern/
    │       │   ├── mod.rs
    │       │   ├── builder.rs
    │       │   ├── composite.rs
    │       │   ├── core.rs
    │       │   ├── matcher.rs
    │       │   └── primitives.rs
    │       └── datalog/           ← moved from engines/datalog/
    ├── detectors/                 ← merged detector/ + detectors/ + registry
    │   ├── mod.rs
    │   ├── id.rs                  (from detector/)
    │   ├── traits.rs              (from detector/)
    │   ├── registry.rs            (from pipeline/registry.rs)
    │   ├── reentrancy.rs
    │   ├── arithmetic.rs
    │   ├── access_control.rs
    │   └── … (all other concrete detectors)
    └── output/
```

Public API from `analyzer::lib.rs` remains unchanged — callers still access
`analyzer::PassManager`, `analyzer::AnalysisContext`, `analyzer::BugDetectionPass`, etc.
via top-level re-exports.

---

## Steps

### Phase 1: Rename `scanner` → `analyzer`

1. Rename directory: `crates/scanner/` → `crates/analyzer/`
2. `crates/analyzer/Cargo.toml`: `name = "scanner"` → `name = "analyzer"`
3. Root `Cargo.toml`:
   - `"crates/scanner"` → `"crates/analyzer"` in `[workspace] members`
4. `crates/verazt/Cargo.toml`: dep `scanner` → `analyzer = { path = "../analyzer" }`
5. `crates/verazt/src/main.rs`: `scanner::cli::run(...)` → `analyzer::cli::run(...)`
6. `crates/analyzer/tests/**/*.rs`: all `use scanner::` → `use analyzer::`

### Phase 2: Merge `detector/` + `engines/` + `pipeline/` into `detectors/`

**Merge `detector/` into `detectors/`:**

1. Move `detector/id.rs` → `detectors/id.rs`
2. Move `detector/traits.rs` → `detectors/traits.rs`
3. Delete `detector/` directory
4. Update `detectors/mod.rs`: add `pub mod id; pub mod traits;`

**Move `engines/` into `analysis/frameworks/` (done after Phase 3 move):**

5. Move `engines/pattern/` → `analysis/frameworks/grep/` (rename to match project terminology)
6. Move `engines/datalog/` → `analysis/frameworks/datalog/`
7. Delete `engines/` directory
8. Update `analysis/frameworks/mod.rs`: add `pub mod grep; pub mod datalog;`
9. Fix internal imports: `use crate::engines::pattern::` → `use crate::analysis::frameworks::grep::`

**Move `pipeline/registry.rs` into `detectors/`:**

10. Move `pipeline/registry.rs` → `detectors/registry.rs`
11. Update `detectors/mod.rs`: add `pub mod registry;`
12. Fix imports: `use crate::pipeline::registry::` → `use crate::detectors::registry::`

**Collapse `pipeline/` to single file:**

13. Rename `pipeline/engine.rs` → `pipeline.rs` (top-level single file module)
14. Delete `pipeline/mod.rs` and `pipeline/` directory
15. Update `pipeline.rs` internal imports (registry now in `crate::detectors::registry`)

**Update `lib.rs`:**

16. Remove `pub mod detector;` and `pub mod engines;`
17. Change `pub mod pipeline;` — now points to `pipeline.rs` (automatic)
18. Update re-exports:
    - `pub use detector::{...}` → `pub use detectors::{...}`
    - `pub use pipeline::{DetectorRegistry, register_all_detectors}` → `pub use detectors::registry::{...}`
19. Fix all remaining `use crate::detector::` / `use crate::engines::` references

### Phase 3: Merge `analysis` into `analyzer`

1. Move `crates/analysis/src/` → `crates/analyzer/src/analysis/`
   - Rename `lib.rs` → `mod.rs` (Rust submodule convention)
   - All internal `crate::` paths in the moved files remain valid as-is
2. `crates/analyzer/src/lib.rs`:
   - Add `pub mod analysis;`
   - Replace `pub use analysis::{...}` (external crate) → `pub use crate::analysis::{...}`
   - The `pass` compatibility alias in `analysis/mod.rs` is preserved as-is
3. `crates/analyzer/src/**/*.rs` (detectors, pipeline, engines, etc.):
   - `use analysis::` → `use crate::analysis::`
4. `crates/analyzer/Cargo.toml`: remove `analysis = { path = "../analysis" }` dependency
5. Root `Cargo.toml`: remove `"crates/analysis"` from `[workspace] members`
6. `crates/verifier/Cargo.toml` (if it depends on `analysis`):
   - Update to `analyzer = { path = "../analyzer" }` and use `analyzer::analysis::...`
   - Delete `crates/analysis/` directory after all references are gone

### Phase 4: Verification

1. `cargo build` — fix any remaining path/import errors
2. `cargo test` — all tests pass
3. Manual smoke test: `verazt check <contract>` runs end-to-end

---

## Files Affected

| Path | Change |
|------|--------|
| **Phase 1** | |
| `crates/scanner/` | Rename → `crates/analyzer/` |
| `crates/analyzer/Cargo.toml` | Rename crate |
| `Cargo.toml` | Update workspace members |
| `crates/verazt/Cargo.toml` | `scanner` dep → `analyzer` |
| `crates/verazt/src/main.rs` | Update crate ref and CLI invocation |
| `crates/analyzer/tests/**/*.rs` | `use scanner::` → `use analyzer::` |
| **Phase 2** | |
| `src/detector/` | Deleted — `id.rs`, `traits.rs` moved into `detectors/` |
| `src/detectors/mod.rs` | Add `pub mod id, traits, registry;` |
| `src/engines/pattern/` | Moved + renamed → `src/analysis/frameworks/grep/` (after Phase 3) |
| `src/engines/datalog/` | Moved → `src/analysis/frameworks/datalog/` (after Phase 3) |
| `src/engines/` | Deleted |
| `src/pipeline/registry.rs` | Moved → `src/detectors/registry.rs` |
| `src/pipeline/engine.rs` | Moved → `src/pipeline.rs` (single file module) |
| `src/pipeline/` | Deleted |
| `src/lib.rs` | Remove `mod detector`, `mod engines`; update all re-exports |
| `src/**/*.rs` | Fix `crate::detector::`, `crate::engines::`, `crate::pipeline::registry::` imports |
| **Phase 3** | |
| `crates/analysis/src/` | Moved → `src/analysis/`; `lib.rs` → `mod.rs` |
| `src/analysis/frameworks/mod.rs` | Add `pub mod grep; pub mod datalog;` |
| `crates/analyzer/Cargo.toml` | Remove `analysis` dep |
| `Cargo.toml` | Remove `crates/analysis` from members |
| `src/lib.rs` | Add `pub mod analysis;`; fix re-exports to `crate::analysis::` |
| `src/**/*.rs` | `use analysis::` → `use crate::analysis::` |
| `crates/analysis/` | Deleted |
| `crates/verifier/Cargo.toml` | Update `analysis` dep → `analyzer` (if needed) |

---

## Checklist

### Phase 1: Rename `scanner` → `analyzer`

- [x] Rename directory `crates/scanner/` → `crates/analyzer/`
- [x] `crates/analyzer/Cargo.toml`: change `name = "scanner"` → `name = "analyzer"`
- [x] Root `Cargo.toml`: replace `"crates/scanner"` → `"crates/analyzer"` in `[workspace] members`
- [x] `crates/verazt/Cargo.toml`: replace dep `scanner` → `analyzer = { path = "../analyzer" }`
- [x] `crates/verazt/src/main.rs`: replace `scanner::cli::run(...)` → `analyzer::cli::run(...)`
- [x] `crates/analyzer/tests/**/*.rs`: replace all `use scanner::` → `use analyzer::`

### Phase 2: Restructure `analyzer` modules

**Merge `detector/` into `detectors/`:**

- [x] Move `src/detector/id.rs` → `src/detectors/id.rs`
- [x] Move `src/detector/traits.rs` → `src/detectors/traits.rs`
- [x] Delete `src/detector/` directory
- [x] `src/detectors/mod.rs`: add `pub mod id; pub mod traits;`

**Move `pipeline/registry.rs` into `detectors/`:**

- [x] Move `src/pipeline/registry.rs` → `src/detectors/registry.rs`
- [x] `src/detectors/mod.rs`: add `pub mod registry;`
- [x] Fix imports: `use crate::pipeline::registry::` → `use crate::detectors::registry::`

**Collapse `pipeline/` to single file:**

- [x] Move `src/pipeline/engine.rs` → `src/pipeline.rs`
- [x] Delete `src/pipeline/mod.rs` and `src/pipeline/` directory
- [x] Fix `pipeline.rs` internal imports (registry now at `crate::detectors::registry`)

**Update `src/lib.rs`:**

- [x] Remove `pub mod detector;` and `pub mod engines;`
- [x] `pub mod pipeline;` now resolves to `pipeline.rs` — verify no change needed
- [x] `pub use detector::{...}` → `pub use detectors::{BugDetectionPass, ConfidenceLevel, DetectorId, DetectorResult, create_bug, create_bug_with_details}`
- [x] `pub use pipeline::{DetectorRegistry, register_all_detectors}` → `pub use detectors::registry::{...}`
- [x] Fix all remaining `use crate::detector::` and `use crate::engines::` references across `src/`

### Phase 3: Merge `analysis` crate into `analyzer`

- [x] Move `crates/analysis/src/` → `crates/analyzer/src/analysis/`
- [x] Rename `analysis/lib.rs` → `analysis/mod.rs`
- [x] Move `src/engines/pattern/` → `src/analysis/frameworks/grep/`
- [x] Move `src/engines/datalog/` → `src/analysis/frameworks/datalog/`
- [x] Delete `src/engines/` directory
- [x] `src/analysis/frameworks/mod.rs`: add `pub mod grep; pub mod datalog;`
- [x] Fix internal `grep` imports: `use crate::engines::pattern::` → `use crate::analysis::frameworks::grep::`
- [x] `src/lib.rs`: add `pub mod analysis;`
- [x] `src/lib.rs`: replace all `pub use analysis::{...}` → `pub use crate::analysis::{...}`
- [x] `src/**/*.rs`: replace `use analysis::` → `use crate::analysis::` throughout
- [x] `crates/analyzer/Cargo.toml`: remove `analysis = { path = "../analysis" }` dependency
- [x] Root `Cargo.toml`: remove `"crates/analysis"` from `[workspace] members`
- [x] Check `crates/verifier/Cargo.toml` for `analysis` dependency — update to `analyzer` if present
- [x] Delete `crates/analysis/` directory

### Phase 4: Verification

- [x] `cargo build` — no errors
- [x] `cargo test` — all tests pass (21 pre-existing frontend parser failures unrelated to this refactoring)
- [ ] Manual smoke test: `verazt check <contract>` runs end-to-end

---

## Open Questions

- Should the CLI subcommand remain `check` or be renamed to `analyze`?
- Does `verifier` use `analysis` directly? If yes, it should depend on `analyzer` and access `analyzer::analysis::*`.
