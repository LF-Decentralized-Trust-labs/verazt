//! Integration tests for libsolidity/memory-guard-tests

//--------------------------------------------------------------------
// ATTRIBUTES TO RELAX LINTING FOR UNIT TESTS
// Allow using `unwrap` function in unit tests
#![cfg_attr(feature = "linting", allow(clippy::unwrap_used))]
//---------------------------------------------------------------------

use super::test_utils::test_compiling_solidity_dir;

/// Test compiling `tests/libsolidity/memory-guard-tests/`
#[test]
fn main() {
    let dir = "tests/libsolidity/memory-guard-tests/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/memory-guard-tests/comment`
#[test]
fn comment() {
    let dir = "tests/libsolidity/memory-guard-tests/comment";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}

/// Test compiling `tests/libsolidity/memory-guard-tests/dialectString`
#[test]
fn dialect_string() {
    let dir = "tests/libsolidity/memory-guard-tests/dialectString";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}
