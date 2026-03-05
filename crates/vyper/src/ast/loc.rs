//! Vyper AST source location.

/// Source location from Vyper's JSON AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loc {
    pub lineno: u32,
    pub col_offset: u32,
    pub end_lineno: u32,
    pub end_col_offset: u32,
}
