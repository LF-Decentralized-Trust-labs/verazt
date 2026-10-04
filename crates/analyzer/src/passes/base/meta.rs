//! Pass Metadata Types
//!
//! This module defines the metadata types for passes:
//! - `PassLevel`: The granularity level at which a pass operates
//! - `PassRepresentation`: The representation a pass operates on

use std::fmt::{self, Display};

// =========================================================================
// PassLevel
// =========================================================================

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
}

impl Display for PassLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =========================================================================
// PassRepresentation
// =========================================================================

/// The IR a pass operates on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassRepresentation {
    /// SSA-form basic-block IR (lowered from SIR through CIR).
    Bir,

    /// Structured source-level IR (tree form).
    Sir,
}

impl PassRepresentation {
    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            PassRepresentation::Bir => "BIR",
            PassRepresentation::Sir => "SIR",
        }
    }
}

impl Display for PassRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_display() {
        assert_eq!(PassLevel::Function.to_string(), "function");
        assert_eq!(PassLevel::Variable.to_string(), "variable");
    }
}
