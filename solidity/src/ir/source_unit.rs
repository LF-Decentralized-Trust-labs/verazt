//! Module handling a source unit.

use crate::ir::printer;
use crate::ir::*;
use std::{
    fmt::{self, Display},
    path::Path,
};

//-------------------------------------------------------------------------
// Data structures representing source unit
//-------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SourceUnit {
    pub path: String,
    pub elems: Vec<SourceUnitElem>,
}

#[derive(Debug, Clone)]
pub enum SourceUnitElem {
    ErrorDef(ErrorDef),
    FuncDef(FuncDef),
    VarDecl(VarDecl),
    StructDef(StructDef),
    EnumDef(EnumDef),
    ContractDef(ContractDef),
}

//-------------------------------------------------------------------------
// Implementations for SourceUnit
//-------------------------------------------------------------------------

impl SourceUnit {
    pub fn new(path: &str, elems: Vec<SourceUnitElem>) -> Self {
        SourceUnit { path: path.to_string(), elems }
    }

    pub fn name(&self) -> Option<String> {
        Path::new(&self.path)
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    pub fn print_pretty(&self) {
        println!("\nFile: {}", &self.path);
        printer::print_ir(&format!("{self}"));
    }
}

impl Display for SourceUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elems = self
            .elems
            .iter()
            .map(|elem| format!("{elem}"))
            .collect::<Vec<String>>()
            .join("\n\n");
        write!(f, "{elems}")
    }
}

//-------------------------------------------------------------------------
// Implementations for SourceUnitElem
//-------------------------------------------------------------------------

impl SourceUnitElem {}

impl From<ErrorDef> for SourceUnitElem {
    fn from(e: ErrorDef) -> Self {
        SourceUnitElem::ErrorDef(e)
    }
}

impl From<FuncDef> for SourceUnitElem {
    fn from(f: FuncDef) -> Self {
        SourceUnitElem::FuncDef(f)
    }
}

impl From<VarDecl> for SourceUnitElem {
    fn from(v: VarDecl) -> Self {
        SourceUnitElem::VarDecl(v)
    }
}

impl From<StructDef> for SourceUnitElem {
    fn from(s: StructDef) -> Self {
        SourceUnitElem::StructDef(s)
    }
}

impl From<EnumDef> for SourceUnitElem {
    fn from(e: EnumDef) -> Self {
        SourceUnitElem::EnumDef(e)
    }
}

impl From<ContractDef> for SourceUnitElem {
    fn from(c: ContractDef) -> Self {
        SourceUnitElem::ContractDef(c)
    }
}

impl Display for SourceUnitElem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceUnitElem::ContractDef(c) => write!(f, "{c}"),
            SourceUnitElem::FuncDef(fd) => write!(f, "{fd}"),
            SourceUnitElem::VarDecl(v) => write!(f, "{v};"),
            SourceUnitElem::StructDef(s) => write!(f, "{s}"),
            SourceUnitElem::EnumDef(e) => write!(f, "{e}"),
            SourceUnitElem::ErrorDef(e) => write!(f, "{e}"),
        }
    }
}
