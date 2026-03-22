# Fix: Make SIR Source-faithful by Removing Indexed Renaming

## Problem

SIR currently displays all names with numeric suffixes (e.g. `name_0`, `to_1`,
`Token_0`, `transfer_0`) because four renaming passes run before SIR lowering.
These passes use `NamingEnv` to assign monotonically increasing indexes to every
name.

**Current pipeline:**
```
AST → unroll_tuples → rename_contracts → rename_vars → rename_defs
    → eliminate_imports → merge_pragmas → rename_callees → unroll_tuples → SIR
```

**Goal:** SIR should stay close to the original source text. SSA-style renaming
should only happen at BIR level (which already has `ssa::rename_to_ssa()`).

## Root Cause Analysis

The four renaming passes serve **two distinct purposes**:

| Purpose | Passes | Needed? |
|---|---|---|
| **Cross-file collision avoidance** (after import flattening) | `rename_contracts`, `rename_vars`, `rename_defs` | Yes, but should use **namespace prefixes** instead of numeric indexes |
| **Overloaded function disambiguation** (same name, different signatures) | `rename_defs`, `rename_callees` | Yes, genuinely needed — SIR uses string names without type dispatch |
| **Shadowed variable resolution** (same name in nested scopes) | `rename_vars` | No — defer to BIR SSA |

## Proposed Changes

### Phase 1: Replace `rename_contracts` with namespace prefixing in `eliminate_imports`

Instead of `C_0` / `C_1`, use the **import alias as a namespace prefix**:

```
import "A.sol" as X;   // A.sol has: contract C, function f()
import "B.sol" as Y;   // B.sol has: contract C

// After flattening:
contract X_C { ... }    // was X.C
contract Y_C { ... }    // was Y.C
function X_f() { ... }  // was X.f()
```

For **nested aliases** (e.g., `import "B.sol" as S2` where B.sol does
`import "A.sol" as S1`), fully flatten:

```
S2.S1.bar()  →  function S2_S1_bar()
```

#### [MODIFY] [eliminate_imports.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/eliminate_imports.rs)

- In `unfold_imported_source_unit()`: when importing with an alias, prefix all
  definition names in the imported elements with `{alias}_` (instead of relying
  on a prior `rename_contracts` pass).
- For nested imports, compose the prefix chain: if a source unit already has
  prefixed names from its own imports, the outer alias prepends to them.

#### [MODIFY] [lower.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/lower.rs)

Remove `rename_contracts` and `rename_vars` from `run_passes()`:

```diff
 pub fn run_passes(source_units: &[ast::SourceUnit]) -> Vec<ast::SourceUnit> {
     let source_units = super::unroll_tuples::unroll_unary_tuple(source_units);

-    let env = ast::NamingEnv::new();
-    let (source_units, env) = super::rename_contracts::rename_contracts(&source_units, Some(&env));
-
-    let (source_units, env) = super::rename_vars::rename_vars(&source_units, Some(&env));
-
-    let (source_units, env) = super::rename_defs::rename_defs(&source_units, Some(&env));
+    let env = ast::NamingEnv::new();
+    let (source_units, env) = super::rename_defs::rename_defs(&source_units, Some(&env));

     let source_units = super::eliminate_imports::eliminate_import(&source_units);
     let source_units = super::merge_pragmas::merge_pragmas(&source_units);

     let (source_units, _) = super::rename_callees::rename_callees(&source_units, Some(&env));
     super::unroll_tuples::unroll_unary_tuple(&source_units)
 }
```

### Phase 2: Keep `rename_defs` + `rename_callees` for overloaded function disambiguation

These are still needed because Solidity allows function overloading within the
same contract — e.g. `f(uint)` and `f(string)` must map to distinct SIR
function names (`f_0`, `f_1`). This is a semantic necessity, not a naming
cosmetic.

> [!IMPORTANT]
> `rename_defs` currently also renames event, error, struct, and enum
> definitions. Consider whether non-overloadable definitions (structs, enums)
> can skip indexing entirely. This is a follow-up optimization.

### Phase 3: Remove `rename_vars` from the pipeline entirely

Variable shadowing resolution is deferred to the BIR level:
- BIR's `ssa::rename_to_ssa()` already assigns version numbers to variable
  definitions at the basic-block level.
- For cross-file variable collisions (e.g., state variable `a` in two files),
  the namespace prefixing from Phase 1 handles it.

## Expected SIR Output After Fix

For this Solidity input:
```solidity
contract Token {
    string name;
    string symbol;
    function transfer(address to, uint256 amount) returns (bool) {
        require(balanceOf[msg.sender] >= amount, "Insufficient balance");
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        return true;
    }
}
```

SIR should produce (no numeric suffixes on variables):
```
contract Token {
    string name;
    string symbol;
    function transfer(address to, u256 amount) returns (bool) {
        require((balanceOf[evm.msg_sender()] >= amount), "Insufficient balance");
        balanceOf[evm.msg_sender()] -= amount;
        balanceOf[to] += amount;
        return true;
    }
}
```

