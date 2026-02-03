# Plan to Restructure `solidity/src/passes`

This document outlines the plan to restructure the `solidity/src/passes` module by moving code into more appropriate locations based on their purpose.

## Overview

The current `passes` module contains code that serves three distinct purposes:
1. **Codegen**: Transforming AST to IR
2. **AST Simplification**: Simplifying or normalizing the Solidity AST
3. **Utility Functions**: Helper functions shared across passes

After restructuring, the `passes` directory will be **deleted**.

---

## Module Categorization

### 1. AST → IR Transformation (Move to `solidity/src/codegen`)

| File | Purpose |
|------|---------|
| `transform.rs` | Transforms the original Solidity AST to Smart Contract IR |

#### Renaming Requirements

When moving `transform.rs` to `codegen/`, also perform the following renames:

1. **Rename data structure**: `Transformer` → `IrGen`
2. **Rename function prefixes**: `transform_*` → `lower_*` (e.g., `transform_source_unit` → `lower_source_unit`)

---

### 2. AST Simplification/Normalization (Move to `solidity/src/ast/simplify`)

| File | Purpose |
|------|---------|
| `elim_func_modifier.rs` | Eliminates function modifier invocations by inlining modifier bodies |
| `elim_import_directives.rs` | Eliminates `import` directives by merging imported elements |
| `elim_named_args.rs` | Eliminates named arguments in function calls |
| `elim_using_directives.rs` | Removes `using` directives in contracts |
| `flatten_expr.rs` | Flattens complex expressions to three-value code format |
| `flatten_name_index.rs` | Flattens names by merging base name with index |
| `merge_pragmas.rs` | Merges multiple `pragma` directives into consolidated versions |
| `rename_callees.rs` | Renames function callees to match renamed function definitions |
| `rename_contracts.rs` | Renames contracts with unique indices |
| `rename_defs.rs` | Renames definitions (functions, events, errors, structs, enums) |
| `rename_vars.rs` | Renames variables to resolve shadowing across scopes |
| `resolve_inheritance.rs` | Resolves inheritance keywords (e.g., `super`) and linearizes contracts |
| `substitution.rs` | Utility for substituting identifiers/names in expressions and statements |
| `unroll_unary_tuple.rs` | Unrolls single-element tuples `(expr)` to `expr` |
| `utils.rs` | Helper functions for normalization passes (e.g., unit test configuration) |

---

### 3. IR Normalization (Move to `solidity/src/ir/normalize`)

> **Note**: Currently, there are no modules in `passes` that specifically normalize IR.
> All existing passes operate on the AST before transformation to IR.
> This category is included for future expansion if IR normalization passes are needed.

---

## Proposed Changes

### [NEW] `solidity/src/codegen/`

Create a new directory and move:
- `passes/transform.rs` → `codegen/transform.rs`
- Create `codegen/mod.rs` to export the module

---

### [NEW] `solidity/src/ast/simplify/`

Create a new directory and move:
- `passes/elim_func_modifier.rs` → `ast/simplify/elim_func_modifier.rs`
- `passes/elim_import_directives.rs` → `ast/simplify/elim_import_directives.rs`
- `passes/elim_named_args.rs` → `ast/simplify/elim_named_args.rs`
- `passes/elim_using_directives.rs` → `ast/simplify/elim_using_directives.rs`
- `passes/flatten_expr.rs` → `ast/simplify/flatten_expr.rs`
- `passes/flatten_name_index.rs` → `ast/simplify/flatten_name_index.rs`
- `passes/merge_pragmas.rs` → `ast/simplify/merge_pragmas.rs`
- `passes/rename_callees.rs` → `ast/simplify/rename_callees.rs`
- `passes/rename_contracts.rs` → `ast/simplify/rename_contracts.rs`
- `passes/rename_defs.rs` → `ast/simplify/rename_defs.rs`
- `passes/rename_vars.rs` → `ast/simplify/rename_vars.rs`
- `passes/resolve_inheritance.rs` → `ast/simplify/resolve_inheritance.rs`
- `passes/substitution.rs` → `ast/simplify/substitution.rs`
- `passes/unroll_unary_tuple.rs` → `ast/simplify/unroll_unary_tuple.rs`
- `passes/utils.rs` → `ast/simplify/utils.rs`

Create `ast/simplify/mod.rs` with:
- Module declarations for all moved files
- Public re-exports matching current `passes/mod.rs`
- The `run_passes` function

---

### [NEW] `solidity/src/ir/normalize/`

Create an empty directory with `mod.rs` for future IR normalization passes.

---

### [DELETE] `solidity/src/passes/`

After all files are moved, delete the entire `passes` directory.

---

## Update Module References

### Update `solidity/src/lib.rs`

Replace:
```rust
pub mod passes;
```

With:
```rust
pub mod codegen;
// ast/simplify is already under ast module
// ir/normalize is already under ir module
```

### Update `solidity/src/ast/mod.rs`

Add:
```rust
pub mod simplify;
```

### Update `solidity/src/ir/mod.rs`

Add:
```rust
pub mod normalize;
```

---

## Update Import Statements

All files referencing `crate::passes::*` must be updated to use:
- `crate::codegen::*` for transform functions
- `crate::ast::simplify::*` for AST simplification functions

Key files to update:
- `solidity/src/compile.rs`
- Any external crates that depend on `solidity::passes`

---

## Verification Plan

### 1. Build Verification
```bash
cargo build -p solidity
```

### 2. Test Verification
```bash
cargo test -p solidity
```

### 3. Full Workspace Check
```bash
cargo build --workspace
cargo test --workspace
```

---

## Implementation Order

1. Create `solidity/src/codegen/` directory and `mod.rs`
2. Move `transform.rs` to `codegen/`
3. Create `solidity/src/ast/simplify/` directory and `mod.rs`
4. Move all AST simplification files to `ast/simplify/`
5. Create `solidity/src/ir/normalize/` directory and `mod.rs`
6. Update `solidity/src/lib.rs`
7. Update `solidity/src/ast/mod.rs`
8. Update `solidity/src/ir/mod.rs`
9. Update all import statements throughout the codebase
10. Delete `solidity/src/passes/` directory
11. Run build and tests to verify

---

## Notes

- The `mod.rs` in `ast/simplify/` should replicate the structure of the current `passes/mod.rs`, including the `run_passes` function and all public exports.
- The `substitution.rs` module is a utility used by other simplification passes, so it belongs in `ast/simplify/`.
- The `utils.rs` module contains only test configuration helpers and should stay with the simplification passes.
