//! Move dialect for SIR.
//!
//! Covers Move language constructs for both Aptos and Sui targets.
//! The chain-specific distinction is made via `#sir.chain_target`
//! module attribute (`"aptos"` or `"sui"`). Framework-level calls
//! (e.g., `aptos_framework::coin::deposit` vs `sui::transfer::transfer`)
//! are represented as regular `FunctionCall` expressions with fully
//! qualified names, resolved at the compilation frontend.

use crate::sir::exprs::Expr;
use crate::sir::types::Type;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Move dialect types
// ═══════════════════════════════════════════════════════════════════

/// Move-specific types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MoveType {
    /// `!move.resource<T>` — struct type with key ability (lives in global
    /// storage).
    Resource(Box<Type>),
    /// `!move.signer` — signer capability passed to entry functions.
    Signer,
    /// `!move.type_tag` — phantom type tag (used in forall quantifiers in
    /// specs).
    TypeTag,
}

impl Display for MoveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveType::Resource(ty) => write!(f, "!move.resource<{ty}>"),
            MoveType::Signer => write!(f, "!move.signer"),
            MoveType::TypeTag => write!(f, "!move.type_tag"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Move dialect expressions
// ═══════════════════════════════════════════════════════════════════

/// Move-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveExpr {
    /// `move.borrow_global<T>(addr)` — immutable global storage borrow.
    BorrowGlobal { ty: Type, addr: Box<Expr> },
    /// `move.borrow_global_mut<T>(addr)` — mutable global storage borrow.
    BorrowGlobalMut { ty: Type, addr: Box<Expr> },
    /// `move.exists<T>(addr)` — check if resource exists at address.
    Exists { ty: Type, addr: Box<Expr> },
    /// `move.signer_address(signer)` — get address from signer.
    SignerAddress(Box<Expr>),
    /// `move.move_to(resource, signer)` — publish resource to signer's address.
    MoveTo {
        resource: Box<Expr>,
        signer: Box<Expr>,
    },
    /// `move.move_from<T>(addr)` — remove resource from address.
    MoveFrom { ty: Type, addr: Box<Expr> },
    /// `move.write_ref(ref, value)` — write through a mutable reference.
    WriteRef {
        reference: Box<Expr>,
        value: Box<Expr>,
    },
    /// `move.ghost_var(name)` — spec-only ghost variable.
    GhostVar(String),
}

impl Display for MoveExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveExpr::BorrowGlobal { ty, addr } => {
                write!(f, "move.borrow_global<{ty}>({addr})")
            }
            MoveExpr::BorrowGlobalMut { ty, addr } => {
                write!(f, "move.borrow_global_mut<{ty}>({addr})")
            }
            MoveExpr::Exists { ty, addr } => write!(f, "move.exists<{ty}>({addr})"),
            MoveExpr::SignerAddress(e) => write!(f, "move.signer_address({e})"),
            MoveExpr::MoveTo { resource, signer } => {
                write!(f, "move.move_to({resource}, {signer})")
            }
            MoveExpr::MoveFrom { ty, addr } => {
                write!(f, "move.move_from<{ty}>({addr})")
            }
            MoveExpr::WriteRef { reference, value } => {
                write!(f, "move.write_ref({reference}, {value})")
            }
            MoveExpr::GhostVar(name) => write!(f, "move.ghost_var({name})"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Move dialect statements
// ═══════════════════════════════════════════════════════════════════

/// Move-specific statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveStmt {
    /// `abort code` — terminate with error code.
    Abort(Box<Expr>),
    /// `spec { ... }` — spec block embedded in function body.
    SpecBlock { assertions: Vec<Expr> },
}

impl Display for MoveStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveStmt::Abort(e) => write!(f, "abort {e};"),
            MoveStmt::SpecBlock { assertions } => {
                writeln!(f, "spec {{")?;
                for a in assertions {
                    writeln!(f, "    {a};")?;
                }
                write!(f, "}}")
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Move dialect member declarations
// ═══════════════════════════════════════════════════════════════════

/// Move-specific member declarations inside a module/contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveMemberDecl {
    /// `move.struct_def` — struct declaration with abilities as
    /// `#move.abilities` attribute.
    StructDef {
        name: String,
        type_params: Vec<MoveTypeParam>,
        fields: Vec<(String, Type)>,
    },
    /// `move.spec_fun` — spec-only function (not compiled, used in proofs).
    SpecFun {
        name: String,
        params: Vec<(String, Type)>,
        ret: Type,
        body: Option<Expr>,
    },
    /// `friend <module_path>` — friend declaration.
    FriendDecl(String),
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
            MoveMemberDecl::StructDef { name, type_params, fields } => {
                write!(f, "move.struct_def {name}")?;
                if !type_params.is_empty() {
                    let params: Vec<_> = type_params
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
                for (n, t) in fields {
                    writeln!(f, "    {n}: {t},")?;
                }
                write!(f, "}}")
            }
            MoveMemberDecl::SpecFun { name, params, ret, body } => {
                let ps: Vec<_> = params.iter().map(|(n, t)| format!("{n}: {t}")).collect();
                write!(f, "spec fun {name}({}): {ret}", ps.join(", "))?;
                match body {
                    Some(e) => write!(f, " {{ {e} }}"),
                    None => write!(f, ";"),
                }
            }
            MoveMemberDecl::FriendDecl(path) => write!(f, "friend {path};"),
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
