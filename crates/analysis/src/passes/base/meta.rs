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

// =========================================================================
// PassRepresentation
// =========================================================================

/// The representation a pass operates on.
///
/// Passes can work on AST (high-level source structure), IR (low-level
/// normalized form), or both (hybrid passes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassRepresentation {
    /// Operates on AST only
    Ast,

    /// Operates on IR only (requires IR generation)
    Ir,

    /// Operates on BIR (requires BIR generation)
    Air,

    /// Operates on both AST and IR
    Hybrid,
}

impl PassRepresentation {
    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            PassRepresentation::Ast => "AST",
            PassRepresentation::Ir => "IR",
            PassRepresentation::Air => "BIR",
            PassRepresentation::Hybrid => "Hybrid",
        }
    }

    /// Check if this representation requires IR.
    pub fn requires_ir(&self) -> bool {
        matches!(
            self,
            PassRepresentation::Ir | PassRepresentation::Air | PassRepresentation::Hybrid
        )
    }

    /// Check if this representation requires BIR.
    pub fn requires_air(&self) -> bool {
        matches!(self, PassRepresentation::Air)
    }

    /// Check if this representation uses AST.
    pub fn uses_ast(&self) -> bool {
        matches!(self, PassRepresentation::Ast | PassRepresentation::Hybrid)
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

    // PassLevel tests
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

    // PassRepresentation tests
    #[test]
    fn test_requires_ir() {
        assert!(!PassRepresentation::Ast.requires_ir());
        assert!(PassRepresentation::Ir.requires_ir());
        assert!(PassRepresentation::Air.requires_ir());
        assert!(PassRepresentation::Hybrid.requires_ir());
    }

    #[test]
    fn test_requires_air() {
        assert!(!PassRepresentation::Ast.requires_air());
        assert!(!PassRepresentation::Ir.requires_air());
        assert!(PassRepresentation::Air.requires_air());
        assert!(!PassRepresentation::Hybrid.requires_air());
    }

    #[test]
    fn test_uses_ast() {
        assert!(PassRepresentation::Ast.uses_ast());
        assert!(!PassRepresentation::Ir.uses_ast());
        assert!(!PassRepresentation::Air.uses_ast());
        assert!(PassRepresentation::Hybrid.uses_ast());
    }
}