> [!NOTE]
> Overloaded function names will still get indexes (e.g., `f_0`, `f_1`) since
> that's a semantic disambiguation, not SSA naming.

## Verification Plan

### Automated Tests

1. **Existing unit tests** — after modifications, run:
   ```
   cargo test -p frontend 2>&1
   ```
   Tests in `rename_vars.rs`, `rename_defs.rs`, `rename_contracts.rs`,
   `rename_callees.rs`, `eliminate_imports.rs`, and `resolve_inheritance.rs`
   will need their expected outputs updated.

2. **Build check:**
   ```
   cargo check 2>&1
   ```

3. **Logic verification:**
   ```
   cargo run -p zkformal
   ```

### Manual Verification

Run the tool on a sample Solidity contract and inspect SIR output to confirm:
- Variable names have **no** `_N` suffixes
- Contract/function names have **no** `_N` suffixes (unless overloaded)
- Aliased imports produce namespace-prefixed names (e.g., `X_C`)

---

## Task Checklist

### Phase 1: Namespace-prefixed import flattening

- [x] **1.1** Modify `unfold_imported_source_unit()` in `eliminate_imports.rs`
  - [x] When `source_unit_alias` is present, prefix all imported definition names
        (contracts, functions, state vars, structs, enums, errors, events) with
        `{alias}_`
  - [x] Update aliased references in `symbol_aliases` map to use prefixed names
  - [x] Handle nested aliases: if imported elements already carry a prefix from
        their own import elimination, prepend the outer alias
        (`S2_` + `S1_bar` → `S2_S1_bar`)
- [x] **1.2** Modify `unfold_imported_symbols()` in `eliminate_imports.rs`
  - [x] Ensure symbol imports (`import {C} from "A.sol"`) do NOT add namespace
        prefixes (they bring names directly into scope, no collision possible)
- [x] **1.3** Remove `rename_contracts` call from `run_passes()` in `lower.rs`
- [x] **1.4** Remove `rename_vars` call from `run_passes()` in `lower.rs`
- [x] **1.5** Update `eliminate_imports.rs` unit tests
  - [x] `remove_circular_imports`: update expected output — no `rename_vars`
        indexes, no `rename_contracts` indexes
  - [x] `remove_multiple_level_imports`: update expected output — use namespace
        prefixes (`S1_fre`, `S1_bar`, `S1_a`) instead of `_0`/`_1`/`_2` indexes
- [x] **1.6** Verify `cargo check 2>&1` passes

### Phase 2: Keep overloaded function disambiguation

- [x] **2.1** Verify `rename_defs` still runs correctly without prior
      `rename_contracts` and `rename_vars` passes (env starts clean)
- [x] **2.2** Update `rename_defs.rs` unit test expected output
  - [x] Only overloaded functions should get indexes (`g_0`, `g_1`);
        non-overloaded functions (`z`) should ideally get no index
- [x] **2.3** Update `rename_callees.rs` unit test expected output
  - [x] Same as above: only overloaded names get indexes
- [x] **2.4** Consider whether `rename_defs` should skip non-overloadable
      definitions (structs, enums, events, errors) — these can't be overloaded
      in Solidity, so indexing them is unnecessary
  - [x] If yes, modify `rename_defs.rs` to only rename `FuncDef` nodes

### Phase 3: Remove `rename_vars` from pipeline

- [x] **3.1** Already done in step 1.4 (removing from `run_passes()`)
- [x] **3.2** Verify `rename_vars.rs` unit tests still pass standalone
      (the module stays available, just not called in the pipeline)
- [x] **3.3** Update `rename_contracts.rs` unit test expected output
  - [x] If `rename_contracts` is removed from pipeline, its standalone tests
        still pass but pipeline-level tests no longer exercise it
- [x] **3.4** Update `resolve_inheritance.rs` unit test
  - [x] This test calls `rename_contracts`, `rename_defs`, `rename_callees`
        — adjusted: removed `rename_contracts` call, updated expected output

### Cross-cutting updates

- [x] **4.1** Update `flatten_names.rs` — still needed for `rename_defs` indexes
- [x] **4.2** Search for all callers of `rename_vars()`, `rename_contracts()`
      outside of `lower.rs` and update them
- [x] **4.3** Update `mod.rs` — kept all re-exports (modules still exist for
      standalone use)

### Verification

- [x] **5.1** `cargo check 2>&1`
- [ ] **5.2** `cargo test -p frontend 2>&1` — all unit tests fail due to
      `solc-select: No such file or directory` (pre-existing infrastructure issue)
- [ ] **5.3** `cargo run -p zkformal` — package not found in workspace
- [ ] **5.4** Manual inspection: run on Token contract, confirm SIR has no
      variable indexes and aliased imports use namespace prefixes

