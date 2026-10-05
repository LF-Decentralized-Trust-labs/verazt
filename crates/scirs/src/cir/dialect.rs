//! CIR dialect remainder.
//!
//! SIR → CIR lowering maps chain-specific constructs to the shared feature
//! forms (`Load`, `Store`, internal/external calls, `Env`, `Emit`). What
//! does not generalize is kept here, typed per chain, with canonical
//! (`CanonExpr` / `CanonStmt`) operands.

use crate::cir::exprs::CanonExpr;
use crate::cir::stmts::CanonStmt;
use crate::semantics::EvmBuiltin;
use crate::sir::types::Type;
use common::loc::Loc;
use common::string::StringExt;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Expressions
// ═══════════════════════════════════════════════════════════════════

/// A chain-specific expression with its result type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonDialectExpr {
    pub kind: CanonDialectKind,
    pub ty: Type,
    pub span: Option<Loc>,
}

/// The chain-specific expression forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonDialectKind {
    Anchor(CanonAnchorExpr),
    Evm(CanonEvmExpr),
    Move(CanonMoveExpr),
}

/// EVM-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonEvmExpr {
    /// A pure EVM builtin (hashing, ABI coding, modular arithmetic, ...).
    Builtin(CanonEvmBuiltinExpr),
    /// Inline assembly text.
    InlineAsm(String),
}

/// An application of an EVM builtin function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonEvmBuiltinExpr {
    pub builtin: EvmBuiltin,
    pub args: Vec<CanonExpr>,
}

/// Move-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonMoveExpr {
    /// `borrow_global_mut<T>(addr)`: a mutable handle that may be written.
    BorrowGlobalMut(CanonMoveGlobalExpr),
    /// `exists<T>(addr)`.
    Exists(CanonMoveGlobalExpr),
    /// `move_from<T>(addr)`: removes and returns a global resource.
    MoveFrom(CanonMoveGlobalExpr),
    /// `signer::address_of(signer)`.
    SignerAddress(Box<CanonExpr>),
    /// `*reference = value`.
    WriteRef(CanonMoveWriteRefExpr),
}

/// A Move operation on the global resource of type `ty` under `addr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonMoveGlobalExpr {
    pub addr: Box<CanonExpr>,
    pub ty: Type,
}

/// A Move write through a mutable reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonMoveWriteRefExpr {
    pub reference: Box<CanonExpr>,
    pub value: Box<CanonExpr>,
}

/// Anchor (Solana)-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonAnchorExpr {
    /// A mutable account handle.
    AccountLoadMut(Box<CanonExpr>),
    /// `find_program_address(seeds, program_id)`.
    FindProgramAddress(CanonAnchorPdaExpr),
    /// `account.key()` of a signer account.
    SignerKey(Box<CanonExpr>),
}

/// A program-derived-address computation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonAnchorPdaExpr {
    pub program_id: Box<CanonExpr>,
    pub seeds: Vec<CanonExpr>,
}

// ═══════════════════════════════════════════════════════════════════
// Statements
// ═══════════════════════════════════════════════════════════════════

/// A chain-specific statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonDialectStmt {
    Evm(CanonEvmStmt),
    Move(CanonMoveStmt),
}

