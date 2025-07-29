//! Integration tests for libsolidity/gas-tests

use super::test_utils::test_compiling_solidity_dir;

/// Test compiling contracts in `tests/libsolidity/gas-tests/`
#[test]
fn main() {
    let dir = "tests/libsolidity/gas-tests/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}
