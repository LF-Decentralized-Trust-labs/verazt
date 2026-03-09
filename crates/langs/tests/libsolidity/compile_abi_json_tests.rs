//! Integration tests for libsolidity/abi-json

use super::test_utils::test_compiling_solidity_dir;

/// Test compiling contracts in `tests/libsolidity/abi-json/`
#[test]
fn main() {
    let dir = "tests/libsolidity/abi-json/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}