/// EVM-specific statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonEvmStmt {
    /// `selfdestruct(recipient)`.
    Selfdestruct(CanonSelfdestructStmt),
    /// `try guarded returns (...) { ... } catch ... { ... }`.
    TryCatch(CanonTryCatchStmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonSelfdestructStmt {
    pub recipient: CanonExpr,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonTryCatchStmt {
    pub guarded: CanonExpr,
    pub returns: Vec<(String, Type)>,
    pub body: Vec<CanonStmt>,
    pub catch_clauses: Vec<CanonCatchClause>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonCatchClause {
    pub error: Option<String>,
    pub params: Vec<(String, Type)>,
    pub body: Vec<CanonStmt>,
    pub span: Option<Loc>,
}

/// Move-specific statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonMoveStmt {
    /// `abort code`.
    Abort(CanonAbortStmt),
    /// A specification block (assertions only, not executable).
    SpecBlock(CanonSpecBlockStmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonAbortStmt {
    pub code: CanonExpr,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonSpecBlockStmt {
    pub assertions: Vec<CanonExpr>,
    pub span: Option<Loc>,
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl CanonDialectExpr {
    /// The sub-expressions this expression evaluates, in order.
    pub fn operands(&self) -> Vec<&CanonExpr> {
        match &self.kind {
            CanonDialectKind::Anchor(CanonAnchorExpr::AccountLoadMut(account))
            | CanonDialectKind::Anchor(CanonAnchorExpr::SignerKey(account)) => vec![account],
            CanonDialectKind::Anchor(CanonAnchorExpr::FindProgramAddress(pda)) => pda
                .seeds
                .iter()
                .chain(std::iter::once(&*pda.program_id))
                .collect(),
            CanonDialectKind::Evm(CanonEvmExpr::Builtin(builtin)) => builtin.args.iter().collect(),
            CanonDialectKind::Evm(CanonEvmExpr::InlineAsm(_)) => vec![],
            CanonDialectKind::Move(CanonMoveExpr::BorrowGlobalMut(global))
            | CanonDialectKind::Move(CanonMoveExpr::Exists(global))
            | CanonDialectKind::Move(CanonMoveExpr::MoveFrom(global)) => vec![&global.addr],
            CanonDialectKind::Move(CanonMoveExpr::SignerAddress(signer)) => vec![signer],
            CanonDialectKind::Move(CanonMoveExpr::WriteRef(write)) => {
                vec![&write.reference, &write.value]
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Display
// ═══════════════════════════════════════════════════════════════════

impl Display for CanonDialectExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            CanonDialectKind::Anchor(e) => write!(f, "{e}"),
            CanonDialectKind::Evm(e) => write!(f, "{e}"),
            CanonDialectKind::Move(e) => write!(f, "{e}"),
        }
    }
}

impl Display for CanonEvmExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonEvmExpr::Builtin(e) => write!(f, "evm.{:?}({})", e.builtin, join(&e.args)),
            CanonEvmExpr::InlineAsm(_) => write!(f, "evm.inline_asm"),
        }
    }
}

impl Display for CanonMoveExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonMoveExpr::BorrowGlobalMut(e) => {
                write!(f, "move.borrow_global_mut<{}>({})", e.ty, e.addr)
            }
            CanonMoveExpr::Exists(e) => write!(f, "move.exists<{}>({})", e.ty, e.addr),
            CanonMoveExpr::MoveFrom(e) => write!(f, "move.move_from<{}>({})", e.ty, e.addr),
            CanonMoveExpr::SignerAddress(e) => write!(f, "move.signer_address({e})"),
            CanonMoveExpr::WriteRef(e) => {
                write!(f, "move.write_ref({}, {})", e.reference, e.value)
            }
        }
    }
}

impl Display for CanonAnchorExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonAnchorExpr::AccountLoadMut(e) => write!(f, "anchor.account_load_mut({e})"),
            CanonAnchorExpr::FindProgramAddress(e) => {
                write!(f, "anchor.find_program_address([{}], {})", join(&e.seeds), e.program_id)
            }
            CanonAnchorExpr::SignerKey(e) => write!(f, "anchor.signer_key({e})"),
        }
    }
}

impl Display for CanonDialectStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonDialectStmt::Evm(CanonEvmStmt::Selfdestruct(s)) => {
                write!(f, "evm.selfdestruct({});", s.recipient)
            }
            CanonDialectStmt::Evm(CanonEvmStmt::TryCatch(s)) => write!(f, "{s}"),
            CanonDialectStmt::Move(CanonMoveStmt::Abort(s)) => {
                write!(f, "move.abort({});", s.code)
            }
            CanonDialectStmt::Move(CanonMoveStmt::SpecBlock(s)) => {
                write!(f, "spec {{ {} }}", join(&s.assertions))
            }
        }
    }
}

impl Display for CanonTryCatchStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "try {}", self.guarded)?;
        if !self.returns.is_empty() {
            write!(f, " returns ({})", join_params(&self.returns))?;
        }
        write_body(f, &self.body)?;
        for clause in &self.catch_clauses {
            write!(
                f,
                " catch {}({})",
                clause.error.as_deref().unwrap_or(""),
                join_params(&clause.params)
            )?;
            write_body(f, &clause.body)?;
        }
        Ok(())
    }
}

fn join(exprs: &[CanonExpr]) -> String {
    exprs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn join_params(params: &[(String, Type)]) -> String {
    params
        .iter()
        .map(|(name, ty)| format!("{ty} {name}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn write_body(f: &mut fmt::Formatter<'_>, body: &[CanonStmt]) -> fmt::Result {
    writeln!(f, " {{")?;
    for stmt in body {
        writeln!(f, "{}", format!("{stmt}").indent(2))?;
    }
    write!(f, "}}")
}
