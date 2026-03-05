# Plan: Vyper Support in SmartHunt

## Overview

This plan describes extending `crates/smarthunt` to support Vyper contracts alongside
Solidity contracts. The goal is a full pipeline:

```
.vy file
  ↓ vyper -f ast
JSON AST
  ↓ vyper::parser
vyper::ast::SourceUnit
  ↓ normalize::run_passes
Normalized AST
  ↓ vyper::irgen
scir::Module
  ↓ smarthunt detectors
Bug reports
```

The `crates/vyper` crate already implements every step up to and including SCIR
generation (`vyper::compile_file(path) -> Result<scir::Module>`). What remains is
wiring smarthunt to invoke that crate and adapting the analysis/detection layer to
work with Vyper input.

---

## Current State

### What exists in `crates/vyper`

| Component | Status | Key entry points |
|---|---|---|
| Compiler invocation | Done | `parser::parse_input_file(path)` |
| JSON AST parser | Done | `AstParser::parse(json, path)` |
| Internal AST types | Done | `ast::{SourceUnit, FuncDef, …}` |
| Normalization passes | Done | `ast::normalize::run_passes(su)` |
| SCIR lowering | Done | `irgen::lower_source_unit(su)` |
| Public API | Done | `vyper::compile_file(path) -> Result<scir::Module>` |

### What smarthunt lacks

- No language detection (all input assumed Solidity)
- `Cargo.toml` does not depend on the `vyper` crate
- `AnalysisContext` holds `Vec<solidity::ast::SourceUnit>`; no SCIR path for detectors
- No `--language` CLI flag or auto-detection by file extension
- Detectors operate directly on `solidity::ast::*` types; they do not run against SCIR

---

## Design Decisions

### 1. Language detection strategy

Detect language from file extension at the CLI layer:
- `.sol` → Solidity pipeline (unchanged)
- `.vy` → Vyper pipeline

No mixed-language batches in a single `AnalysisContext` (same extension rule as today;
each `run_analysis` call handles one language). An optional `--language solidity|vyper`
flag can override auto-detection.

### 2. How detectors see Vyper input

Two categories of detectors exist today:

**AST-level GREP detectors** (11 detectors) — operate on `solidity::ast::SourceUnit`.
These must be ported to work on SCIR or reimplemented against `vyper::ast::SourceUnit`.
The recommended approach is an **SCIR-based matcher** so detectors become
language-neutral. This is a larger refactor; for an initial MVP the Vyper pipeline
can expose `vyper::ast::SourceUnit` to a parallel set of Vyper-specific GREP detectors.

**DFA detectors** (5 detectors) — already partly consume `scir::Module` (via
`ir_units` in `AnalysisContext`). These are best served by generating SCIR from Vyper
and inserting it into the existing IR slot. Because SCIR is language-neutral (EVM
dialect covers both Solidity and Vyper), many DFA checks can run unchanged.

**Recommended MVP scope:** wire Vyper → SCIR → existing DFA detectors; skip AST-level
GREP detectors for Vyper in the first iteration (they either don't apply or need
language-specific reimplementation).

### 3. Preserving Solidity compatibility

All existing code paths are preserved. The only changes are:
- New conditional branch in `main.rs` / `pipeline/engine.rs` for `.vy` files
- New `InputLanguage` enum threaded through config
- `AnalysisContext` gains an `input_language` field; detector logic gates on it

---

## Changes to `crates/vyper`

These changes are prerequisites for the smarthunt integration. They belong in the
vyper crate itself, not in smarthunt.

### Vyper A — Activate pragma-based version selection in `parser/mod.rs`

`extract_version_pragma()` already parses `# @version ^0.3.9` from source but its
result is currently only logged and then discarded. This step makes it drive actual
compiler selection, mirroring `solc-select` in the Solidity crate.

**New constant:**

```rust
const VYPER_SELECT: &str = "vyper-select";
```

**New helper — list installed Vyper versions:**

```rust
fn get_installed_vyper_vers() -> Vec<String> {
    // vyper-select versions  →  prints one version per line
    let output = Command::new(VYPER_SELECT).arg("versions").output();
    // parse output lines, filter semver strings
}
```

**New helper — configure the active Vyper version:**

```rust
pub fn configure_vyper_compiler(ver: &Version) {
    // 1. Check whether `ver` is already installed:
    //    vyper-select versions  (look for ver in output)
    // 2. If not: vyper-select install <ver>
    // 3. Activate: vyper-select use <ver>
}
```

