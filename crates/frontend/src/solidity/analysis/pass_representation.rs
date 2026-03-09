//! Pass Representation Types
//!
//! This module defines the representation types that passes can operate on.

use std::fmt::{self, Display};

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

    /// Operates on both AST and IR
    Hybrid,
}

impl PassRepresentation {
    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            PassRepresentation::Ast => "AST",
            PassRepresentation::Ir => "IR",
            PassRepresentation::Hybrid => "Hybrid",
        }
    }

    /// Check if this representation requires IR.
    pub fn requires_ir(&self) -> bool {
        matches!(self, PassRepresentation::Ir | PassRepresentation::Hybrid)
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

    #[test]
    fn test_requires_ir() {
        assert!(!PassRepresentation::Ast.requires_ir());
        assert!(PassRepresentation::Ir.requires_ir());
        assert!(PassRepresentation::Hybrid.requires_ir());
    }

    #[test]
    fn test_uses_ast() {
        assert!(PassRepresentation::Ast.uses_ast());
        assert!(!PassRepresentation::Ir.uses_ast());
        assert!(PassRepresentation::Hybrid.uses_ast());
    }
}
