//! EVM dialect member declarations with enum-of-structs pattern.

use crate::sir::stmts::Stmt;
use crate::sir::types::Type;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each EvmMemberDecl variant
// ═══════════════════════════════════════════════════════════════════

/// An event definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmEventDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub indexed: Vec<bool>,
    pub anonymous: bool,
    pub loc: Loc,
}

/// A modifier definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmModifierDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub body: Vec<Stmt>,
    pub loc: Loc,
}

/// An error definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmErrorDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub loc: Loc,
}

/// An enum definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmEnumDef {
    pub name: String,
    pub variants: Vec<String>,
    pub loc: Loc,
}

/// A struct definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmStructDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// EvmMemberDecl enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// EVM-specific member declarations inside a contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmMemberDecl {
    EventDef(EvmEventDef),
    ModifierDef(EvmModifierDef),
    ErrorDef(EvmErrorDef),
    EnumDef(EvmEnumDef),
    StructDef(EvmStructDef),
}

impl Display for EvmMemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmMemberDecl::EventDef(e) => {
                let ps: Vec<_> = e.params.iter().map(|(n, t)| format!("{t} {n}")).collect();
                write!(f, "event {}({});", e.name, ps.join(", "))
            }
            EvmMemberDecl::ModifierDef(e) => {
                write!(f, "modifier {}(...) {{ ... }}", e.name)
            }
            EvmMemberDecl::ErrorDef(e) => {
                let ps: Vec<_> = e.params.iter().map(|(n, t)| format!("{t} {n}")).collect();
                write!(f, "error {}({});", e.name, ps.join(", "))
            }
            EvmMemberDecl::EnumDef(e) => {
                write!(f, "enum {} {{ {} }}", e.name, e.variants.join(", "))
            }
            EvmMemberDecl::StructDef(e) => {
                let fs: Vec<_> = e.fields.iter().map(|(n, t)| format!("{t} {n}")).collect();
                write!(f, "struct {} {{ {} }}", e.name, fs.join("; "))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// EVM dialect-level declarations (future use)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmDecl {
    _Placeholder,
}

impl Display for EvmDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmDecl::_Placeholder => write!(f, "// evm decl placeholder"),
        }
    }
}