**New helper — find a compatible Vyper version:**

```rust
pub fn find_compatible_vyper_versions(
    pragma_ver: &Option<String>,
) -> Result<Vec<Version>> {
    // Enumerate get_installed_vyper_vers(), filter by pragma_ver semver range.
    // Fall back to the latest installed version if no pragma given.
    // Return error if no compatible version exists.
}
```

**Update `parse_input_file` signature and body:**

```rust
pub fn parse_input_file(
    input_file: &str,
    vyper_ver: Option<&str>,   // NEW: explicit version override (from CLI)
) -> Result<SourceUnit> {
    // 1. Read source, call extract_version_pragma()
    // 2. If pragma found, parse it as a semver range; validate it is a known
    //    Vyper generation (>= 0.2.0)
    // 3. Resolve compiler version:
    //    a. If vyper_ver override given, parse as range and intersect with pragma
    //    b. Else use pragma range alone
    //    c. Call find_compatible_vyper_versions() to pick the best match
    // 4. Call configure_vyper_compiler(resolved_ver) to activate it
    // 5. Invoke the compiler as before
}
```

**Update `parse_vyper_source_code` to forward the version:**

```rust
pub fn parse_vyper_source_code(
    source_code: &str,
    vyper_ver: Option<&str>,   // NEW
) -> Result<SourceUnit>
```

**Update public API in `lib.rs`:**

```rust
pub fn compile_file(input_file: &str, vyper_ver: Option<&str>) -> Result<scir::Module>
pub fn compile_source(source_code: &str, vyper_ver: Option<&str>) -> Result<scir::Module>
```

### Vyper B — Version validation

Mirror the Solidity minimum-version guard:

```rust
// In parse_input_file, after parsing the pragma range:
if !check_range_constraint(&pragma_range, ">=0.2.0") {
    fail!("Only Vyper versions >= 0.2.0 are supported, but found: {}", ver);
}
```

Minimum supported version is `0.2.0` (first stable release with the JSON AST output
format used by `-f ast`).

---

## Implementation Steps (smarthunt)

### Step 1 — Add `vyper` dependency to smarthunt

File: `crates/smarthunt/Cargo.toml`

```toml
[dependencies]
vyper = { path = "../vyper" }
```

### Step 2 — Add `InputLanguage` enum to config

File: `crates/smarthunt/src/config.rs`

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum InputLanguage {
    #[default]
    Solidity,
    Vyper,
}
```

Add `pub input_language: InputLanguage` to `AnalysisConfig`.

### Step 3 — Language detection in CLI

File: `crates/smarthunt/src/main.rs`

```rust
// new CLI flag
#[arg(long, value_enum)]
pub language: Option<InputLanguage>,
```

Detection logic (in `run_analysis`):

```rust
fn detect_language(file: &str, override_lang: Option<InputLanguage>) -> InputLanguage {
    if let Some(lang) = override_lang {
        return lang;
    }
    if file.ends_with(".vy") {
        InputLanguage::Vyper
    } else {
        InputLanguage::Solidity
    }
}
```

All input files must share the same detected language (emit an error if mixed).

### Step 4 — Vyper compilation path in `run_analysis`

File: `crates/smarthunt/src/main.rs`

Replace the current single-path loop:

```rust
// Before (Solidity only):
let source_units = solidity::parser::parse_input_file(file, ...)?;

