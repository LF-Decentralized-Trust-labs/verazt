# Plan: Fix Failing Frontend Tests

## Summary

Multiple test suites in `crates/frontend` are failing across 12 root cause categories.
This plan groups all failures by root cause and proposes targeted fixes.

---

## Checklist

### Yul Parser — in [ast_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/json_ast_parser/ast_parser.rs) (Categories 1–2)
- [ ] **1.** `parse_yul_func_def` (~line 1950): make `"parameters"` field optional, default to empty `Vec`
- [ ] **2.** `parse_yul_function_call` (~line 2170): make `"type"` field optional, default to `YulType::Unkn` when absent/empty
- [ ] **3.** `parse_yul_ident` (~line 2196): make `"type"` field optional, default to `YulType::Unkn` when absent/empty
- [ ] **4.** `parse_yul_ident_or_member_expr` (~line 2208): make `"type"` field optional, default to `YulType::Unkn` when absent/empty

### Override Parsing — in [ast_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/json_ast_parser/ast_parser.rs) (Category 3)
- [ ] **5.** `parse_overriding` (~line 734): when `"overrides"` array is empty, return `Overriding::All` instead of `Overriding::Some(vec![])`

### Normalization — in [lower.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/lower.rs) (Category 4)
- [ ] **6.** `run_passes` (~line 30): add a pass after `rename_defs` to strip `override`/`virtual` from all `FuncDef` nodes

### Slice Parsing — in [ast_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/json_ast_parser/ast_parser.rs) (Category 6)
- [ ] **7.** `parse_index_range_access` (~line 1468): make `startExpression` and `endExpression` optional (Solidity allows `x[4:]`, `x[:2]`, `x[:]`)

### Type Parser — in [type_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/type_parser/type_parser.rs) (Category 7)
- [ ] **8.** Add grammar rules and handling for `literal_string "..."` and `int_const N` inside tuple types

### Empty Tuple Components — in [defs.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/ast/defs.rs) (Category 8)
- [ ] **9.** Fix `VarDeclStmt` or tuple `Display` to not output empty tuple slots like `(bool success, )` which Solc rejects

### Constant Initializer — in [ast_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/json_ast_parser/ast_parser.rs) or [defs.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/ast/defs.rs) (Category 9)
- [ ] **10.** Investigate and fix why constant variable declarations lose their initializer value after parsing/printing
  - [ ] Check if `parse_var_decl` correctly reads the `"value"` field
  - [ ] Check if the `Display` impl for `VarDecl` correctly prints the initializer

### Base Constructor Arguments — in [ast_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/json_ast_parser/ast_parser.rs) or [defs.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/ast/defs.rs) (Category 10)
- [ ] **11.** Investigate and fix why base constructor arguments are dropped
  - [ ] Check `parse_inheritance_specifier` for proper argument parsing
  - [ ] Check `Display` impl for `InheritanceSpecifier`/`BaseContract` to ensure args are printed

### Pragma Version Parsing — in parsing code (Category 11)
- [ ] **12.** Fix pragma version range parsing to handle `>=0.4.0<0.9.0` (missing space before `<`)

### Test Filter — in [build.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/build.rs) (Categories 5 + 12)
- [ ] **13.** Update `is_error_test` or add exclusion list for:
  - [ ] Files containing `==== ExternalSource:` directives
  - [ ] Files with unresolvable imports (external dependencies)
  - [ ] `event_with_variables_of_internal_types.sol` (invalid Solidity)

### Verification
- [ ] **14.** Run all affected test suites:
  - [ ] `cargo test -p frontend -- ast_json 2>&1`
  - [ ] `cargo test -p frontend -- memory_guard_tests 2>&1`
  - [ ] `cargo test -p frontend -- semantic_tests 2>&1`
  - [ ] Full regression: `cargo test -p frontend 2>&1`

---

## Root Cause Analysis

### Category 1 — Yul JSON nodes with missing/empty `type` field (many tests)

**Affected test suites:** `ast_json`, `memory_guard_tests`, `semantic_tests`

**Representative tests:** `assembly_call`, `assembly_switch`, `abiEncoderV1_cleanup_cleanup`, `abiEncoderV2_cleanup_address`, `asmForLoop_for_loop_break`, `externalContracts_snark`, `externalContracts_prbmath_PRBMathCommon`, etc.

**Root cause:** The Solc JSON AST omits or empties `"type"` for Yul functions/identifiers. Parser requires it.

| Function | Error message |
|---|---|
| `parse_yul_function_call` | `Function call type not found` |
| `parse_yul_ident` | `Yul identifier type not found` |
| `parse_yul_ident_or_member_expr` | `Yul identifier type not found` |

---

### Category 2 — Yul function definitions with missing `parameters` field

**Affected tests:** `assembly_leave`, `assembly_nested_functions`

**Root cause:** Yul functions with zero parameters omit the `"parameters"` field.

---

### Category 3 — `override()` printing with empty override list

**Affected tests:** `override_`, `externalContracts_deposit_contract`

**Root cause:** `parse_overriding` returns `Overriding::Some(vec![])` when the override list is empty, which prints as `override()` — invalid Solidity.

---

### Category 4 — Normalization breaks renamed functions

**Affected tests:** `two_base_functions`, `struct_storage_ptr`, `calldata_bytes_internal`, `calldata_internal_library`, `calldata_internal_multi_array`, `calldata_internal_multi_fixed_array`, `calldata_array_two_dimensional`, `calldata_array_two_dimensional_1`, `bytes_in_constructors_packer`, `storage_array_ref`, `same_constants_different_files`, `errors_via_import`

