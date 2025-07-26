//! Integration tests for libsolidity/abi-json

//--------------------------------------------------------------------
// ATTRIBUTES TO RELAX LINTING FOR UNIT TESTS
// Allow using `unwrap` function in unit tests
#![cfg_attr(feature = "linting", allow(clippy::unwrap_used))]
//---------------------------------------------------------------------

use super::test_utils::test_compiling_solidity_dir;

/// Test compiling contracts in `tests/libsolidity/abi-json/`
#[test]
fn main() {
    let dir = "tests/libsolidity/abi-json/";
    test_compiling_solidity_dir(dir, vec![], "0.8.19");
}
