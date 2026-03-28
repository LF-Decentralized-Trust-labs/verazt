//! SIR definitions: contracts, functions, storage, type aliases.

use crate::sir::attrs::Attr;
use crate::sir::dialect::DialectMemberDecl;
use crate::sir::exprs::Expr;
use crate::sir::spec::FuncSpec;
use crate::sir::stmts::Stmt;
use crate::sir::types::Type;
use common::loc::Loc;
use common::string::StringExt;
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
    pub span: Option<Loc>,
}

/// A member declaration inside a contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberDecl {
    Storage(StorageDecl),
    Function(FunctionDecl),
    TypeAlias(TypeAlias),
    GlobalInvariant(Expr),
    Dialect(DialectMemberDecl),
    /// A `using Library for Type` directive (preserved before SIR → CIR
    /// lowering).
    UsingFor(UsingForDecl),
}

// ═══════════════════════════════════════════════════════════════════
// Using-for directive
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsingForDecl {
    /// The library or function name.
    pub library: String,
    /// Target type (`None` = `using for *`).
    pub target_type: Option<crate::sir::types::Type>,
    pub span: Option<Loc>,
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
    pub span: Option<Loc>,
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
    /// Modifier invocations on this function (preserved before SIR → CIR
    /// lowering).
    pub modifier_invocs: Vec<ModifierInvoc>,
    pub span: Option<Loc>,
}

// ═══════════════════════════════════════════════════════════════════
// Modifier invocation
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierInvoc {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Option<Loc>,
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
    pub fn new(name: String, members: Vec<MemberDecl>, span: Option<Loc>) -> Self {
        ContractDecl { name, parents: vec![], attrs: vec![], members, span }
    }

    // ─── Cross-chain structural helpers ────────────────────────

    /// Collect all storage variable names from this contract.
    pub fn storage_names(&self) -> Vec<String> {
        self.members
            .iter()
            .filter_map(|m| {
                if let MemberDecl::Storage(s) = m {
                    Some(s.name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Recursively check if any statement writes to storage (assigns to a
    /// storage variable name).
    pub fn has_storage_write(stmts: &[Stmt], storage_vars: &[String]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Assign(a) => {
                    if Self::expr_references_storage(&a.lhs, storage_vars) {
                        return true;
                    }
                }
                Stmt::AugAssign(a) => {
                    if Self::expr_references_storage(&a.lhs, storage_vars) {
                        return true;
                    }
                }
                Stmt::If(s) => {
                    if Self::has_storage_write(&s.then_body, storage_vars) {
                        return true;
                    }
                    if let Some(else_body) = &s.else_body {
                        if Self::has_storage_write(else_body, storage_vars) {
                            return true;
                        }
                    }
                }
                Stmt::While(s) => {
                    if Self::has_storage_write(&s.body, storage_vars) {
                        return true;
                    }
                }
                Stmt::For(s) => {
                    if Self::has_storage_write(&s.body, storage_vars) {
                        return true;
                    }
                }
                Stmt::Block(stmts) => {
                    if Self::has_storage_write(stmts, storage_vars) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Check if an expression references a storage variable.
    pub fn expr_references_storage(expr: &Expr, storage_vars: &[String]) -> bool {
        match expr {
            Expr::Var(v) => storage_vars.contains(&v.name),
            Expr::IndexAccess(ia) => Self::expr_references_storage(&ia.base, storage_vars),
            Expr::FieldAccess(fa) => Self::expr_references_storage(&fa.base, storage_vars),
            _ => false,
        }
    }

    /// Check if a function body contains an Assert before the first storage
    /// write.
    pub fn has_assert_before_storage_write(stmts: &[Stmt], storage_vars: &[String]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Assert(_) => return true,
                Stmt::Assign(a) => {
                    if Self::expr_references_storage(&a.lhs, storage_vars) {
                        return false;
                    }
                }
                Stmt::AugAssign(a) => {
                    if Self::expr_references_storage(&a.lhs, storage_vars) {
                        return false;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl StorageDecl {
    pub fn new(name: String, ty: Type, init: Option<Expr>, span: Option<Loc>) -> Self {
        StorageDecl { name, ty, init, attrs: vec![], span }
    }
}

impl FunctionDecl {
    pub fn new(
        name: String,
        params: Vec<Param>,
        returns: Vec<Type>,
        body: Option<Vec<Stmt>>,
        span: Option<Loc>,
    ) -> Self {
        FunctionDecl {
            name,
            type_params: vec![],
            params,
            returns,
            attrs: vec![],
            spec: None,
            body,
            modifier_invocs: vec![],
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

impl Display for MemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemberDecl::Storage(s) => write!(f, "{s}"),
            MemberDecl::Function(fd) => write!(f, "{fd}"),
            MemberDecl::TypeAlias(ta) => write!(f, "type {} = {};", ta.name, ta.ty),
            MemberDecl::GlobalInvariant(inv) => write!(f, "@invariant({inv})"),
            MemberDecl::Dialect(d) => write!(f, "{d}"),
            MemberDecl::UsingFor(u) => match &u.target_type {
                Some(ty) => write!(f, "using {} for {};", u.library, ty),
                None => write!(f, "using {} for *;", u.library),
            },
        }
    }
}

impl MemberDecl {
    pub fn grouping_category(&self) -> u8 {
        match self {
            MemberDecl::Storage(s) => match &s.ty {
                Type::Map(_, _) => 2,
                _ => 1,
            },
            MemberDecl::UsingFor(_) => 0,
            MemberDecl::Dialect(_) => 3,
            MemberDecl::Function(_) => 4,
            MemberDecl::TypeAlias(_) => 5,
            MemberDecl::GlobalInvariant(_) => 6,
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
        if !self.modifier_invocs.is_empty() {
            let mods: Vec<_> = self
                .modifier_invocs
                .iter()
                .map(|m| {
                    if m.args.is_empty() {
                        m.name.clone()
                    } else {
                        let args: Vec<_> = m.args.iter().map(|a| a.to_string()).collect();
                        format!("{}({})", m.name, args.join(", "))
                    }
                })
                .collect();
            write!(f, " {}", mods.join(" "))?;
        }
        match &self.body {
            None => write!(f, ";"),
            Some(stmts) => {
                writeln!(f, " {{")?;
                for s in stmts {
                    writeln!(f, "{}", format!("{s}").indent(2))?;
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
