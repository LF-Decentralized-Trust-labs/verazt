//! Pass Level Definitions
//!
//! This module defines the granularity levels at which passes operate.

use std::fmt::{self, Display};

/// Granularity level at which a pass operates.
///
/// Passes can operate at different levels of the AST/IR hierarchy,
/// from program-wide analysis down to individual variable tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PassLevel {
    /// Program level - multi-contract analysis
    /// AST: SourceUnit, IR: Module
    Program = 0,

    /// Contract level - single contract analysis
    /// AST: ContractDef, IR: Contract
    Contract = 1,

    /// Function level - single function analysis
    /// AST: FuncDef, IR: Function
    Function = 2,

    /// Block level - basic block analysis
    /// AST: Block, IR: BasicBlock
    Block = 3,

    /// Statement level - individual statement analysis
    /// AST: Stmt, IR: Instruction sequence
    Statement = 4,

    /// Expression level - individual expression analysis
    /// AST: Expr, IR: Instruction
    Expression = 5,

    /// Variable level - variable tracking
    /// AST: VarDecl, IR: SSA Variable
    Variable = 6,
}

impl PassLevel {
    /// Get the string representation of the level.
    pub fn as_str(&self) -> &'static str {
        match self {
            PassLevel::Program => "program",
            PassLevel::Contract => "contract",
            PassLevel::Function => "function",
            PassLevel::Block => "block",
            PassLevel::Statement => "statement",
            PassLevel::Expression => "expression",
            PassLevel::Variable => "variable",
        }
    }

    /// Get a description of the level.
    pub fn description(&self) -> &'static str {
        match self {
            PassLevel::Program => "Multi-contract analysis",
            PassLevel::Contract => "Single contract analysis",
            PassLevel::Function => "Single function analysis",
            PassLevel::Block => "Basic block analysis",
            PassLevel::Statement => "Individual statement analysis",
            PassLevel::Expression => "Individual expression analysis",
            PassLevel::Variable => "Variable tracking analysis",
        }
    }

    /// Check if this level is coarser than another.
    pub fn is_coarser_than(&self, other: &PassLevel) -> bool {
        (*self as u8) < (*other as u8)
    }

    /// Check if this level is finer than another.
    pub fn is_finer_than(&self, other: &PassLevel) -> bool {
        (*self as u8) > (*other as u8)
    }
}

impl Display for PassLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<u8> for PassLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => PassLevel::Program,
            1 => PassLevel::Contract,
            2 => PassLevel::Function,
            3 => PassLevel::Block,
            4 => PassLevel::Statement,
            5 => PassLevel::Expression,
            6 => PassLevel::Variable,
            _ => PassLevel::Program,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(PassLevel::Program.is_coarser_than(&PassLevel::Contract));
        assert!(PassLevel::Function.is_coarser_than(&PassLevel::Block));
        assert!(PassLevel::Variable.is_finer_than(&PassLevel::Expression));
    }

    #[test]
    fn test_level_display() {
        assert_eq!(PassLevel::Function.to_string(), "function");
        assert_eq!(PassLevel::Variable.to_string(), "variable");
    }
}