**Root cause:** The normalization pipeline renames functions (`f` → `f_0`) but:
- `override`/`virtual` specifiers reference stale names
- Callees in some contexts (e.g., `this.test()`, `L.f()`, recursive calls) are not updated
- Import elimination merges declarations causing name conflicts

---

### Category 5 — Invalid Solidity test file

**Affected test:** `event_with_variables_of_internal_types`

**Root cause:** File contains invalid Solidity (`event E(function() internal)`).

---

### Category 6 — Slice expression parsing (`IndexRangeAccess`)

**Affected tests:** `decode_slice`, `abi_encode_call_uint_bytes`, `array_slice_calldata_as_argument_of_external_calls`, `bytes_concat_different_types`, `calldata_bound_dynamic_array_or_slice`

**Root cause:** The parser requires both `startExpression` and `endExpression` in `IndexRangeAccess` nodes, but Solidity allows open-ended slices like `x[4:]` (no end) or `x[:2]` (no start) or `x[:]` (neither).

| Error | Meaning |
|---|---|
| `Slice end expression not found` | `x[4:]` — start present, end absent |
| `Slice start expression not found` | `x[:2]` — start absent, end present |

**Fix:** Make both fields optional in the parser (at [ast_parser.rs:1468–1472](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/json_ast_parser/ast_parser.rs#L1468-L1472)).

---

### Category 7 — Type parser doesn't handle `literal_string` / `int_const`

**Affected tests:** `abi_encode_call_is_consistent`, `string_literal_assign_to_storage_bytes`

**Root cause:** The type parser cannot parse types like `tuple(int_const 1,literal_string "123")`. These are intermediate compiler types that appear in typeDescriptions.

**Fix:** Add `literal_string` and `int_const` handling to the type parser ([type_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/type_parser/type_parser.rs)).

---

### Category 8 — Empty tuple components

**Affected tests:** `calldata_slice_access`, `abi_encode_call` (`(bool success, )`)

**Root cause:** The AST printer outputs `(x, , )` for tuples with empty components, which is invalid Solidity. Empty tuple declarations like `(bool success, ) = ...;` should omit the trailing comma or the empty slot should not be printed.

---

### Category 9 — Constant initializer lost during parsing

**Affected tests:** `constant_string_at_file_level`, `constant_variables`

**Root cause:** Parsed constant declarations lose their initializer value, producing `bytes32 constant st;` instead of `bytes32 constant st = "test";`. Solidity requires constants to be initialized.

---

### Category 10 — Base constructor arguments dropped

**Affected tests:** `base_constructor_arguments`, `function_usage_in_constructor_arguments`, `constructor_with_params_inheritance_2`

**Root cause:** After parsing, base constructor arguments in inheritance specifiers (e.g., `contract Base is BaseBase(42)`) are dropped, so the re-printed code omits them, causing "No arguments passed to the base constructor" errors.

---

### Category 11 — Pragma version parsing

**Affected test:** `FixedFeeRegistrar`

**Root cause:** Pragma version `>=0.4.0<0.9.0` (without space before `<`) fails to parse.

---

### Category 12 — External source / unresolvable imports (test infrastructure)

**Affected tests:** `base64`, `prbmath_signed`, `prbmath_unsigned`, `strings`, `ramanujan_pi`, `PRBMathSD59x18`, `PRBMathUD60x18`, `external_import`, `multiple_equals_signs`, `multisource`, `non_normalized_paths`, `import_with_subdir`, `external_subdir_import`

**Root cause:** These tests use `==== ExternalSource: ... ====` directives or import files from external directories that aren't available during the test. The preprocessor doesn't handle these cases, causing compile failures. These should be excluded from auto-generated tests.

---

## Proposed Changes

### [MODIFY] [ast_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/json_ast_parser/ast_parser.rs)

- **(1–4)** Make `"type"` and `"parameters"` optional in Yul parsing functions
- **(5)** Fix `parse_overriding` to return `Overriding::All` for empty override lists
- **(7)** Make `startExpression`/`endExpression` optional in `IndexRangeAccess`

### [MODIFY] [type_parser.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/parsing/type_parser/type_parser.rs)

- **(8)** Handle `literal_string` and `int_const` types in tuple parsing

### [MODIFY] [defs.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/ast/defs.rs)

- **(9)** Fix tuple printing for empty components
- **(10)** Fix `VarDecl` Display to preserve constant initializers
- **(11)** Fix base constructor argument printing

### [MODIFY] [lower.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/src/solidity/lowering/lower.rs)

- **(6)** Strip `override`/`virtual` in `run_passes`

### [MODIFY] [build.rs](file:///home/trung/Workspace/sbip/verazt/crates/frontend/build.rs)

- **(12)** Fix pragma version parsing for missing spaces
- **(13)** Exclude external source tests and invalid Solidity files from test generation

---

## Verification Plan

### Automated Tests

```bash
# All ast_json tests
cargo test -p frontend -- ast_json 2>&1

# All memory_guard_tests
cargo test -p frontend -- memory_guard_tests 2>&1

# Semantic tests (sample from each category)
cargo test -p frontend -- semantic_tests 2>&1
```

### Regression

```bash
# Full frontend test suite
cargo test -p frontend 2>&1
```
