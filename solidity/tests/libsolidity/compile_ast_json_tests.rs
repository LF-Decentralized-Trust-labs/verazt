//! Integration tests for libsolidity/ast-json

//--------------------------------------------------------------------
// ATTRIBUTES TO RELAX LINTING FOR UNIT TESTS
// Allow using `unwrap` function in unit tests
#![cfg_attr(feature = "linting", allow(clippy::unwrap_used))]
//---------------------------------------------------------------------

use super::test_utils::test_compiling_solidity_dir;

/// Test compiling `tests/libsolidity/ast-json`
#[test]
fn main() {
    let dir = "tests/libsolidity/ast-json/";
    let skipped_tests = vec!["event_with_variables_of_internal_types.sol"];
    test_compiling_solidity_dir(dir, skipped_tests, "0.8.19");
}

/// Test compiling `tests/libsolidity/ast-json/assembly`
#[test]
fn assembly() {
    let dir = "tests/libsolidity/ast-json/assembly/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}
