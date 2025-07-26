//! Module handling builtin Solidity structures like functions, variables, etc.

/// Module containing names of built-in keywords.
pub mod keywords {
    pub const THIS: &str = "this";
    pub const SELECTOR: &str = "selector";
    pub const SUPER: &str = "super";
}

/// Module containing names of built-in functions.
pub mod builtin_functions {
    pub const ADDMOD: &str = "addmod";
    pub const ASSERT: &str = "assert";
    pub const BYTES: &str = "bytes";
    pub const BYTES32: &str = "bytes32";
    pub const CALL: &str = "call";
    pub const ECRECOVER: &str = "ecrecover";
    pub const FALLBACK: &str = "fallback";
    pub const GASLEFT: &str = "gasleft";
    pub const KECCAK256: &str = "keccak256";
    pub const MULMOD: &str = "mulmod";
    pub const RECEIVE: &str = "receive";
    pub const REQUIRE: &str = "require";
    pub const REVERT: &str = "revert";
    pub const RIPEMD160: &str = "ripemd160";
    pub const SELFDESTRUCT: &str = "selfdestruct";
    pub const SHA256: &str = "sha256";
    pub const TYPE: &str = "type";
}

/// Check whether a function name is a builtin function.
pub fn is_builtin_function(func_name: &str) -> bool {
    use builtin_functions::*;
    matches!(
        func_name,
        ADDMOD
            | ASSERT
            | BYTES
            | BYTES32
            | CALL
            | ECRECOVER
            | FALLBACK
            | GASLEFT
            | KECCAK256
            | MULMOD
            | RECEIVE
            | REQUIRE
            | REVERT
            | RIPEMD160
            | SELFDESTRUCT
            | SHA256
            | TYPE
    )
}
