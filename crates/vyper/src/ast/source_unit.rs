//! Vyper AST top-level source unit.

use super::defs::*;
use super::loc::Loc;

/// A Vyper source unit (one `.vy` file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUnit {
    pub path: String,
    pub body: Vec<SourceUnitElem>,
    pub loc: Option<Loc>,
}

/// Top-level elements in a Vyper source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceUnitElem {
    VersionAnnotation(VersionAnnotation),
    Event(EventDef),
    Struct(StructDef),
    Interface(InterfaceDef),
    StateVar(StateVarDecl),
    Func(FuncDef),
    EnumDef(EnumDef),
    Flag(FlagDef),
    TypeAlias(TypeAlias),
    Import(ImportStmt),
}
