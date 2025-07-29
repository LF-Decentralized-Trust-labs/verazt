//! Integration tests for libsolidity/memory-guard-tests

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
