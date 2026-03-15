//! SIR module — the top-level compilation unit.

use crate::sir::attrs::Attr;
use crate::sir::defs::ContractDecl;
use crate::sir::dialect::DialectDecl;
use crate::sir::utils::printer;
use std::fmt::{self, Display};
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════
// Module and Decl
// ═══════════════════════════════════════════════════════════════════

/// A SIR module — the top-level container for a compilation unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub id: String,
    pub attrs: Vec<Attr>,
    pub decls: Vec<Decl>,
}

/// A top-level declaration within a module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decl {
    Contract(ContractDecl),
    Dialect(DialectDecl),
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl Module {
    pub fn new(id: &str, decls: Vec<Decl>) -> Self {
        Module { id: id.to_string(), attrs: vec![], decls }
    }

    pub fn name(&self) -> Option<String> {
        Path::new(&self.id)
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    pub fn print_pretty(&self) {
        println!("\nModule: {}", &self.id);
        printer::print_ir(&format!("{self}"));
    }
}

impl Display for Module {
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

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Decl::Contract(c) => &c.name,
            Decl::Dialect(_) => "<dialect>",
        }
    }
}

impl Display for Decl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Decl::Contract(c) => write!(f, "{c}"),
            Decl::Dialect(d) => write!(f, "{d}"),
        }
    }
}

impl From<ContractDecl> for Decl {
    fn from(c: ContractDecl) -> Self {
        Decl::Contract(c)
    }
}
