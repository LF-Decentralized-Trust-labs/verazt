//! CIR definitions: canonicalized contracts, functions, storage, type aliases.
//!
//! Key differences from SIR definitions:
//! - `CanonContractDecl` has no `parents` field (inheritance resolved).
//! - `CanonMemberDecl` has no `Modifier` variant (modifiers inlined).
//! - `CanonFunctionDecl` body is always `Some` after normalization.

use crate::cir::exprs::CanonExpr;
use crate::cir::stmts::CanonStmt;
use crate::sir::attrs::Attr;
use crate::sir::dialect::DialectMemberDecl;
use crate::sir::loc::Span;
use crate::sir::spec::FuncSpec;
use crate::sir::types::Type;
use common::string::StringExt;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Contract declaration (inheritance resolved — no parents)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonContractDecl {
    pub name: String,
    // NOTE: no `parents` — inheritance is resolved.
    pub attrs: Vec<Attr>,
    pub members: Vec<CanonMemberDecl>,
    pub span: Option<Span>,
}

/// A member declaration inside a canonical contract.
///
/// The `Modifier` variant from SIR is absent — modifiers have been inlined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonMemberDecl {
    Storage(CanonStorageDecl),
    Function(CanonFunctionDecl),
    TypeAlias(CanonTypeAlias),
    GlobalInvariant(CanonExpr),
    Dialect(DialectMemberDecl),
}

// ═══════════════════════════════════════════════════════════════════
// Storage declaration
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonStorageDecl {
    pub name: String,
    pub ty: Type,
    pub init: Option<CanonExpr>,
    pub attrs: Vec<Attr>,
    pub span: Option<Span>,
}

// ═══════════════════════════════════════════════════════════════════
// Function declaration
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonFunctionDecl {
    pub name: String,
    pub type_params: Vec<CanonTypeParam>,
    pub params: Vec<CanonParam>,
    pub returns: Vec<Type>,
    pub attrs: Vec<Attr>,
    pub spec: Option<FuncSpec>,
    /// After normalization the body is always present.
    pub body: Vec<CanonStmt>,
    pub span: Option<Span>,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonParam {
    pub name: String,
    pub ty: Type,
}

/// A type parameter (for generic functions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonTypeParam {
    pub name: String,
}

// ═══════════════════════════════════════════════════════════════════
// Type alias
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonTypeAlias {
    pub name: String,
    pub ty: Type,
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl CanonContractDecl {
    pub fn new(name: String, members: Vec<CanonMemberDecl>, span: Option<Span>) -> Self {
        CanonContractDecl { name, attrs: vec![], members, span }
    }
}

impl CanonStorageDecl {
    pub fn new(name: String, ty: Type, init: Option<CanonExpr>, span: Option<Span>) -> Self {
        CanonStorageDecl { name, ty, init, attrs: vec![], span }
    }
}

impl CanonFunctionDecl {
    pub fn new(
        name: String,
        params: Vec<CanonParam>,
        returns: Vec<Type>,
        body: Vec<CanonStmt>,
        span: Option<Span>,
    ) -> Self {
        CanonFunctionDecl {
            name,
            type_params: vec![],
            params,
            returns,
            attrs: vec![],
            spec: None,
            body,
            span,
        }
    }
}

impl CanonParam {
    pub fn new(name: String, ty: Type) -> Self {
        CanonParam { name, ty }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Display implementations
// ═══════════════════════════════════════════════════════════════════

impl Display for CanonContractDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "contract {} {{", self.name)?;
        let mut prev_category = None;
        for m in &self.members {
            let cat = m.grouping_category();
            if let Some(prev) = prev_category {
                if prev != cat || cat == 4 {
                    writeln!(f)?;
                }
            }
            writeln!(f, "{}", format!("{m}").indent(2))?;
            prev_category = Some(cat);
        }
        write!(f, "}}")
    }
}

impl Display for CanonMemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonMemberDecl::Storage(s) => write!(f, "{s}"),
            CanonMemberDecl::Function(fd) => write!(f, "{fd}"),
            CanonMemberDecl::TypeAlias(ta) => write!(f, "type {} = {};", ta.name, ta.ty),
            CanonMemberDecl::GlobalInvariant(inv) => write!(f, "@invariant({inv})"),
            CanonMemberDecl::Dialect(d) => write!(f, "{d}"),
        }
    }
}

impl CanonMemberDecl {
    pub fn grouping_category(&self) -> u8 {
        match self {
            CanonMemberDecl::Storage(s) => match &s.ty {
                Type::Map(_, _) => 2,
                _ => 1,
            },
            CanonMemberDecl::Dialect(_) => 3,
            CanonMemberDecl::Function(_) => 4,
            CanonMemberDecl::TypeAlias(_) => 5,
            CanonMemberDecl::GlobalInvariant(_) => 6,
        }
    }
}

impl Display for CanonStorageDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.ty, self.name)?;
        if let Some(init) = &self.init {
            write!(f, " = {init}")?;
        }
        write!(f, ";")
    }
}

impl Display for CanonFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(spec) = &self.spec {
            write!(f, "{spec}")?;
        }
        write!(f, "function")?;
        if !self.name.is_empty() {
            write!(f, " {}", self.name)?;
        }
        let params: Vec<_> = self
            .params
            .iter()
            .map(|p| format!("{} {}", p.ty, p.name))
            .collect();
        write!(f, "({})", params.join(", "))?;
        if !self.returns.is_empty() {
            let rets: Vec<_> = self.returns.iter().map(|t| t.to_string()).collect();
            write!(f, " returns ({})", rets.join(", "))?;
        }
        writeln!(f, " {{")?;
        for s in &self.body {
            writeln!(f, "{}", format!("{s}").indent(2))?;
        }
        write!(f, "}}")
    }
}

impl Display for CanonParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.ty, self.name)
    }
}
