//! CIR module — the top-level compilation unit.
//!
//! Mirrors `sir::Module` but contains canonicalized declarations.

use crate::cir::defs::CanonContractDecl;
use crate::sir::attrs::Attr;
use crate::sir::dialect::DialectDecl;
use crate::sir::utils::printer;
use std::fmt::{self, Display};
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════
// Module and Decl
// ═══════════════════════════════════════════════════════════════════

/// A CIR module — the top-level container for a canonicalized compilation unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonModule {
    pub id: String,
    pub attrs: Vec<Attr>,
    pub decls: Vec<CanonDecl>,
}

/// A top-level declaration within a CIR module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonDecl {
    Contract(CanonContractDecl),
    Dialect(DialectDecl),
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl CanonModule {
    pub fn new(id: &str, decls: Vec<CanonDecl>) -> Self {
        CanonModule { id: id.to_string(), attrs: vec![], decls }
    }

    pub fn name(&self) -> Option<String> {
        Path::new(&self.id)
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    pub fn print_pretty(&self) {
        printer::print_ir(&format!("{self}"));
    }
}

impl Display for CanonModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let decls = self
            .decls
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>()
            .join("\n\n");
        write!(f, "{decls}")
    }
}

impl CanonDecl {
    pub fn name(&self) -> &str {
        match self {
            CanonDecl::Contract(c) => &c.name,
            CanonDecl::Dialect(_) => "<dialect>",
        }
    }
}

impl Display for CanonDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonDecl::Contract(c) => write!(f, "{c}"),
            CanonDecl::Dialect(d) => write!(f, "{d}"),
        }
    }
}

impl From<CanonContractDecl> for CanonDecl {
    fn from(c: CanonContractDecl) -> Self {
        CanonDecl::Contract(c)
    }
}
