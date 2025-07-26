//! Integration libsolidity/syntax-tests

//--------------------------------------------------------------------
// ATTRIBUTES TO RELAX LINTING FOR UNIT TESTS
// Allow using `unwrap` function in unit tests
#![cfg_attr(feature = "linting", allow(clippy::unwrap_used))]
//---------------------------------------------------------------------

use super::test_utils::test_compiling_solidity_dir;

/// Test compiling `tests/libsolidity/syntax-tests/`
#[test]
fn main() {
    let dir = "tests/libsolidity/syntax-tests/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/abstract/`
#[test]
fn abstract_() {
    let dir = "tests/libsolidity/syntax-tests/abstract/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/array/`
#[test]
fn array() {
    for dir in &[
        "tests/libsolidity/syntax-tests/array/concat/",
        "tests/libsolidity/syntax-tests/array/invalid/",
        "tests/libsolidity/syntax-tests/array/invalidCopy/",
        "tests/libsolidity/syntax-tests/array/length/",
        "tests/libsolidity/syntax-tests/array/pop/",
        "tests/libsolidity/syntax-tests/array/push/",
        "tests/libsolidity/syntax-tests/array/slice/",
        "tests/libsolidity/syntax-tests/array/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19");
    }
}

/// Test compiling `tests/libsolidity/syntax-tests/bound/`
#[test]
fn bound() {
    let dir = "tests/libsolidity/syntax-tests/bound/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/bytecodeReferences/`
#[test]
fn bytecode_references() {
    let dir = "tests/libsolidity/syntax-tests/bytecodeReferences/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/comments/`
#[test]
fn comments() {
    let dir = "tests/libsolidity/syntax-tests/comments/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/constantEvaluator/`
#[test]
fn constant_evaluator() {
    let dir = "tests/libsolidity/syntax-tests/constantEvaluator/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/constants/`
#[test]
fn constants() {
    let dir = "tests/libsolidity/syntax-tests/constants/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/abiEncoder/`
#[test]
fn abi_encoder() {
    let dir = "tests/libsolidity/syntax-tests/abiEncoder/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/constructor/`
#[test]
fn constructor() {
    let dir = "tests/libsolidity/syntax-tests/constructor/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/controlFlow/`
#[test]
fn control_flow() {
    for dir in &[
        "tests/libsolidity/syntax-tests/controlFlow/",
        "tests/libsolidity/syntax-tests/controlFlow/calldataReturn/",
        "tests/libsolidity/syntax-tests/controlFlow/localCalldataVariables/",
        "tests/libsolidity/syntax-tests/controlFlow/localStorageVariables/",
        "tests/libsolidity/syntax-tests/controlFlow/mappingReturn/",
        "tests/libsolidity/syntax-tests/controlFlow/modifiers/",
        "tests/libsolidity/syntax-tests/controlFlow/storageReturn/",
        "tests/libsolidity/syntax-tests/controlFlow/uninitializedAccess/",
        "tests/libsolidity/syntax-tests/controlFlow/unreachableCode/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19");
    }
}

/// Test compiling `tests/libsolidity/syntax-tests/conversion/`
#[test]
fn conversion() {
    let dir = "tests/libsolidity/syntax-tests/conversion/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/dataLocations/`
#[test]
fn data_locations() {
    let dir = "tests/libsolidity/syntax-tests/dataLocations/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    for dir in &[
        "tests/libsolidity/syntax-tests/dataLocations/externalFunction/",
        "tests/libsolidity/syntax-tests/dataLocations/internalFunction/",
        "tests/libsolidity/syntax-tests/dataLocations/libraries/",
        "tests/libsolidity/syntax-tests/dataLocations/libraryExternalFunction/",
        "tests/libsolidity/syntax-tests/dataLocations/libraryInternalFunction/",
        "tests/libsolidity/syntax-tests/dataLocations/privateFunction/",
        "tests/libsolidity/syntax-tests/dataLocations/publicFunction/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19");
    }
}

/// Test compiling `tests/libsolidity/syntax-tests/denominations/`
#[test]
fn denominations() {
    let dir = "tests/libsolidity/syntax-tests/denominations/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/duplicateFunctions/`
#[test]
fn duplicate_functions() {
    let dir = "tests/libsolidity/syntax-tests/duplicateFunctions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/emit/`
#[test]
fn emit() {
    let dir = "tests/libsolidity/syntax-tests/emit/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/enums/`
#[test]
fn enums() {
    let dir = "tests/libsolidity/syntax-tests/enums/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/errors/`
#[test]
fn errors() {
    let dir = "tests/libsolidity/syntax-tests/errors/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/events/`
#[test]
fn events() {
    let dir = "tests/libsolidity/syntax-tests/events/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/fallback/`
#[test]
fn fallback() {
    let dir = "tests/libsolidity/syntax-tests/fallback/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/freeFunctions/`
#[test]
fn free_functions() {
    let dir = "tests/libsolidity/syntax-tests/freeFunctions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/functionCalls/`
#[test]
fn function_calls() {
    let dir = "tests/libsolidity/syntax-tests/functionCalls/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/functionTypes/`
#[test]
fn function_types() {
    let dir = "tests/libsolidity/syntax-tests/functionTypes/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/getter/`
#[test]
fn getter() {
    let dir = "tests/libsolidity/syntax-tests/getter/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/globalFunctions/`
#[test]
fn global_functions() {
    let dir = "tests/libsolidity/syntax-tests/globalFunctions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/iceRegressionTests/`
#[test]
fn ice_regression_tests() {
    let dir = "tests/libsolidity/syntax-tests/iceRegressionTests/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/syntax-tests/iceRegressionTests/declarationUnaryTuple";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/immutable/`
#[test]
fn immutable() {
    let dir = "tests/libsolidity/syntax-tests/immutable/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/imports/`
#[test]
fn imports() {
    let dir = "tests/libsolidity/syntax-tests/imports/";
    let skipped_tests = vec!["boost_filesystem_bug.sol"];
    test_compiling_solidity_dir(dir, skipped_tests, "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/indexing/`
#[test]
fn indexing() {
    let dir = "tests/libsolidity/syntax-tests/indexing/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/inheritance/`
#[test]
fn inheritance() {
    for dir in &[
        "tests/libsolidity/syntax-tests/inheritance/",
        "tests/libsolidity/syntax-tests/inheritance/dataLocation/",
        "tests/libsolidity/syntax-tests/inheritance/duplicated_constructor_call/",
        "tests/libsolidity/syntax-tests/inheritance/fallback_receive/",
        "tests/libsolidity/syntax-tests/inheritance/interface/",
        "tests/libsolidity/syntax-tests/inheritance/interface/diamond/",
        "tests/libsolidity/syntax-tests/inheritance/interface/implementation/",
        "tests/libsolidity/syntax-tests/inheritance/interface/linearization/",
        "tests/libsolidity/syntax-tests/inheritance/override/",
        "tests/libsolidity/syntax-tests/inheritance/virtual/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19");
    }
}

/// Test compiling `tests/libsolidity/syntax-tests/inline_arrays/`
#[test]
fn inline_arrays() {
    let dir = "tests/libsolidity/syntax-tests/inline_arrays/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/inlineAssembly/`
#[test]
fn inline_assembly() {
    let dir = "tests/libsolidity/syntax-tests/inlineAssembly/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/syntax-tests/inlineAssembly/invalid";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/syntax-tests/inlineAssembly/shadowing";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/largeTypes/`
#[test]
fn large_types() {
    let dir = "tests/libsolidity/syntax-tests/largeTypes/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/license/`
#[test]
fn license() {
    let dir = "tests/libsolidity/syntax-tests/license/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/literalOperations/`
#[test]
fn literal_operations() {
    let dir = "tests/libsolidity/syntax-tests/literalOperations/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/literals/`
#[test]
fn literals() {
    let dir = "tests/libsolidity/syntax-tests/literals/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/lvalues/`
#[test]
fn lvalues() {
    let dir = "tests/libsolidity/syntax-tests/lvalues/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/memberLookup/`
#[test]
fn member_lookup() {
    let dir = "tests/libsolidity/syntax-tests/memberLookup/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/metaTypes/`
#[test]
fn meta_types() {
    let dir = "tests/libsolidity/syntax-tests/metaTypes/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/modifiers/`
#[test]
fn modifiers() {
    let dir = "tests/libsolidity/syntax-tests/modifiers/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/multiSource/`
#[test]
fn multi_source() {
    let dir = "tests/libsolidity/syntax-tests/multiSource/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/multiVariableDeclaration/`
#[test]
fn multi_variable_declaration() {
    let dir = "tests/libsolidity/syntax-tests/multiVariableDeclaration/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/nameAndTypeResolution/`
#[test]
fn name_and_type_resolultion_main() {
    let dir = "tests/libsolidity/syntax-tests/nameAndTypeResolution/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/nameAndTypeResolution/`
#[test]
fn name_and_type_resolultion_sub_directories() {
    for dir in &[
        "tests/libsolidity/syntax-tests/nameAndTypeResolution/compoundAssignment/",
        "tests/libsolidity/syntax-tests/nameAndTypeResolution/invalidArgs/",
        "tests/libsolidity/syntax-tests/nameAndTypeResolution/invalidTypes/",
        "tests/libsolidity/syntax-tests/nameAndTypeResolution/shadowsBuiltin/",
        "tests/libsolidity/syntax-tests/nameAndTypeResolution/typeChecking/",
        "tests/libsolidity/syntax-tests/nameAndTypeResolution/warnUnused/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19")
    }
}

/// Test compiling `tests/libsolidity/syntax-tests/natspec/`
#[test]
fn natspec() {
    let dir = "tests/libsolidity/syntax-tests/natspec/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/parsing/`
#[test]
fn parsing() {
    let dir = "tests/libsolidity/syntax-tests/parsing/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/receiveEther/`
#[test]
fn receive_ether() {
    let dir = "tests/libsolidity/syntax-tests/receiveEther/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/returnExpressions/`
#[test]
fn return_expressions() {
    let dir = "tests/libsolidity/syntax-tests/returnExpressions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/revertStatement/`
#[test]
fn revert_statement() {
    let dir = "tests/libsolidity/syntax-tests/revertStatement/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/scoping/`
#[test]
fn scoping() {
    let dir = "tests/libsolidity/syntax-tests/scoping/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/shifts/`
#[test]
fn shifts() {
    let dir = "tests/libsolidity/syntax-tests/shifts/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/specialFunctions/`
#[test]
fn special_functions() {
    let dir = "tests/libsolidity/syntax-tests/specialFunctions/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/string/`
#[test]
fn string() {
    let dir = "tests/libsolidity/syntax-tests/string/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/structs/`
#[test]
fn structs() {
    let dir = "tests/libsolidity/syntax-tests/structs/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/super/`
#[test]
fn super_() {
    let dir = "tests/libsolidity/syntax-tests/super/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/tryCatch/`
#[test]
fn try_catch() {
    let dir = "tests/libsolidity/syntax-tests/tryCatch/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/tupleAssignments/`
#[test]
fn tuple_assignments() {
    let dir = "tests/libsolidity/syntax-tests/tupleAssignments/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/types/`
#[test]
fn types_main() {
    let dir = "tests/libsolidity/syntax-tests/types/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/types/` sub-directories
#[test]
fn types_sub_directories() {
    for dir in &[
        "tests/libsolidity/syntax-tests/types/contractTypeType/members/",
        "tests/libsolidity/syntax-tests/types/function_types/",
        "tests/libsolidity/syntax-tests/types/function_types/selector/",
        "tests/libsolidity/syntax-tests/types/mapping/",
    ] {
        test_compiling_solidity_dir(dir, vec![], "0.8.19")
    }
}

/// Test compiling `tests/libsolidity/syntax-tests/unchecked/`
#[test]
fn unchecked() {
    let dir = "tests/libsolidity/syntax-tests/unchecked/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/underscore/`
#[test]
fn underscore() {
    let dir = "tests/libsolidity/syntax-tests/underscore/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/unterminatedBlocks/`
#[test]
fn unterminated_blocks() {
    let dir = "tests/libsolidity/syntax-tests/unterminatedBlocks/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/unusedVariables/`
#[test]
fn unused_variables() {
    let dir = "tests/libsolidity/syntax-tests/unusedVariables/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/userDefinedValueType/`
#[test]
fn user_defined_value_type() {
    let dir = "tests/libsolidity/syntax-tests/userDefinedValueType/";

    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/using/`
#[test]
fn using() {
    let dir = "tests/libsolidity/syntax-tests/using/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/variableDeclaration/`
#[test]
fn variable_declaration() {
    let dir = "tests/libsolidity/syntax-tests/variableDeclaration/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/viewPureChecker/`
#[test]
fn view_pure_checker() {
    let dir = "tests/libsolidity/syntax-tests/viewPureChecker/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");

    let dir = "tests/libsolidity/syntax-tests/viewPureChecker/array/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/virtualLookup/`
#[test]
fn virtual_lookup() {
    let dir = "tests/libsolidity/syntax-tests/virtualLookup/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/syntax-tests/visibility/`
#[test]
fn visibility() {
    let dir = "tests/libsolidity/syntax-tests/visibility/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}
