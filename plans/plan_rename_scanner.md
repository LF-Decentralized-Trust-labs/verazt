# Plan: Rename `checker` Crate to `scanner`

## Overview

Rename the `checker` crate (currently at `crates/checker/`) to `scanner` to better
reflect its purpose as a security vulnerability scanner for smart contracts.

---

## Step 1: Rename the Directory

```
crates/checker/  →  crates/scanner/
```

---

## Step 2: Update `Cargo.toml` Files

### `crates/scanner/Cargo.toml` (was `crates/checker/Cargo.toml`)
- `name = "checker"` → `name = "scanner"`

### `Cargo.toml` (workspace root)
- Members list: `"crates/checker"` → `"crates/scanner"`
- Workspace deps: `checker = { path = "crates/checker" }` → `scanner = { path = "crates/scanner" }`

### `crates/verazt/Cargo.toml`
- `checker = { workspace = true }` → `scanner = { workspace = true }`

---

## Step 3: Update Rust `use` Paths

### `crates/verazt/src/main.rs`
- `checker::cli::run(...)` → `scanner::cli::run(...)`

### `crates/scanner/tests/main.rs`
- `use checker::{...}` → `use scanner::{...}`

### `crates/scanner/tests/benchmark.rs`
- `use checker::{...}` → `use scanner::{...}`

### `crates/scanner/tests/output.rs`
- `use checker::output::{...}` → `use scanner::output::{...}`

### `crates/scanner/tests/detectors.rs`
- `use checker::{...}` → `use scanner::{...}`

### `crates/scanner/tests/vyper_integration.rs`
- `use checker::{...}` → `use scanner::{...}`

### `crates/scanner/tests/passes.rs`
- `use checker::{...}` → `use scanner::{...}`

---

## Step 4: Update the CLI Subcommand in `verazt`

### `crates/verazt/src/main.rs`
- Rename the `Commands::Checker` variant to `Commands::Scanner`
- Update the variant's doc comment from `/// Check smart contracts for vulnerabilities`
  to `/// Scan smart contracts for bugs and security vulnerabilities`

---

## Step 5: Update Documentation

### `crates/scanner/src/lib.rs`
- Line 1: `//! SmartHunt - Smart Contract Bug Detection`
  → `//! Scanner - Smart Contract Security Vulnerability Scanner`
- Line 8: `//! The checker uses a two-phase pipeline architecture:`
  → `//! The scanner uses a two-phase pipeline architecture:`

### `crates/scanner/src/cli.rs`
- Line 1: `//! Analyze - AST-based Smart Contract Bug Detection CLI`
  → `//! Scanner - AST-based Smart Contract Security Scanner CLI`
- Line 3: `//! This is the main entry point for the Analyze tool.`
  → `//! This is the main entry point for the Scanner tool.`
- Line 22 (`about` text): `"Analyze - AST-based Smart Contract Bug Detection"`
  → `"Scanner - AST-based Smart Contract Security Scanner"`

### `crates/scanner/README.md`
- `Smart contract static analyzer.`
  → `Smart contract security vulnerability scanner.`

### `Cargo.toml` (workspace root)
- `about` field (if present): update any mention of "analyzer" or "checker"

---

## Step 6: Verify

```bash
cargo check -p scanner
cargo test -p scanner
cargo build
```

---

## Summary of All Files Changed

| File | Change |
|---|---|
| `crates/checker/` (directory) | Renamed to `crates/scanner/` |
| `crates/scanner/Cargo.toml` | Package name: `checker` → `scanner` |
| `Cargo.toml` | Member path + workspace dep name |
| `crates/verazt/Cargo.toml` | Dependency name |
| `crates/verazt/src/main.rs` | `checker::` → `scanner::`, `Commands::Checker` → `Commands::Scanner` |
| `crates/scanner/tests/main.rs` | `use checker::` → `use scanner::` |
| `crates/scanner/tests/benchmark.rs` | `use checker::` → `use scanner::` |
| `crates/scanner/tests/output.rs` | `use checker::` → `use scanner::` |
| `crates/scanner/tests/detectors.rs` | `use checker::` → `use scanner::` |
| `crates/scanner/tests/vyper_integration.rs` | `use checker::` → `use scanner::` |
| `crates/scanner/tests/passes.rs` | `use checker::` → `use scanner::` |
| `crates/scanner/src/lib.rs` | Doc comments: "checker" → "scanner" |
| `crates/scanner/src/cli.rs` | Doc comments + `about` text |
| `crates/scanner/README.md` | Crate description |
