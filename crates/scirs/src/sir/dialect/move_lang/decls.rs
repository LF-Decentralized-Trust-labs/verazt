//! Move dialect member declarations with enum-of-structs pattern.

use crate::sir::exprs::Expr;
use crate::sir::types::Type;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each MoveMemberDecl variant
// ═══════════════════════════════════════════════════════════════════

/// `move.struct_def` — struct declaration with abilities as `#move.abilities`
/// attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveStructDef {
    pub name: String,
    pub type_params: Vec<MoveTypeParam>,
    pub fields: Vec<(String, Type)>,
    pub loc: Loc,
}

/// `move.spec_fun` — spec-only function (not compiled, used in proofs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveSpecFun {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub body: Option<Expr>,
    pub loc: Loc,
}

/// `friend <module_path>` — friend declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveFriendDecl {
    pub path: String,
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// MoveMemberDecl enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// Move-specific member declarations inside a module/contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveMemberDecl {
    StructDef(MoveStructDef),
    SpecFun(MoveSpecFun),
    FriendDecl(MoveFriendDecl),
}

/// A type parameter in a Move struct or function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveTypeParam {
    pub name: String,
    /// Whether this is a phantom type parameter (`#move.phantom = true`).
    pub is_phantom: bool,
}

impl Display for MoveMemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveMemberDecl::StructDef(e) => {
                write!(f, "move.struct_def {}", e.name)?;
                if !e.type_params.is_empty() {
                    let params: Vec<_> = e
                        .type_params
                        .iter()
                        .map(|p| {
                            if p.is_phantom {
                                format!("phantom {}", p.name)
                            } else {
                                p.name.clone()
                            }
                        })
                        .collect();
                    write!(f, "<{}>", params.join(", "))?;
                }
                writeln!(f, " {{")?;
                for (n, t) in &e.fields {
                    writeln!(f, "    {n}: {t},")?;
                }
                write!(f, "}}")
            }
            MoveMemberDecl::SpecFun(e) => {
                let ps: Vec<_> = e.params.iter().map(|(n, t)| format!("{n}: {t}")).collect();
                write!(f, "spec fun {}({}): {}", e.name, ps.join(", "), e.ret)?;
                match &e.body {
                    Some(e) => write!(f, " {{ {e} }}"),
                    None => write!(f, ";"),
                }
            }
            MoveMemberDecl::FriendDecl(e) => write!(f, "friend {};", e.path),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Move dialect-level declarations (future use)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveDecl {
    /// Placeholder for future move-level declarations.
    _Placeholder,
}

impl Display for MoveDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveDecl::_Placeholder => write!(f, "// move decl placeholder"),
        }
    }
}
