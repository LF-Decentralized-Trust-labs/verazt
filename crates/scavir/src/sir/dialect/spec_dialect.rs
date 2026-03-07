//! Spec dialect types for CIR.
//!
//! Provides formal verification type constructs used across all chain
//! dialects. Spec expressions (old, result, forall, exists) live in the
//! core grammar (`exprs.rs`); this module adds the spec-specific types.

use crate::sir::types::Type;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Spec dialect types
// ═══════════════════════════════════════════════════════════════════

/// Spec-specific types for formal verification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpecType {
    /// `!spec.prop` — type of a logical proposition.
    Prop,
    /// `!spec.ghost<T>` — ghost variable type (spec-only, not compiled).
    Ghost(Box<Type>),
}

impl Display for SpecType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecType::Prop => write!(f, "!spec.prop"),
            SpecType::Ghost(ty) => write!(f, "!spec.ghost<{ty}>"),
        }
    }
}
