# Plan: Remove `frontend::solidity::analysis` Module

## Background

`crates/frontend/src/solidity/analysis/` is a dead-code pass infrastructure that was
superseded by the `crates/analysis/` crate. All production code (scanner pipeline,
detectors, CLI) imports from `analysis::`, not `frontend::solidity::analysis::`. The
only external reference is a thin test in `crates/scanner/tests/passes.rs` that is
mostly commented out.

## Goals

- Delete the `frontend/src/solidity/analysis/` directory and all its files.
- Remove the `pub mod analysis;` declaration from `frontend/src/solidity/mod.rs`.
- Remove or replace the dead test in `scanner/tests/passes.rs`.
- Ensure the project still compiles and all tests pass.

## Files to Delete

All files under:
```
crates/frontend/src/solidity/analysis/
  context.rs
  dependency.rs
  executor.rs
  manager.rs
  mod.rs
  pass.rs
  pass_id.rs
  pass_level.rs
  pass_representation.rs
  scheduler.rs
  passes/mod.rs
  passes/ast/mod.rs
  passes/ast/call_graph.rs
  passes/ast/inheritance_graph.rs
  passes/ast/modifier_analysis.rs
  passes/ast/symbol_table.rs
  passes/ast/type_index.rs
```

## Files to Edit

### `crates/frontend/src/solidity/mod.rs`

Remove the line:
```rust
pub mod analysis;
```

### `crates/scanner/tests/passes.rs`

Remove the file entirely, or replace its contents with a comment explaining that AST-level
pass tests now live in `crates/analysis/` (if any equivalent tests exist there).

## Verification Steps

1. `cargo build -p frontend` — must compile without errors.
2. `cargo build -p scanner` — must compile without errors.
3. `cargo test -p scanner` — all tests must pass.
4. `cargo test --workspace` — full test suite must pass.
5. Grep for any remaining references to `frontend::solidity::analysis` or
   `solidity::analysis` across the workspace to confirm none remain.

## Risks / Notes

- The `analysis` crate already provides equivalent pass infrastructure (PassManager,
  PassScheduler, PassExecutor, DependencyGraph) operating on IR rather than AST.
- The concrete AST-level artifacts (SymbolTable, CallGraph, InheritanceGraph, TypeIndex,
  ModifierAnalysis) are **not** replicated in `analysis/`. If any future feature needs
  AST-level analysis, those passes should be added to `analysis/src/passes/` targeting
  the Solidity AST, not resurrected in `frontend/`.
- No public API breakage expected since `frontend` is an internal crate.
