# Plan: `verazt compile` Command

## Goal

Add a `verazt compile` subcommand to the single unified `verazt` binary that parses a smart
contract source file and can print its AST, SIR, AIR, and VIR representations. Remove the
standalone `soljc` and `vypc` binaries from `crates/langs`.

---

## Background

### Current State

- `crates/cli/src/main.rs` — the `verazt` binary, currently with two subcommands: `analyzer`
  and `verifier`.
- `crates/langs/src/solidity/main.rs` — standalone `soljc` binary (Solidity → SIR pipeline).
- `crates/langs/src/vyper/main.rs` — standalone `vypc` binary (Vyper → SIR pipeline).

### IR Layers

| Acronym | Full Name | Location |
|---------|-----------|----------|
| AST | Abstract Syntax Tree | `crates/langs/src/{solidity,vyper}/ast/` |
| SIR | Source IR | `crates/mlir/src/sir/` |
| AIR | Analysis IR (SSA/CFG) | `crates/mlir/src/air/` |
| VIR | Verification IR | `crates/mlir/src/vir/` *(planned, not yet implemented)* |

### Compilation Pipeline

```
Input File (.sol / .vy)
    │
    ▼
Parse  →  AST (SourceUnit)
    │
    ▼
Normalize  →  normalized AST
    │
    ▼
irgen::lower_source_unit()  →  SIR (Module)
    │
    ▼
air::lower::lower_module()  →  AIR
    │
    ▼
vir::lower::lower_module()  →  VIR  [future]
```

---

## Changes Required

### 1. Remove Standalone Binaries from `crates/langs`

- **Delete** `crates/langs/src/solidity/main.rs`
- **Delete** `crates/langs/src/vyper/main.rs`
- **Edit** `crates/langs/Cargo.toml`:
  - Remove the `[[bin]]` entries for `soljc` and `vypc`.
  - The `langs` crate becomes a library-only crate (keep `[lib]`).

### 2. Add `compile` Module to `crates/cli`

Create `crates/cli/src/compile/mod.rs` (or `crates/cli/src/compile.rs`) with:

```rust
pub struct Args {
    /// Input smart contract file(s) (.sol or .vy).
    pub input_files: Vec<String>,

    /// Explicit language override (solidity | vyper). Auto-detected from extension if omitted.
    #[arg(long)]
    pub language: Option<Language>,

    // Solidity-specific options (ignored for Vyper)
    #[arg(long)]
    pub base_path: Option<String>,
    #[arg(long)]
    pub include_path: Vec<String>,
    #[arg(long)]
    pub solc_version: Option<String>,

    /// Print the parsed AST.
    #[arg(long, visible_alias = "pip")]
    pub print_ast: bool,

    /// Print the Source IR (SIR).
    #[arg(long, visible_alias = "pir")]
    pub print_sir: bool,

    /// Print the Analysis IR (AIR).
    #[arg(long)]
    pub print_air: bool,

    /// Print the Verification IR (VIR).
    #[arg(long)]
    pub print_vir: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> { ... }
```

Language auto-detection logic:
- `.sol` → Solidity
- `.vy` → Vyper
- Otherwise → error unless `--language` is provided

### 3. Register `compile` Subcommand in `crates/cli/src/main.rs`

```rust
enum Commands {
    Compile(compile::Args),   // new
    Analyzer { args: Vec<String> },
    Verifier { args: Vec<String> },
}
```

Dispatch: `Commands::Compile(args) => compile::run(args)?`

### 4. Wire Up the Pipeline in `compile::run`

Per input file:

1. **Parse** — call `langs::solidity::parser::parse_input_file(...)` or
   `langs::vyper::parser::parse_input_file(...)` depending on language.
2. **Normalize** — call the respective `normalize::run_passes(...)`.
3. If `--print-ast` → pretty-print the normalized AST.
4. **Lower to SIR** — call `langs::{solidity,vyper}::irgen::lower_source_unit(...)`.
5. If `--print-sir` → call `module.print_pretty()`.
6. **Lower to AIR** — call `air::lower::lower_module(...)` (already used by analyzer).
7. If `--print-air` → call `air_module.print_pretty()` (add `print_pretty` to AIR if missing).
8. **Lower to VIR** *(future)* — once `vir` crate exists, call its lowering function.
9. If `--print-vir` → placeholder error or print VIR once implemented.

### 5. Update `crates/cli/Cargo.toml` Dependencies

Add `langs` as a direct dependency if not already present (it may already be transitive through
`analyzer`/`verifier`). Also add `mlir` for direct AIR/VIR access.

---

## File Change Summary

| Action | File |
|--------|------|
| Delete | `crates/langs/src/solidity/main.rs` |
| Delete | `crates/langs/src/vyper/main.rs` |
| Edit | `crates/langs/Cargo.toml` — remove `[[bin]]` sections |
| Create | `crates/cli/src/compile.rs` (or `compile/mod.rs`) |
| Edit | `crates/cli/src/main.rs` — add `Compile` variant and dispatch |
| Edit | `crates/cli/Cargo.toml` — add `langs` and `mlir` deps if missing |

---

## Out of Scope for This Plan

- Implementing VIR lowering (the `--print-vir` flag should return a clear "not yet implemented"
  error until the VIR layer exists).
- Changing the `analyzer` or `verifier` subcommand internals.
- Adding JSON/machine-readable output formats.
