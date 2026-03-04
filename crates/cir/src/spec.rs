//! Function specification for formal verification.

use crate::exprs::Expr;
use std::fmt::{self, Display};

/// Function specification — pre/post conditions and frame conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncSpec {
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub modifies: Vec<StorageRef>,
    pub decreases: Option<Expr>,
}

/// Identifies a storage slot for frame conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageRef {
    pub contract: Option<String>,
    pub field: String,
    /// `None` = wildcard (entire mapping/array).
    pub index: Option<Expr>,
}

impl FuncSpec {
    pub fn new() -> Self {
        FuncSpec { requires: vec![], ensures: vec![], modifies: vec![], decreases: None }
    }
}

impl Default for FuncSpec {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for FuncSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in &self.requires {
            writeln!(f, "@requires({r})")?;
        }
        for e in &self.ensures {
            writeln!(f, "@ensures({e})")?;
        }
        for m in &self.modifies {
            writeln!(f, "@modifies({m})")?;
        }
        if let Some(d) = &self.decreases {
            writeln!(f, "@decreases({d})")?;
        }
        Ok(())
    }
}

impl Display for StorageRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(c) = &self.contract {
            write!(f, "{c}.")?;
        }
        write!(f, "{}", self.field)?;
        if let Some(idx) = &self.index {
            write!(f, "[{idx}]")?;
        }
        Ok(())
    }
}
