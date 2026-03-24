# Plan: Fix Failed Solidity Frontend Compilation Tests (Round 4)

## Background

Approximately 40 semantic tests still fail after rounds 1–3 of fixes. These tests
run through the pipeline:

1. Parse Solidity → JSON AST → internal AST → export to Solidity → re-compile **(TEST PARSING AST)**
2. Run `run_passes` normalization → export → re-compile **(TEST NORMALIZING AST)**

The current normalization pipeline is:
```
unroll_tuples → rename_defs → eliminate_imports → merge_pragmas
→ rename_callees → resolve_inheritance → strip_specifiers → unroll_tuples
```

---

## Error Categories & Root Causes

### Category A: Tuple Decomposition — `VarDeclStmt::Display` Bug (5 tests)

**Tests:** `abi_encode_call` (×2), `bare_call_no_returndatacopy`, `delegatecall_return_value_pre_byzantium`, `library_address_homestead`

**Fails at:** TEST PARSING AST stage

**Error:** `Different number of components on the left hand side (1) than on the right hand side (2)`

**Example:**
```solidity
// Original:   (bool success, ) = address(this).call(...);
// Exported:   (bool success)   = address(this).call(...);
```

**Root Cause:** The JSON AST represents `(bool success, )` as `var_decls: [Some(VarDecl), None]`. The `VarDeclStmt::Display` at [stmts.rs:706-733](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/ast/stmts.rs#L706-L733) has a comment saying trailing `None` entries are preserved, but the single-element branch at line 708 handles the case where `var_decls.len() == 1` — which means `(bool success)` without a trailing comma. The issue is actually in the **parsing**: when the JSON AST has `[Some, None]`, the parser may be dropping the trailing `None`, leaving only `[Some]` which hits the `len() == 1` branch.

**Fix:** Investigate the JSON AST parser to ensure trailing `None` entries in `VarDeclStmt.var_decls` are preserved. Check [parsing/stmts.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing) for `VariableDeclarationStatement` handling and ensure `null` entries in the `declarations` array produce `None` values.

---

### Category B: Member Not Found After Normalization — `rename_callees` Gaps (25+ tests)

**Tests:** `struct_storage_ptr`, `calldata_array_two_dimensional` (×2), `calldata_internal_library`, `calling_nonexisting_contract_throws`, `external_call_to_nonexisting` (×2), `precompile_extcodesize_check`, `return_size_bigger_than_expected`, `return_size_shorter_than_expected` (×2), `inherited_function_calldata_memory`, `inherited_function_calldata_calldata_interface`, `inherited_function_calldata_memory_interface`, `interface_inheritance_conversions`, `super_skip_unimplemented_in_abstract_contract`, `super_skip_unimplemented_in_interface`, `bound_returning_calldata` (×2), `bound_to_calldata` (×2), `internal_call_bound_with_parentheses`, `internal_library_function_bound`, `internal_library_function_return_var_size`, `library_delegatecall_guard_pure`, `library_delegatecall_guard_view_not_needed`, `library_delegatecall_guard_view_needed`, `interfaceID_interfaces`

**Fails at:** TEST NORMALIZING AST stage

**Error patterns:**
- `Member "f" not found or not visible after argument-dependent lookup in type(library L)` 
- `Member "test" not found or not visible after argument-dependent lookup in contract C`
- `Member "f" not found or not visible after argument-dependent lookup in bytes calldata`
- `Member "f" not found or not visible after argument-dependent lookup in struct L.S memory`
- `Member "f_0" not found ...` / `Member "parentFun_0" not found ...`

**Root Cause:** `rename_defs` renames function **definitions** with indices (e.g., `f` → `f_0`). `rename_callees` is supposed to update **call sites** but fails for several patterns:

1. **`this.func()` calls** — `map_member_expr` handles `this` (line 407-413 of rename_callees.rs), but the function lookup via `find_contract_scopes` may fail if the current contract name resolution doesn't match.
2. **Library qualified calls `L.f()`** — The `base_typ` check at lines 416-425 requires `.is_contract_library_type()` or `.is_magic_contract_library_type()` which may not be set for all library meta-type variations.
3. **`using L for T` bound methods** — The existing `find_using_for_contracts` only handles `UsingLib`; the non-function-type fallback (lines 376-399) only works if there's exactly 1 candidate.
4. **Interface/abstract member calls** — `i.f()`, `A(this).f(m)` — the interface contract type may not be found by `find_contract_scopes` when the base type uses a non-indexed name.
5. **Cross-contract external calls** — `ShortReturn(addr).f_0()` where `f_0` is not in the external interface of the contract being cast to.
6. **`.selector` member on renamed functions** — `i.hello_0.selector` fails because `hello_0` is not found in the interface.

**Sub-categories:**

| Sub-issue | Count | Example | Cause |
|-----------|-------|---------|-------|
| Library `L.f()` calls | 6 | `L.f(r, s)`, `L.f(y)` | Library type not matched by `base_typ` check |
| `this.func()` calls | 2 | `this.test(a, i, j)` | `this` lookup in current contract fails |
| `using for` bound methods | 7 | `_x.f()`, `x.f()` | Bound method lookup returns no match |
| Interface/abstract calls | 5 | `i.f(...)`, `A(this).f(m)` | Interface contract not found |
| Cross-contract casts | 5 | `ShortReturn(addr).f_0()`, `d.g_0()` | Renamed name not valid in external interface |

**Fix strategy:** The core issue is that `rename_defs` renames functions with indices, but not all call site patterns are covered by `rename_callees`. The fix must extend `map_member_expr` and `find_call_definition_name` to handle additional patterns. Alternatively, consider **not renaming functions that are called externally** (via member access on contract/interface/library types).

---

### Category C: Undeclared Identifier After Normalization (6 tests)

**Tests:** `storage_array_ref`, `calldata_bytes_internal`, `calldata_internal_multi_array`, `calldata_internal_multi_fixed_array`, `bytes_in_constructors_packer`, `call_unimplemented_base`

**Fails at:** TEST NORMALIZING AST stage

**Error:** `Undeclared identifier` — e.g., `find` not found (suggests `find_0`/`find_1`), or `f` / `g` / `a_1` undeclared.

**Root Cause:** Same underlying issue as Category B — `rename_callees` fails to update non-member-access calls (plain identifier calls like `f(b, 2)` or `find(...)`) when the type comparison in `find_call_definition_name` doesn't match. `map_ident` (line 443) searches the current contract and base contracts, then free functions in the source unit — but the callee type from the AST may not match the renamed function's type due to differences in data location, scope, or other qualifiers.

**Fix:** Improve the type comparison fallback in `find_call_definition_name`. When no exact type match is found but there are candidates matching by base name, use a more lenient matching strategy (e.g., ignore data locations, scope qualifiers, and struct pointer differences). The existing fallback at line 638 only applies when there's exactly 1 candidate — extend it to handle cases with multiple candidates using a scoring/ranking approach.

---

### Category D: Constructor Visibility / Abstract Contract Issues (4 tests)

**Tests:** `constructor_with_params_diamond_inheritance`, `pass_dynamic_arguments_to_the_base_base_with_gap`, `lisa_interfaceId`, `lisa`

**Fails at:** TEST NORMALIZING AST stage

**Error patterns:**
- `Non-abstract contracts cannot have internal constructors`
- `No arguments passed to the base constructor`

**Root Cause:** The `strip_specifiers` pass at [strip_specifiers.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/strip_specifiers.rs) was updated to preserve `is_abstract` (line 15-23). However, these tests still fail because:

1. **Internal constructors without `abstract`**: `resolve_inheritance` may remove the `abstract` keyword from contracts that need it because they have `internal` constructors. Check whether `resolve_inheritance` is incorrectly modifying the contract's `is_abstract` flag.
2. **Missing base constructor arguments**: When `resolve_inheritance` processes `contract B is A { constructor(uint x) internal {} }`, the base contract `A`'s constructor arguments may not be propagated to the derived contract's constructor invocation list.

**Fix:**
1. Ensure `resolve_inheritance` preserves `is_abstract = true` for contracts with `internal` constructors.
2. For the "no arguments passed to base constructor" error, investigate whether `resolve_inheritance` drops base constructor arguments from the inheritance list.

---

### Category E: Overload Resolution Mismatch (1 test)

**Tests:** `super_overload`

**Fails at:** TEST NORMALIZING AST stage

**Error:** `Invalid type for argument in function call. Invalid implicit conversion from int_const 1 to bool` for `B.f_1(1)`.

**Root Cause:** Overloaded function `f` has variants `f(uint)` and `f(bool)`. After renaming, `f_0` = `f(uint)` and `f_1` = `f(bool)`. The call `B.f(1)` gets renamed to `B.f_1(1)` — picking the wrong overload. `f_1` corresponds to `f(bool)` but is called with integer `1`.

**Fix:** In `find_call_definition_name`, when processing member expressions with qualified base (like `B.f`), ensure the overload with matching parameter types is selected. The issue may be in how `super` is resolved by `resolve_inheritance` — the `super.f(1)` call should map to the parent's `f(uint)`, not `f(bool)`.

---

## Task Checklist

### Phase 1: Fix Tuple Decomposition (Category A — 5 tests)

- [x] **1.1** Investigate the JSON AST parser for `VariableDeclarationStatement` handling
  - File: `crates/frontend/src/solidity/parsing/stmts.rs` (or `mod.rs`)
  - Check if `null` entries in the `declarations` JSON array produce `None` values in `var_decls`
  - Verify that trailing `None` values are not stripped during parsing
- [x] **1.2** If the parser preserves `None` correctly, check why `var_decls.len()` becomes 1
  - Add debug logging or a test to trace the parsing of `(bool success, ) = ...`
- [x] **1.3** Fix the root cause (either in parsing or Display)
- [x] **1.4** Verify fix:
  ```bash
  cargo test --package frontend --test compile_libsolidity -- abi_encode_call 2>&1
  cargo test --package frontend --test compile_libsolidity -- bare_call_no_returndatacopy 2>&1
  cargo test --package frontend --test compile_libsolidity -- delegatecall_return_value_pre_byzantium 2>&1
  cargo test --package frontend --test compile_libsolidity -- library_address_homestead 2>&1
  ```

---

### Phase 2: Fix `rename_callees` for Member Expressions (Categories B, C — 31 tests)

All changes in [rename_callees.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/rename_callees.rs):

- [x] **2.1** Fix library qualified calls (`L.f(...)`)
  - In `map_member_expr`, ensure `base_typ` check covers all library meta-type variants
  - Test: the `base_typ` for `L.f(y)` may be `Type::Meta(MetaType::Function)` rather than `Type::Contract(Library)` — add handling for this case
- [x] **2.2** Fix `using for` bound method calls with multiple candidates
  - Current fallback at line 392 only works for exactly 1 candidate
  - Extend to use type-aware matching when multiple candidates exist
- [x] **2.3** Fix interface/abstract contract member calls
  - When `base_typ` is a contract type for an interface, `find_contract_scopes` should also search the interface's own function declarations (not just inherited ones)
  - Ensure `find_contract_def_by_base_name` works for interfaces
- [x] **2.4** Fix cross-contract external calls with renamed functions
  - When a function is renamed (e.g., `f → f_0`) but called via an external cast (`ShortReturn(addr).f_0()`), the renamed name must be valid in the target contract's interface
  - Consider: member access on external contract types should use the same name as the definition in that contract, even if the function is renamed
- [x] **2.5** Fix `map_ident` fallback for plain identifier calls
  - When `find_call_definition_name` finds multiple candidates by base name, improve selection:
    - Try lenient type matching (ignore data locations, scope, pointer flags)
    - If still ambiguous, prefer candidates from the current contract scope
- [x] **2.6** Fix `.selector` member access on renamed functions
  - `i.hello_0.selector` fails because `hello_0` doesn't exist in the interface
  - This is the same root cause as 2.4
- [x] **2.7** Verify fixes:
  ```bash
  # Library calls
  cargo test --package frontend --test compile_libsolidity -- struct_storage_ptr 2>&1
  cargo test --package frontend --test compile_libsolidity -- calldata_internal_library 2>&1
  cargo test --package frontend --test compile_libsolidity -- library_delegatecall_guard_pure 2>&1

  # this.func() calls
  cargo test --package frontend --test compile_libsolidity -- calldata_array_two_dimensional 2>&1

  # using for bound methods
  cargo test --package frontend --test compile_libsolidity -- bound_to_calldata 2>&1
  cargo test --package frontend --test compile_libsolidity -- internal_library_function_bound 2>&1

  # Interface calls
  cargo test --package frontend --test compile_libsolidity -- inherited_function_calldata_memory 2>&1
  cargo test --package frontend --test compile_libsolidity -- interface_inheritance_conversions 2>&1

  # Undeclared identifiers
  cargo test --package frontend --test compile_libsolidity -- storage_array_ref 2>&1
  cargo test --package frontend --test compile_libsolidity -- calldata_bytes_internal 2>&1
  ```

---

### Phase 3: Fix Constructor Visibility (Category D — 4 tests)

- [x] **3.1** Investigate `resolve_inheritance` in [resolve_inheritance.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/resolve_inheritance.rs)
  - Check if it modifies `is_abstract` flag on contracts
  - Check if it drops base constructor arguments
- [x] **3.2** Ensure contracts with `internal` constructors remain `abstract`
  - After `resolve_inheritance`, verify that `is_abstract = true` is preserved
- [x] **3.3** Ensure base constructor arguments in inheritance lists are preserved
  - `contract B is A(42)` → the `(42)` argument must be propagated through normalization
- [x] **3.4** Verify fix:
  ```bash
  cargo test --package frontend --test compile_libsolidity -- diamond_inheritance 2>&1
  cargo test --package frontend --test compile_libsolidity -- pass_dynamic_arguments_to_the_base_base_with_gap 2>&1
  cargo test --package frontend --test compile_libsolidity -- lisa_interfaceId 2>&1
  cargo test --package frontend --test compile_libsolidity -- interfaceID_lisa 2>&1
  ```

---

### Phase 4: Fix Overload Resolution (Category E — 1 test)

- [x] **4.1** Investigate how `super.f(1)` is resolved in `resolve_inheritance`
  - Check if `super` resolution correctly selects the type-compatible overload
- [x] **4.2** Fix `find_call_definition_name` to correctly match `uint` vs `bool` parameter types
  - Ensure the type-compatible overload is selected when multiple `f_N` variants exist
- [x] **4.3** Verify fix:
  ```bash
  cargo test --package frontend --test compile_libsolidity -- super_overload 2>&1
  ```

---

### Phase 5: Final Verification

- [x] **5.1** Full compilation check:
  ```bash
  cargo check --package frontend 2>&1
  ```
- [x] **5.2** Lowering unit tests (regression check):
  ```bash
  cargo test --package frontend --lib -- solidity::lowering 2>&1
  ```
- [x] **5.3** AST JSON tests (regression check):
  ```bash
  cargo test --package frontend --test compile_libsolidity -- ast_json 2>&1
  ```
- [x] **5.4** All semantic tests:
  ```bash
  cargo test --package frontend --test compile_libsolidity -- semantic_tests 2>&1
  ```
- [x] **5.5** Memory guard tests:
  ```bash
  cargo test --package frontend --test compile_libsolidity -- memory_guard 2>&1
  ```
- [x] **5.6** Document remaining failures — **No failures remain**

---

## Summary of Tests by Category

| Category | # Tests | Root Cause | Fix Location |
|----------|---------|------------|--------------|
| A: Tuple decomposition | 5 | `VarDeclStmt` parsing/display bug | `parsing/stmts.rs`, `ast/stmts.rs` |
| B: Member not found | 25 | `rename_callees` gaps | `lowering/rename_callees.rs` |
| C: Undeclared identifier | 6 | `rename_callees` type matching | `lowering/rename_callees.rs` |
| D: Constructor visibility | 4 | `resolve_inheritance` / `strip_specifiers` | `lowering/resolve_inheritance.rs` |
| E: Overload resolution | 1 | Wrong overload selected | `lowering/rename_callees.rs` |
| **Total** | **41** | | |
