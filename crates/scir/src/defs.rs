//! CIR definitions: contracts, functions, storage, type aliases.

use crate::attrs::Attr;
use crate::dialect::DialectMemberDecl;
use crate::exprs::Expr;
use crate::loc::Span;
use crate::spec::FuncSpec;
use crate::stmts::Stmt;
use crate::types::Type;
use extlib::string::StringExt;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Contract declaration
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDecl {
    pub name: String,
    pub parents: Vec<String>,
    pub attrs: Vec<Attr>,
    pub members: Vec<MemberDecl>,
    pub span: Option<Span>,
}

/// A member declaration inside a contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberDecl {
    Storage(StorageDecl),
    Function(FunctionDecl),
    TypeAlias(TypeAlias),
    GlobalInvariant(Expr),
    Dialect(DialectMemberDecl),
}

// ═══════════════════════════════════════════════════════════════════
// Storage declaration (state variable)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageDecl {
    pub name: String,
    pub ty: Type,
    pub init: Option<Expr>,
    pub attrs: Vec<Attr>,
    pub span: Option<Span>,
}

// ═══════════════════════════════════════════════════════════════════
// Function declaration
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub returns: Vec<Type>,
    pub attrs: Vec<Attr>,
    pub spec: Option<FuncSpec>,
    pub body: Option<Vec<Stmt>>,
    pub span: Option<Span>,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

/// A type parameter (for generic functions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParam {
    pub name: String,
}

// ═══════════════════════════════════════════════════════════════════
// Type alias
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias {
    pub name: String,
    pub ty: Type,
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl ContractDecl {
    pub fn new(name: String, members: Vec<MemberDecl>, span: Option<Span>) -> Self {
        ContractDecl { name, parents: vec![], attrs: vec![], members, span }
    }
}

impl StorageDecl {
    pub fn new(name: String, ty: Type, init: Option<Expr>, span: Option<Span>) -> Self {
        StorageDecl { name, ty, init, attrs: vec![], span }
    }
}

impl FunctionDecl {
    pub fn new(
        name: String,
        params: Vec<Param>,
        returns: Vec<Type>,
        body: Option<Vec<Stmt>>,
        span: Option<Span>,
    ) -> Self {
        FunctionDecl {
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

impl Param {
    pub fn new(name: String, ty: Type) -> Self {
        Param { name, ty }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Display implementations
// ═══════════════════════════════════════════════════════════════════

impl Display for ContractDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "contract {}", self.name)?;
        if !self.parents.is_empty() {
            write!(f, " is {}", self.parents.join(", "))?;
        }
        writeln!(f, " {{")?;
        for m in &self.members {
            writeln!(f, "{}", format!("{m}").indent(4))?;
        }
        write!(f, "}}")
    }
}

impl Display for MemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemberDecl::Storage(s) => write!(f, "{s}"),
            MemberDecl::Function(fd) => write!(f, "{fd}"),
            MemberDecl::TypeAlias(ta) => write!(f, "type {} = {};", ta.name, ta.ty),
            MemberDecl::GlobalInvariant(inv) => write!(f, "@invariant({inv})"),
            MemberDecl::Dialect(d) => write!(f, "{d}"),
        }
    }
}

impl Display for StorageDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.ty, self.name)?;
        if let Some(init) = &self.init {
            write!(f, " = {init}")?;
        }
        write!(f, ";")
    }
}

impl Display for FunctionDecl {
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
        match &self.body {
            None => write!(f, ";"),
            Some(stmts) => {
                writeln!(f, " {{")?;
                for s in stmts {
                    writeln!(f, "{}", format!("{s}").indent(4))?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.ty, self.name)
    }
}