// After:
match detect_language(file, args.language) {
    InputLanguage::Solidity => {
        let sus = solidity::parser::parse_input_file(file, base_path, ...)?;
        sol_source_units.extend(sus);
    }
    InputLanguage::Vyper => {
        let module = vyper::compile_file(file)?;
        ir_units.push(module);
    }
}
```

For Vyper, skip the Solidity AST path entirely; SCIR modules go directly into
`context.ir_units`.

### Step 5 — Thread SCIR into `AnalysisContext` for Vyper

File: `crates/smarthunt/src/analysis/context.rs`

`AnalysisContext` already has `pub ir_units: Option<Vec<scir::Module>>`. For Vyper
input, populate this field directly from `vyper::compile_file` results instead of
generating IR from Solidity ASTs.

Add `pub input_language: InputLanguage` so passes and detectors can gate on language.

### Step 6 — Gate existing passes on language

File: `crates/smarthunt/src/analysis/manager.rs` (and individual pass files)

AST-level passes (`SymbolTablePass`, `CallGraphPass`, `InheritanceGraphPass`,
`ModifierAnalysisPass`) operate on `solidity::ast::SourceUnit`. They must be skipped
when `context.input_language == Vyper`. The `PassManager` should check
`input_language` before scheduling AST passes.

DFA passes that consume `scir::Module` (`CfgPass`, and downstream DFA detectors) can
run unchanged because SCIR is language-neutral.

### Step 7 — DFA detectors: run against Vyper SCIR

File: `crates/smarthunt/src/dfa/detectors/*.rs`

These detectors already retrieve `context.ir_units`. No code change is required
provided the context is populated correctly in Step 4/5. Verify each DFA detector
compiles and produces results against a Vyper SCIR module.

Detectors expected to work out of the box:
- `ReentrancyDfaDetector` — CEI violation is language-independent
- `UncheckedCallDfaDetector` — `raw_call` is lowered to `EvmExpr::RawCall` in SCIR
- `CeiViolationDfaDetector` — state mutation ordering is language-neutral

Detectors that may need Vyper-specific attention:
- `UninitializedDfaDetector` — Vyper initializes all variables to zero; may produce
  false positives. Add a guard: skip if `source_lang == "vyper"`.
- `DeadCodeDfaDetector` — verify that Vyper's `pass` statement is not flagged.

### Step 8 — GREP detectors: disable for Vyper (MVP)

File: `crates/smarthunt/src/pipeline/engine.rs`

In `resolve_detectors()`, filter out GREP-based detectors when
`config.input_language == Vyper`:

```rust
if config.input_language == InputLanguage::Vyper {
    detectors.retain(|d| d.representation() != PassRepresentation::Ast);
}
```

This is a safe MVP choice. Vyper-specific GREP detectors can be added in a follow-up.

### Step 9 — Output: tag bugs with source language

File: `crates/smarthunt/src/output/{json,markdown,sarif}.rs`

When formatting bugs, include `"source_lang": "vyper"` (already present as a SCIR
module attribute). This helps downstream consumers distinguish Solidity vs. Vyper
findings.

### Step 10 — Tests

Add integration tests in `crates/smarthunt/tests/`:

```
tests/
  vyper_integration.rs   # compile examples/vyper/token.vy, vault.vy, vault_buggy.vy
                         # assert no panic, assert expected bugs detected in vault_buggy.vy
```

Minimum test cases:
1. `token.vy` — clean contract, expect 0 high-severity bugs
2. `vault_buggy.vy` — intentionally buggy, expect reentrancy detected
3. CLI smoke test: `smarthunt vault_buggy.vy --format json` exits 0

### Step 11 — Update README.md

File: `README.md`

Add a **Prerequisites** entry for the Vyper compiler (parallel to the existing
`solc-select` entry), and a **Usage** section showing how to run smarthunt against
Vyper contracts.

**Prerequisites addition:**

```markdown
## Install vyper-select

We use `vyper-select` to manage and switch between Vyper compiler versions,
analogous to `solc-select` for Solidity.

```bash
pip install vyper-select
```

Install the required Vyper version(s):

```bash
# Install a specific version
vyper-select install 0.3.10

# Or install all available versions
vyper-select install all
```

Verify the installation:

```bash
vyper-select versions
vyper --version
```
```

**Usage section (new or extended):**

```markdown
# Usage

## Analyzing Solidity contracts

```bash
smarthunt path/to/contract.sol
```

## Analyzing Vyper contracts

```bash
smarthunt path/to/contract.vy
```

The language is detected automatically from the file extension (`.vy` → Vyper,
`.sol` → Solidity). Use `--language` to override:

```bash
smarthunt --language vyper path/to/contract.vy
```

## Common options

```bash
# JSON output
smarthunt contract.vy --format json --output report.json

# Run detectors in parallel
smarthunt contract.vy --parallel

# Enable or disable specific detectors
smarthunt contract.vy --enable reentrancy,unchecked-call
smarthunt contract.vy --disable dead-code

# List all available detectors
smarthunt list-detectors
```
```

**Notes to add:**

- Note that only DFA-based detectors run on Vyper in the initial release; AST-level
  pattern detectors are Solidity-only and are silently skipped for `.vy` input.
- Reference the `examples/vyper/` directory as sample contracts.

---

## File-Level Change Summary

**`crates/vyper` (prerequisites for smarthunt integration):**

| File | Change |
|---|---|
| `crates/vyper/src/parser/mod.rs` | Add `configure_vyper_compiler`, `find_compatible_vyper_versions`, `get_installed_vyper_vers`; update `parse_input_file` and `parse_vyper_source_code` signatures to accept `vyper_ver: Option<&str>`; activate pragma-based version selection |
| `crates/vyper/src/lib.rs` | Update `compile_file` and `compile_source` signatures to forward `vyper_ver` |

**`crates/smarthunt`:**

| File | Change |
|---|---|
| `crates/smarthunt/Cargo.toml` | Add `vyper` path dependency |
| `crates/smarthunt/src/config.rs` | Add `InputLanguage` enum + field in `AnalysisConfig` |
| `crates/smarthunt/src/main.rs` | Add `--language` and `--vyper-version` flags; branch on language in `run_analysis`; pass `vyper_ver` to `vyper::compile_file` |
| `crates/smarthunt/src/analysis/context.rs` | Add `input_language` field; populate `ir_units` for Vyper |
| `crates/smarthunt/src/analysis/manager.rs` | Skip AST passes when `input_language == Vyper` |
| `crates/smarthunt/src/pipeline/engine.rs` | Filter GREP detectors for Vyper in MVP |
| `crates/smarthunt/src/dfa/detectors/uninitialized.rs` | Guard against Vyper false positives |
| `crates/smarthunt/src/dfa/detectors/dead_code.rs` | Verify `pass` statement handling |
| `crates/smarthunt/tests/vyper_integration.rs` | New integration tests |
| `README.md` | Add `vyper-select` prerequisite and Vyper usage examples |

**No changes required in:** `crates/scir/`, `crates/solidity/`

---

## What Is Out of Scope (Follow-Up Work)

### Vyper-specific detectors

The MVP disables all GREP detectors for Vyper. Follow-up work should port or rewrite
detectors that are relevant to Vyper:

- **Reentrancy guard missing** — Vyper has `@nonreentrant` decorator; detect functions
  that perform external calls without it
- **Floating pragma** — detect `# @version` pragmas that allow a wide version range
  (e.g., `>=0.3.0`) rather than pinning to a specific version
- **`default_return_value` misuse** — Vyper's `raw_call` with `default_return_value`
  silently swallows failures; detect unchecked usage
- **Integer overflow in Vyper <0.3.0** — versions before 0.3.0 lacked built-in
  overflow protection; flag contracts with an old pragma
- **Timestamp dependence** — already detected for Solidity; port to Vyper SCIR
  (`EvmExpr::Timestamp` is emitted by both lowerers)

The long-term solution is to migrate GREP detectors to operate on SCIR rather than
`solidity::ast`, making them language-neutral automatically.

### Vyper AST-level analysis passes

`SymbolTablePass`, `CallGraphPass`, `InheritanceGraphPass`, and `ModifierAnalysisPass`
all operate on `solidity::ast::SourceUnit`. To run them on Vyper contracts requires
one of:

- Abstracting the pass trait over a generic AST visitor
- Implementing Vyper-specific variants of each pass against `vyper::ast`
- Migrating all analysis to SCIR (eliminating AST-level passes entirely)

Vyper's simpler model (no inheritance, no modifiers, no imports in ≥0.3.x) means
some passes can be simplified or eliminated rather than ported.

### Multi-file Vyper analysis

Vyper 0.3.x does not support imports, so each `.vy` file is a self-contained
contract. Future Vyper versions may add module imports; smarthunt should be extended
to accept multiple `.vy` files and resolve cross-file references when that happens.

### Mixed Solidity + Vyper analysis

Supporting a single `smarthunt` invocation with both `.sol` and `.vy` files would
require a shared analysis context holding both AST types. Currently blocked by
detectors being typed to one language's AST.

### Vyper version coverage

The minimum supported version is set to `0.2.0`. Contracts using `0.1.x` or earlier
(which used a different AST format) are out of scope.

---

## Prerequisites

- `vyper-select` installed: `pip install vyper-select`
- At least one Vyper version installed via `vyper-select install <version>`
- `crates/vyper` compiles cleanly: `cargo build -p vyper`
- Example contracts in `examples/vyper/` are valid Vyper 0.2.0 or later
