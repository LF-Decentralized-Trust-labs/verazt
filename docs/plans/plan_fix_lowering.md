# Fix: Remove SSA-style Variable Indexing from SIR

## Problem

SIR currently displays variables with SSA-style numeric suffixes (e.g. `name_0`,
`to_1`, `amount_0`) because the AST normalization pipeline runs four renaming
passes **before** lowering to SIR:

```
AST → unroll_tuples → rename_contracts → rename_vars → rename_defs
    → eliminate_imports → merge_pragmas → rename_callees → unroll_tuples → SIR
```

The `Name` type carries a `base` + optional `index`, and `Name::to_string()`
emits `base_index`.  When `lower.rs` calls `.to_string()` on every name, the
indexes are baked into SIR strings.

**Desired state:** SIR should stay close to the original AST source text.
SSA renaming should only happen during CIR → BIR lowering (which already has
`ssa::rename_to_ssa()`).

## Analysis: Two Distinct Concerns in Current Renaming

| Pass | Purpose | Needed before SIR? |
|---|---|---|
| `rename_contracts` | Disambiguate contracts with same name across source units | Yes — needed for `eliminate_imports` |
| `rename_defs` | Disambiguate overloaded functions, events, errors, structs, enums | Yes — needed for `rename_callees` |
| `rename_callees` | Resolve callee identifiers to their disambiguated definitions | Yes — needed for correct SIR |
| **`rename_vars`** | **Resolve shadowed local/state variables with scoped indexes** | **No** — this is SSA-like work |

## Proposed Fix

### Phase 1: Remove `rename_vars` from AST→SIR pipeline

#### [MODIFY] [lower.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/lower.rs)

Remove the `rename_vars` call from `run_passes()`:

```diff
 pub fn run_passes(source_units: &[ast::SourceUnit]) -> Vec<ast::SourceUnit> {
     let source_units = super::unroll_tuples::unroll_unary_tuple(source_units);
     print_output_source_units(&source_units);

     let env = ast::NamingEnv::new();
     let (source_units, env) = super::rename_contracts::rename_contracts(&source_units, Some(&env));
     print_output_source_units(&source_units);

-    let (source_units, env) = super::rename_vars::rename_vars(&source_units, Some(&env));
-    print_output_source_units(&source_units);
-
-    let (source_units, env) = super::rename_defs::rename_defs(&source_units, Some(&env));
+    let (source_units, env) = super::rename_defs::rename_defs(&source_units, Some(&env));
     print_output_source_units(&source_units);
     ...
```

### Phase 2: Move variable shadowing resolution to CIR → BIR lowering

The BIR lowering already has `ssa::rename_to_ssa()` which assigns version
numbers to variable definitions at the basic-block level. This **already
handles the SSA concern**, so `rename_vars` is simply redundant once removed
from the AST normalization.

If the existing `ssa::rename_to_ssa()` does not properly handle all shadow-resolution
cases (e.g. nested blocks), it should be extended rather than keeping the AST-level pass.

> [!IMPORTANT]
> After removing `rename_vars` from `run_passes()`, variables in SIR will keep
> their original source names. Shadowed variables within different scopes will
> share the same name in SIR — this is acceptable because SIR is meant to be
> source-faithful. The disambiguation happens later at BIR level where SSA form
> is constructed.

### Phase 3: Update unit tests

#### [MODIFY] [rename_vars.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/rename_vars.rs)

The unit tests in `rename_vars.rs` should still pass (they test the `rename_vars`
function directly, not via `run_passes()`). No changes needed here — the module
remains available for use in future CIR→BIR work if needed.

#### Integration tests / end-to-end

After the change, SIR output for input like:

```solidity
contract Token {
    string name;
    function transfer(address to, uint256 amount) returns (bool) { ... }
}
```

should produce:

```
contract Token_0 {
    string name;
    function transfer_0(address to, u256 amount) returns (bool) { ... }
}
```

- Contract and definition names still get indexes (they have overload-disambiguation purpose).
- **Variable names (`name`, `to`, `amount`) no longer get indexes.**

## Verification Plan

### Automated Tests

1. **Existing unit tests must still pass:**
   ```
   cargo test -p frontend 2>&1
   ```

2. **Build verification:**
   ```
   cargo check 2>&1
   ```

3. **Logic verification:**
   ```
   cargo run -p zkformal
   ```

### Manual Verification

Run the tool on a sample Solidity file (e.g. the Token contract from the issue)
and inspect the SIR output to confirm variable names no longer have `_N` suffixes
while contract/function definition names still do.
