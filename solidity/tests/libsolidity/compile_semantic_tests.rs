//! Integration test for libsolidity/semantic-tests

use super::test_utils::test_compiling_solidity_dir;

/// Test compiling files in the root folder `tests/libsolidity/semantic-tests/`
#[test]
fn root() {
    let dir = "tests/libsolidity/semantic-tests/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/abiencodedecode/`
#[test]
fn abi_encode_decode() {
    let dir = "tests/libsolidity/semantic-tests/abiencodedecode/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/abiEncoderV1/`
#[test]
fn abi_encoder_v1() {
    for dir in &[
        "tests/libsolidity/semantic-tests/abiEncoderV1/",
        "tests/libsolidity/semantic-tests/abiEncoderV1/cleanup/",
        "tests/libsolidity/semantic-tests/abiEncoderV1/struct/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19")
    }
}

/// Test compiling `tests/libsolidity/semantic-tests/abiEncoderV2/`
#[test]
fn abi_encoder_v2() {
    for dir in &[
        "tests/libsolidity/semantic-tests/abiEncoderV2/",
        "tests/libsolidity/semantic-tests/abiEncoderV1/cleanup/",
        "tests/libsolidity/semantic-tests/abiEncoderV1/struct/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19")
    }
}

/// Test compiling `tests/libsolidity/semantic-tests/accessor/`
#[test]
fn accessor() {
    let dir = "tests/libsolidity/semantic-tests/accessor";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/arithmetics/`
#[test]
fn arithmetics() {
    let dir = "tests/libsolidity/semantic-tests/arithmetics";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/array/`
#[test]
fn array() {
    let dir = "tests/libsolidity/semantic-tests/array";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    for dir in &[
        "tests/libsolidity/semantic-tests/array/concat/",
        "tests/libsolidity/semantic-tests/array/copying/",
        "tests/libsolidity/semantic-tests/array/delete",
        "tests/libsolidity/semantic-tests/array/indexAccess",
        "tests/libsolidity/semantic-tests/array/pop",
        "tests/libsolidity/semantic-tests/array/push",
        "tests/libsolidity/semantic-tests/array/slices",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19")
    }
}

/// Test compiling `tests/libsolidity/semantic-tests/asmForLoop/`
#[test]
fn asm_for_loop() {
    let dir = "tests/libsolidity/semantic-tests/asmForLoop/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/builtinFunctions/`
#[test]
fn builtin_functions() {
    let dir = "tests/libsolidity/semantic-tests/builtinFunctions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/calldata`
#[test]
fn calldata() {
    let dir = "tests/libsolidity/semantic-tests/calldata/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/cleanup/`
#[test]
fn cleanup() {
    let dir = "tests/libsolidity/semantic-tests/cleanup/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/constantEvaluator/`
#[test]
fn constant_evaluator() {
    let dir = "tests/libsolidity/semantic-tests/constantEvaluator/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/constants/`
#[test]
fn constants() {
    let dir = "tests/libsolidity/semantic-tests/constants/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/constructor/`
#[test]
fn constructor() {
    let dir = "tests/libsolidity/semantic-tests/constructor/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/conversions/`
#[test]
fn conversions() {
    let dir = "tests/libsolidity/semantic-tests/conversions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/ecrecover/`
#[test]
fn ecrecover() {
    let dir = "tests/libsolidity/semantic-tests/ecrecover/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/enums/`
#[test]
fn enums() {
    let dir = "tests/libsolidity/semantic-tests/enums/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/errors/`
#[test]
fn errors() {
    let dir = "tests/libsolidity/semantic-tests/errors/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/events/`
#[test]
fn events() {
    let dir = "tests/libsolidity/semantic-tests/events/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/exponentiation/`
#[test]
fn exponentiation() {
    let dir = "tests/libsolidity/semantic-tests/exponentiation/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/expressions/`
#[test]
fn expressions() {
    let dir = "tests/libsolidity/semantic-tests/expressions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/fallback/`
#[test]
fn fallback() {
    let dir = "tests/libsolidity/semantic-tests/fallback/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/freeFunctions/`
#[test]
fn free_functions() {
    let dir = "tests/libsolidity/semantic-tests/freeFunctions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/functionCall/`
#[test]
fn function_call() {
    let dir = "tests/libsolidity/semantic-tests/functionCall/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/semantic-tests/functionCall/inheritance";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/functionSelector/`
#[test]
fn function_selector() {
    let dir = "tests/libsolidity/semantic-tests/functionSelector/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/functionTypes/`
#[test]
fn function_types() {
    let dir = "tests/libsolidity/semantic-tests/functionTypes/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/getters/`
#[test]
fn getters() {
    let dir = "tests/libsolidity/semantic-tests/getters/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/immutable/`
#[test]
fn immutable() {
    let dir = "tests/libsolidity/semantic-tests/immutable/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/inheritance/`
#[test]
fn inheritance() {
    let dir = "tests/libsolidity/semantic-tests/inheritance/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/semantic-tests/inheritance/dataLocation";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/inlineAssembly/`
#[test]
fn inline_assembly() {
    let dir = "tests/libsolidity/semantic-tests/inlineAssembly/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/integer/`
#[test]
fn integer() {
    let dir = "tests/libsolidity/semantic-tests/integer/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/interfaceID/`
#[test]
fn interface_id() {
    let dir = "tests/libsolidity/semantic-tests/interfaceID/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/isoltestTesting/`
#[test]
fn isoltest_testing() {
    let dir = "tests/libsolidity/semantic-tests/isoltestTesting/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/semantic-tests/isoltestTesting/storage";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/libraries/`
#[test]
fn libraries() {
    let dir = "tests/libsolidity/semantic-tests/libraries/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/literals/`
#[test]
fn literals() {
    let dir = "tests/libsolidity/semantic-tests/literals/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/memoryManagement/`
#[test]
fn memory_management() {
    let dir = "tests/libsolidity/semantic-tests/memoryManagement/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/metaTypes/`
#[test]
fn meta_types() {
    let dir = "tests/libsolidity/semantic-tests/metaTypes/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/modifiers/`
#[test]
fn modifiers() {
    let dir = "tests/libsolidity/semantic-tests/modifiers/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/multiSource/`
#[test]
fn multi_source() {
    let dir = "tests/libsolidity/semantic-tests/multiSource/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/operators/`
#[test]
fn operators() {
    let dir = "tests/libsolidity/semantic-tests/operators/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/semantic-tests/operators/shifts";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/optimizer/`
#[test]
fn optimizer() {
    let dir = "tests/libsolidity/semantic-tests/optimizer/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/payable/`
#[test]
fn payable() {
    let dir = "tests/libsolidity/semantic-tests/payable/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/receive/`
#[test]
fn receive() {
    let dir = "tests/libsolidity/semantic-tests/receive/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/reverts/`
#[test]
fn reverts() {
    let dir = "tests/libsolidity/semantic-tests/reverts/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/revertStrings/`
#[test]
fn revert_strings() {
    let dir = "tests/libsolidity/semantic-tests/revertStrings/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/salted_create/`
#[test]
fn salted_create() {
    let dir = "tests/libsolidity/semantic-tests/salted_create/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/smoke/`
#[test]
fn smoke() {
    let dir = "tests/libsolidity/semantic-tests/smoke/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/specialFunctions/`
#[test]
fn special_functions() {
    let dir = "tests/libsolidity/semantic-tests/specialFunctions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/state/`
#[test]
fn state() {
    let dir = "tests/libsolidity/semantic-tests/state/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/statements/`
#[test]
fn statements() {
    let dir = "tests/libsolidity/semantic-tests/statements/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/storage/`
#[test]
fn storage() {
    let dir = "tests/libsolidity/semantic-tests/storage/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/strings/`
#[test]
fn strings() {
    let dir = "tests/libsolidity/semantic-tests/strings/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/semantic-tests/strings/concat";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/structs/`
#[test]
fn structs() {
    let dir = "tests/libsolidity/semantic-tests/structs/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/semantic-tests/structs/calldata";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/semantic-tests/structs/conversion";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/tryCatch`
#[test]
fn try_catch() {
    let dir = "tests/libsolidity/semantic-tests/tryCatch/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/types/`
#[test]
fn types() {
    let dir = "tests/libsolidity/semantic-tests/types/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/semantic-tests/types/mapping/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/underscore/`
#[test]
fn underscore() {
    let dir = "tests/libsolidity/semantic-tests/underscore/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling
/// `tests/libsolidity/semantic-tests/uninitializedFunctionPointer/`
#[test]
fn uninitialized_function_pointer() {
    let dir = "tests/libsolidity/semantic-tests/uninitializedFunctionPointer/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/userDefinedValueType/`
#[test]
fn user_defined_value_type() {
    let dir = "tests/libsolidity/semantic-tests/userDefinedValueType/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/using/`
#[test]
fn using() {
    let dir = "tests/libsolidity/semantic-tests/using/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/variables/`
#[test]
fn variables() {
    let dir = "tests/libsolidity/semantic-tests/variables/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/various/`
#[test]
fn various() {
    let dir = "tests/libsolidity/semantic-tests/various/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/semantic-tests/viaYul/`
#[test]
fn via_yul() {
    for dir in &[
        "tests/libsolidity/semantic-tests/viaYul/",
        "tests/libsolidity/semantic-tests/viaYul/array_memory_allocation/",
        "tests/libsolidity/semantic-tests/viaYul/cleanup/",
        "tests/libsolidity/semantic-tests/viaYul/conditional/",
        "tests/libsolidity/semantic-tests/viaYul/conversion/",
        "tests/libsolidity/semantic-tests/viaYul/loops/",
        "tests/libsolidity/semantic-tests/viaYul/storage/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19")
    }
}

/// Test compiling `tests/libsolidity/semantic-tests/virtualFunctions/`
#[test]
fn virtual_functions() {
    let dir = "tests/libsolidity/semantic-tests/virtualFunctions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}
