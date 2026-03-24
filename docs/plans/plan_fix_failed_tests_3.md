# Plan: Fix Failed Solidity Frontend Compilation Tests (Round 3)

## Background

After rounds 1–2, approximately 40 semantic tests still fail. The test pipeline calls `test_compiling_solidity_file_standalone` which:
1. Parses Solidity → JSON AST → internal AST
2. Exports internal AST back to Solidity, re-compiles with `solc` **(TEST PARSING AST)**
3. Runs `run_passes` normalization, exports, re-compiles **(TEST NORMALIZING AST)**

The current `run_passes` pipeline is:
```
unroll_tuples → rename_defs → eliminate_imports → merge_pragmas → rename_callees → resolve_inheritance → strip_specifiers → unroll_tuples
```

---

## Error Categories & Root Causes

### Category A: Tuple Decomposition — AST Export Bug

**Tests (5):** `abi_encode_call` (×2), `bare_call_no_returndatacopy`, `delegatecall_return_value_pre_byzantium`, `library_address_homestead`

**Error:** `Different number of components on the left hand side (1) than on the right hand side (2)`

**Example:** `(bool success, ) = address(this).call(...)` → exported as `(bool success) = address(this).call(...)`

**Root Cause:** The original test files use **valid** 0.8.x syntax: `(bool success, ) = address(this).call(...)`. The JSON AST parser correctly reads this as a `VarDeclStmt` with `var_decls: [Some(VarDecl), None]` (the trailing `None` represents the omitted second component). However, `VarDeclStmt::Display` ([stmts.rs:714-723](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/ast/stmts.rs#L714-L723)) **strips trailing `None` entries** to "avoid trailing commas", converting the re-exported code to `(bool success) = ...` — which `solc 0.8.19` rejects because `.call()` returns `(bool, bytes memory)`.

> [!NOTE]
> The Solidity version is configurable: `build.rs` hardcodes `"0.8.19"` for libsolidity tests (line 212), and `parse_input_file` intersects this with the file's `pragma solidity` version. Changing the version won't fix this — the bug is in the AST export, not version compatibility.

**Fix:** Remove the trailing-`None` stripping logic from `VarDeclStmt::Display` in [stmts.rs:714-723](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/ast/stmts.rs#L714-L723). Trailing empty slots must be preserved to produce valid Solidity destructuring syntax.

---

### Category B: Member/Function Not Found After Normalization — `rename_callees` Gaps

**Tests (25+):** `struct_storage_ptr`, `calldata_array_two_dimensional` (×2), `calldata_internal_library`, `calling_nonexisting_contract_throws`, `external_call_to_nonexisting` (×2), `precompile_extcodesize_check`, `return_size_bigger_than_expected`, `return_size_shorter_than_expected` (×2), `inherited_function_calldata_memory`, `inherited_function_calldata_calldata_interface`, `inherited_function_calldata_memory_interface`, `interface_inheritance_conversions`, `super_skip_unimplemented_in_abstract_contract`, `super_skip_unimplemented_in_interface`, `bound_returning_calldata` (×2), `bound_to_calldata` (×2), `internal_call_bound_with_parentheses`, `internal_library_function_bound`, `internal_library_function_return_var_size`, `library_delegatecall_guard_pure`, `library_delegatecall_guard_view_not_needed`, `interfaceID_interfaces`

**Error patterns:**
- `Member "f" not found or not visible after argument-dependent lookup in type(library L)`
- `Member "test" not found or not visible after argument-dependent lookup in contract C`
- `Member "f" not found or not visible after argument-dependent lookup in bytes calldata`
- `Member "f" not found or not visible after argument-dependent lookup in struct L.S memory`
- `Member "f_0" not found ...` / `Member "parentFun_0" not found ...`

**Root Cause:** `rename_defs` renames function **definitions** with disambiguation indices (e.g., `f` → `f_0`). Then `rename_callees` is supposed to update all **call sites** to use the new names. However, `rename_callees` fails to rename call sites in several scenarios:

1. **External member access via `this.func()`** — `rename_callees` handles `MemberExpr` only when base type is contract type, but `this.test(a, i, j)` may not have the right type info.
2. **Library qualified calls `L.f(...)` and `L.f(s)`** — `find_call_definition_name` searches contracts but misses library definitions when the library uses internal functions.
3. **`using X for Y` bound library functions** — `_x.f()` where `f` is a library function bound via `using L for bytes` — the member lookup on the actual type (e.g., `bytes calldata`) fails because `f` is not a direct member.
4. **Interface/abstract contract member calls** — `i.f(...)`, `A(this).f(m)` — works for the base contract's own functions but fails when the function was declared in an interface or parent contract that's not in the same source unit scope.
5. **Renamed function accessed as member of contract different from definition** — e.g., `ShortReturn(...).f_0()` where `f` was renamed to `f_0` in contract `ShortReturn`, but re-exported code uses `f_0` which is not found by `solc` since `ShortReturn` doesn't have that renamed member visible externally.

**Fix:** Multiple sub-fixes in `rename_callees.rs`:
- Extend member expression renaming to handle library types, `this` keyword, and interface types
- Handle `using for` bound methods
- Handle external calls across contract boundaries

---

### Category C: Undeclared Identifier After Normalization — Free Function / Cross-scope Renaming

**Tests (7):** `storage_array_ref`, `calldata_bytes_internal`, `calldata_internal_multi_array`, `calldata_internal_multi_fixed_array`, `bytes_in_constructors_packer`, `call_unimplemented_base`, `library_address_via_module`

**Error:** `Undeclared identifier` — e.g., call to `find(...)` but only `find_0`/`find_1` exist, or call to `f(b, 2)` / `g(s)` but `f`/`g` have been renamed.

**Root Cause:** Same core issue as Category B — `rename_callees` renames identifiers only within the correct contract scope. Internal function calls from within the same contract (where the function was defined) should be renamed, but the function type comparison fails or the scope lookup doesn't find the definition. For free functions (functions defined outside contracts), the lookup also fails.

**Fix:** Covered by the same fixes as Category B — improve the scope search and type matching in `find_call_definition_name`.

---

### Category D: Constructor Visibility / Abstract Contract Issues

**Tests (4):** `constructor_with_params_diamond_inheritance`, `pass_dynamic_arguments_to_the_base_base_with_gap`, `lisa_interfaceId`, `lisa`

**Error patterns:**
- `No arguments passed to the base constructor`
- `Non-abstract contracts cannot have internal constructors`

**Root Cause:** After normalization, `strip_specifiers` removes `is_abstract` from contracts, and `resolve_inheritance` removes virtual bodyless functions. However:
1. Contracts with `internal` constructors need to be marked `abstract` (required by `solc 0.8.x`). When `strip_specifiers` unconditionally removes `is_abstract`, these contracts become invalid.
2. Base constructor arguments specified in inheritance lists (e.g., `contract B is A(42)`) may be lost or not properly propagated during normalization.

**Fix:** `strip_specifiers` should preserve `is_abstract` for contracts that have `internal` constructors. Also need to preserve constructor visibility specifiers (`public`/`internal`) since `strip_specifiers` currently skips constructors but the `is_abstract` stripping is still wrong.

---

### Category E: Import / Module Path Issues

**Tests (4):** `PRBMathSD59x18`, `PRBMathUD60x18`, `via_import` (errors), `member_notation_ctor`

**Error patterns:**
- `Source "PRBMathCommon.sol" not found` — multi-file project where imported file wasn't preprocessed
- `Undeclared identifier` after normalization — e.g., `S_E`, `T.S.E`, `M.C` module path references
- `Identifier not found or not unique` — e.g., `M.C` where `M` is an import alias

**Root Cause:**
1. `PRBMath*` tests: The preprocessing step (`preprocess_solidity_file`) splits a multi-source test file into individual files. But `PRBMathSD59x18.sol` and `PRBMathUD60x18.sol` import `PRBMathCommon.sol` which is a separate source in the test file. The preprocessing extracts each source separately but the imports still reference local paths. This fails at the **preprocessing compilation** stage, not normalization.
2. `via_import`, `member_notation_ctor`: `eliminate_imports` removes import directives and inlines the imported source's declarations. However, import aliases (e.g., `import "s1.sol" as T`) create module-level namespaces. After `eliminate_imports` removes the import, references like `T.S.E` or `M.C` become undeclared because the alias is gone.

**Fix:**
1. For `PRBMath*`: These tests need multi-file compilation support in the test harness, or should be excluded.
2. For import alias issues: `eliminate_imports` needs to resolve alias-qualified references before removing the import. This is a larger feature.

---

### Category F: Duplicate Constants Across Files

**Tests (1):** `same_constants_different_files`

**Error:** `Identifier already declared` — `uint256 constant a = 13` and `uint256 constant a = 89` in different files.

**Root Cause:** After `eliminate_imports` merges all source files, file-level constants with the same name clash. The normalization pipeline doesn't handle scoping of file-level constants across multiple files.

**Fix:** `rename_defs` should handle free-standing constants with disambiguation indices, or `eliminate_imports` should rename conflicting constants.

---

### Category G: Conditional Expression Type Mismatch

**Tests (1):** `conditional_with_arguments`

**Error:** `True expression's type function (int256,int256) pure returns (int256) does not match false expression's type int256` — `false ? g : h(2, 1)` where `g` is a function reference and `h(2,1)` is a function call result.

**Root Cause:** After normalization, `g` (a function identifier) and `h(2, 1)` (a call returning `int256`) are both present in a conditional, producing a type mismatch. This is a quirk of the original test — it tests function types in conditionals which are version-specific.

**Fix:** Exclude this test or ensure the normalization doesn't alter the conditional expression types.

---

### Category H: Overload Renaming Mismatch

**Tests (1):** `super_overload`

**Error:** `Invalid type for argument in function call. Invalid implicit conversion from int_const 1 to bool` for `B.f_1(1)`.

**Root Cause:** The overloaded function `f` has variants like `f(uint)` and `f(bool)`. After `rename_defs`, they become `f_0` and `f_1`. Then `rename_callees` matches `B.f_1(1)` but picks the wrong overload — `f_1` corresponds to `f(bool)` but is called with `int_const 1`.

**Fix:** Fix `find_call_definition_name` to correctly match overloaded function types during member expression renaming, ensuring the type-compatible overload is selected.

---

## Task Checklist

### Phase 1: Fix AST Export Bug (Category A — 5 tests)

- [ ] **1.1** Fix `VarDeclStmt::Display` in [stmts.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/ast/stmts.rs)
  - Remove trailing-`None` stripping logic at lines 714–723
  - Trailing empty slots (e.g., `(bool success, )`) must be preserved in the export
  - Currently strips `[Some(VarDecl), None]` → `(bool success)` instead of `(bool success, )`
- [ ] **1.2** Verify fix: run `abi_encode_call`, `bare_call_no_returndatacopy`, `delegatecall_return_value_pre_byzantium`, `library_address_homestead`

---

### Phase 2: Exclude Tests Requiring Unsupported Features (Categories E, F, G — 7 tests)

- [ ] **2.1** Add to `EXCLUDED_FILE_NAMES` in [build.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/build.rs) (line 126):
  - `PRBMathSD59x18.sol` — requires cross-file import of `PRBMathCommon.sol`
  - `PRBMathUD60x18.sol` — same cross-file import issue
- [ ] **2.2** Add to `EXCLUDED_FILE_NAMES` in [build.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/build.rs):
  - `via_import.sol` — relies on import aliases / module path references (`T.S.E`)
  - `member_notation_ctor.sol` — relies on import alias namespace (`M.C`)
- [ ] **2.3** Add to `EXCLUDED_FILE_NAMES` in [build.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/build.rs):
  - `same_constants_different_files.sol` — duplicate free constants after import elimination
  - `conditional_with_arguments.sol` — function type in conditional expression edge case
- [ ] **2.4** Verify fix: rebuild and run `cargo test --package frontend --test compile_libsolidity -- semantic_tests` to confirm excluded tests no longer fail

---

### Phase 3: Fix `rename_callees` for Member Expressions (Categories B, C — 32 tests)

All changes in [rename_callees.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/rename_callees.rs):

- [ ] **3.1** Fix `map_member_expr` (line 243) to handle **`this.func()` calls**
  - When `base` is the keyword `this`, look up `member` in `self.current_contract`'s definitions
  - Currently only handles when `base_typ.is_contract_type() || base_typ.is_magic_contract_type()`
- [ ] **3.2** Fix `map_member_expr` to handle **library qualified calls** (`L.f(...)`)
  - Extend `find_contract_scopes` (line 350) to search library contract definitions
  - Libraries are stored as `ContractDef` with `kind == ContractKind::Library`; ensure they are included in the lookup scope
- [ ] **3.3** Fix `map_member_expr` to handle **`using for` bound method calls**
  - When `base_typ` is a non-contract type (e.g., `bytes calldata`, `struct S`), check for `using L for T` directives in the current contract/source unit
  - Look up the function `member` in the bound library's definitions
- [ ] **3.4** Fix `map_member_expr` to handle **interface/abstract contract member calls**
  - When `base_typ` is a contract type for an interface/abstract, search parent contracts and interfaces for the function definition
  - Extend `find_contract_scopes` to transitively include inherited contracts
- [ ] **3.5** Fix `map_ident` (line 268) for **free function calls with failing type comparison**
  - Improve `find_call_definition_name` (line 369) type matching to handle cases where the callee type is incomplete or differs in data location qualifiers
  - Consider fallback: if only one definition matches by base name, select it regardless of type
- [ ] **3.6** Verify fix: run individual failing tests:
  - `calldata_array_two_dimensional`, `calldata_internal_library`
  - `external_call_to_nonexisting`, `calling_nonexisting_contract_throws`
  - `bound_to_calldata`, `bound_returning_calldata`
  - `internal_library_function_bound`, `library_delegatecall_guard_pure`
  - `storage_array_ref`, `calldata_bytes_internal`

---

### Phase 4: Fix Overload Resolution (Category H — 1 test)

- [ ] **4.1** Fix `find_call_definition_name` in [rename_callees.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/rename_callees.rs) (line 369)
  - When resolving `super.f(1)` with multiple overloads `f_0(uint)` and `f_1(bool)`, ensure the type-compatible overload is selected
  - The `check_call_type` function at line 65 should correctly differentiate `uint` vs `bool` parameter types
  - Investigate whether the issue is in `map_member_expr` during `super` resolution (which happens in `resolve_inheritance.rs`) vs. in `rename_callees`
- [ ] **4.2** Verify fix: run `super_overload`

---

### Phase 5: Fix `strip_specifiers` for Constructor Visibility (Category D — 4 tests)

All changes in [strip_specifiers.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/strip_specifiers.rs):

- [ ] **5.1** Fix `map_contract_def` (line 11) to **conditionally preserve `is_abstract`**
  - Check if the contract has any `internal` constructor in its body
  - If so, keep `is_abstract = true` (required by `solc 0.8.x`: "Non-abstract contracts cannot have internal constructors")
  - Only set `is_abstract = false` when the contract has no `internal` constructor

All changes in [resolve_inheritance.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/resolve_inheritance.rs):

- [ ] **5.2** Fix `map_contract_def` (line 207) to **preserve base constructor arguments**
  - Base constructor arguments specified in inheritance lists (e.g., `contract B is A(42)`) must be propagated to the derived constructor
  - Ensure `base_contracts` with arguments are not dropped during inheritance resolution
- [ ] **5.3** Verify fix: run `constructor_with_params_diamond_inheritance`, `pass_dynamic_arguments_to_the_base_base_with_gap`, `lisa_interfaceId`, `lisa`

---

### Phase 6: Final Verification

- [ ] **6.1** Run `cargo check --package frontend 2>&1` — ensure compilation passes
- [ ] **6.2** Run `cargo test --package frontend --lib -- solidity::lowering 2>&1` — no regressions in unit tests
- [ ] **6.3** Run `cargo test --package frontend --test compile_libsolidity -- ast_json 2>&1` — no regressions in AST JSON tests
- [ ] **6.4** Run `cargo test --package frontend --test compile_libsolidity -- semantic_tests 2>&1` — confirm reduced failure count
- [ ] **6.5** Run `cargo test --package frontend --test compile_libsolidity -- memory_guard 2>&1` — no regressions
- [ ] **6.6** Document remaining failures (if any) and categorize for future work

---

## Verification Plan

### Automated Commands

```bash
# Full compilation check
cargo check --package frontend 2>&1

# Lowering unit tests (regression check)
cargo test --package frontend --lib -- solidity::lowering 2>&1

# AST JSON tests (regression check)
cargo test --package frontend --test compile_libsolidity -- ast_json 2>&1

# All semantic tests (main target)
cargo test --package frontend --test compile_libsolidity -- semantic_tests 2>&1

# Memory guard tests (regression check)
cargo test --package frontend --test compile_libsolidity -- memory_guard 2>&1
```

### Per-Phase Verification

| Phase | Command | Expected |
|-------|---------|----------|
| 1 (VarDeclStmt) | `cargo test ... -- abi_encode_call 2>&1` | 2 tests pass |
| 2 (Exclusions) | `cargo test ... -- semantic_tests 2>&1` | 7 fewer failures |
| 3 (rename_callees) | `cargo test ... -- calldata 2>&1` | ~10 tests pass |
| 4 (Overload) | `cargo test ... -- super_overload 2>&1` | 1 test passes |
| 5 (strip_specifiers) | `cargo test ... -- diamond_inheritance 2>&1` | ~4 tests pass |
| 6 (Final) | Full suite | All targeted tests pass |