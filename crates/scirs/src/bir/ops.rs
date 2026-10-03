//! SSA Ops for BIR.
//!
//! Each SIR statement/expression lowers to one or more `Op` values.
//!
//! Ops come in three groups:
//! - core ops (constants, arithmetic, control-related values),
//! - shared feature ops (`Load`, `Store`, `Call`, `Env`, `Emit`) that every
//!   chain lowers to and that generic analyses understand,
//! - typed per-chain ops (`Dialect`) for constructs that do not generalize.
//!
//! Taint sources/sinks, storage references, and call risks are not stored
//! on ops; they are derived from the op kind (see `bir::interfaces`).

use crate::bir::cfg::FunctionId;
use crate::sir::{Attr, BinOp, Lit, Loc, OverflowSemantics, Type, UnOp};
use std::fmt::{self, Display};

pub use crate::semantics::{EnvVar, EvmBuiltin, ExternalKind};

// ═══════════════════════════════════════════════════════════════════
// ID types
// ═══════════════════════════════════════════════════════════════════

/// A unique identifier for an Op within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpId(pub usize);

/// A reference to an Op result (used as operand).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpRef(pub OpId);

/// An SSA name: `{original_name}_{version}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SsaName {
    pub base: String,
    pub version: u32,
}

/// A parameter index in a function signature.
pub type ParamIndex = usize;

/// A return value index.
pub type ReturnIndex = usize;

impl Display for OpId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl Display for OpRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SsaName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%v{}", self.version)
    }
}

impl SsaName {
    pub fn new(base: &str, version: u32) -> Self {
        SsaName { base: base.to_string(), version }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Op
// ═══════════════════════════════════════════════════════════════════

/// An SSA operation (value node) in BIR.
#[derive(Debug, Clone)]
pub struct Op {
    pub id: OpId,
    pub kind: OpKind,
    pub result: Option<(SsaName, Type)>,
    pub attrs: Vec<Attr>,
    pub span: Option<Loc>,
}

impl Op {
    pub fn new(id: OpId, kind: OpKind) -> Self {
        Op { id, kind, result: None, attrs: vec![], span: None }
    }

    pub fn with_result(mut self, name: SsaName, ty: Type) -> Self {
        self.result = Some((name, ty));
        self
    }

    pub fn with_span(mut self, span: Loc) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_attrs(mut self, attrs: Vec<Attr>) -> Self {
        self.attrs = attrs;
        self
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Results are printed as op ids, the same namespace operands use.
        if let Some((_, ty)) = &self.result {
            write!(f, "{}: {ty} = ", self.id)?;
        }
        write!(f, "{}", self.kind)
    }
}

// ═══════════════════════════════════════════════════════════════════
// OpKind
// ═══════════════════════════════════════════════════════════════════

/// The kind of SSA operation.
#[derive(Debug, Clone)]
pub enum OpKind {
    // ── Core ops ──────────────────────────────────────────────
    /// A constant literal value.
    Const(Lit),
    /// Binary operation.
    BinOp {
        op: BinOp,
        lhs: OpRef,
        rhs: OpRef,
        overflow: OverflowSemantics,
    },
    /// Unary operation.
    UnOp { op: UnOp, operand: OpRef },
    /// Assertion.
    Assert { cond: OpRef },
    /// Return from function.
    Return(Vec<OpRef>),
    /// Function parameter.
    Param { index: ParamIndex },
    /// Expression statement (side effects only).
    ExprStmt { expr: OpRef },

    // ── Shared feature ops (chain-agnostic) ──────────────────
    /// Read persistent state.
    Load(LoadOp),
    /// Write (or delete) persistent state.
    Store(StoreOp),
    /// Internal or external call.
    Call(CallOp),
    /// Read an execution-environment value (caller, timestamp, ...).
    Env(EnvVar),
    /// Emit an event / log entry.
    Emit(EmitOp),

    // ── Per-chain typed ops ──────────────────────────────────
    /// A chain-specific operation that does not generalize.
    Dialect(DialectOp),

    // ── Unresolved names ─────────────────────────────────────
    /// A name not bound to a local definition: a contract, library, or
    /// type name, a specification variable, or an undeclared identifier.
    Symbol { name: String },

    /// A construct not yet lowered to a typed op. Operands are kept so
    /// that def-use and taint still flow through it.
    Opaque { description: String, operands: Vec<OpRef> },
}

impl Display for OpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpKind::Const(lit) => write!(f, "const {lit}"),
            OpKind::BinOp { op, lhs, rhs, overflow } => {
                write!(f, "binop {op} {lhs}, {rhs} [{overflow:?}]")
            }
            OpKind::UnOp { op, operand } => write!(f, "unop {op} {operand}"),
            OpKind::Assert { cond } => write!(f, "assert {cond}"),
            OpKind::Return(vals) => write!(f, "return {}", join_refs(vals)),
            OpKind::Param { index } => write!(f, "param {index}"),
            OpKind::ExprStmt { expr } => write!(f, "expr_stmt {expr}"),
            OpKind::Load(load) => write!(f, "{load}"),
            OpKind::Store(store) => write!(f, "{store}"),
            OpKind::Call(call) => write!(f, "{call}"),
            OpKind::Env(var) => write!(f, "env {var}"),
            OpKind::Emit(emit) => write!(f, "{emit}"),
            OpKind::Dialect(op) => write!(f, "{op}"),
            OpKind::Symbol { name } => write!(f, "symbol \"{name}\""),
            OpKind::Opaque { description, operands } => {
                write!(f, "opaque({description})")?;
                if !operands.is_empty() {
                    write!(f, " [{}]", join_refs(operands))?;
                }
                Ok(())
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Shared feature ops
// ═══════════════════════════════════════════════════════════════════

/// A read of persistent state: `load resource[keys...]`.
#[derive(Debug, Clone)]
pub struct LoadOp {
    pub resource: Resource,
    /// Index operands (mapping keys, array indices, owner address).
    pub keys: Vec<OpRef>,
}

/// A write of persistent state: `store resource[keys...] = value`.
#[derive(Debug, Clone)]
pub struct StoreOp {
    pub resource: Resource,
    pub keys: Vec<OpRef>,
    /// The stored value; `None` deletes the entry (e.g. Move `move_from`).
    pub value: Option<OpRef>,
}

/// A call to an internal function or an external contract/program.
#[derive(Debug, Clone)]
pub struct CallOp {
    pub target: CallTarget,
    pub args: Vec<OpRef>,
    /// Native value transferred with the call (ETH, lamports).
    pub value: Option<OpRef>,
}

/// An event emission.
#[derive(Debug, Clone)]
pub struct EmitOp {
    pub event: String,
    pub args: Vec<OpRef>,
}

/// A persistent-state location family touched by `Load` / `Store`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resource {
    /// A Solana/Anchor account, identified by the account operand.
    AnchorAccount(OpRef),
    /// A Move global resource of the given type, keyed by owner address.
    MoveGlobal(Type),
    /// A contract state variable, as a dotted path for struct fields
    /// (e.g. `balances`, `config.owner`).
    StateVar(String),
}

/// The callee of a `CallOp`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallTarget {
    /// A call leaving the current contract/program.
    External(ExternalCallee),
    /// A statically resolved function of the same contract.
    Internal(FunctionId),
}

/// The destination of an external call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalCallee {
    pub kind: ExternalKind,
    /// The called address/program, when it is an explicit operand.
    pub address: Option<OpRef>,
}

impl Display for LoadOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "load {}", self.resource)?;
        write_keys(f, &self.keys)
    }
}

impl Display for StoreOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(_) => write!(f, "store {}", self.resource)?,
            None => write!(f, "delete {}", self.resource)?,
        }
        write_keys(f, &self.keys)?;
        if let Some(value) = self.value {
            write!(f, " = {value}")?;
        }
        Ok(())
    }
}

impl Display for CallOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "call {}({})", self.target, join_refs(&self.args))?;
        if let Some(value) = self.value {
            write!(f, " value {value}")?;
        }
        Ok(())
    }
}

impl Display for EmitOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "emit {}({})", self.event, join_refs(&self.args))
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resource::AnchorAccount(account) => write!(f, "anchor.account<{account}>"),
            Resource::MoveGlobal(ty) => write!(f, "move.global<{ty}>"),
            Resource::StateVar(name) => write!(f, "@{name}"),
        }
    }
}

impl Display for CallTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallTarget::External(callee) => {
                write!(f, "external.{:?}", callee.kind)?;
                if let Some(address) = callee.address {
                    write!(f, " {address}")?;
                }
                Ok(())
            }
            CallTarget::Internal(func) => write!(f, "{func}"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Per-chain typed ops
// ═══════════════════════════════════════════════════════════════════

/// A chain-specific op.
#[derive(Debug, Clone)]
pub enum DialectOp {
    Anchor(AnchorOp),
    Evm(EvmOp),
    Move(MoveOp),
}

/// EVM-specific ops.
#[derive(Debug, Clone)]
pub enum EvmOp {
    /// A pure EVM builtin (hashing, ABI coding, modular arithmetic, ...).
    Builtin(EvmBuiltinOp),
    /// Inline assembly; opaque to analysis, havoc for verification.
    InlineAsm(String),
    /// `selfdestruct(recipient)`.
    Selfdestruct(OpRef),
}

/// An application of an EVM builtin function.
#[derive(Debug, Clone)]
pub struct EvmBuiltinOp {
    pub builtin: EvmBuiltin,
    pub args: Vec<OpRef>,
}

/// Move-specific ops.
#[derive(Debug, Clone)]
pub enum MoveOp {
    /// `borrow_global_mut<T>(addr)`: a mutable handle that may be written.
    BorrowGlobalMut(MoveGlobalOp),
    /// `exists<T>(addr)`: reads presence of a global resource.
    Exists(MoveGlobalOp),
    /// `move_from<T>(addr)`: removes and returns a global resource.
    MoveFrom(MoveGlobalOp),
    /// `signer::address_of(signer)`.
    SignerAddress(OpRef),
    /// `*reference = value`.
    WriteRef(MoveWriteRefOp),
}

/// A Move op on the global resource of type `ty` stored under `addr`.
#[derive(Debug, Clone)]
pub struct MoveGlobalOp {
    pub addr: OpRef,
    pub ty: Type,
}

/// A Move write through a mutable reference.
#[derive(Debug, Clone)]
pub struct MoveWriteRefOp {
    pub reference: OpRef,
    pub value: OpRef,
}

/// Anchor (Solana)-specific ops.
#[derive(Debug, Clone)]
pub enum AnchorOp {
    /// `ctx.accounts.x.load_mut()`: a mutable account handle.
    AccountLoadMut(OpRef),
    /// `find_program_address(seeds, program_id)`.
    FindProgramAddress(AnchorPdaOp),
    /// `account.key()` of a signer account.
    SignerKey(OpRef),
}

/// A program-derived-address computation.
#[derive(Debug, Clone)]
pub struct AnchorPdaOp {
    pub program_id: OpRef,
    pub seeds: Vec<OpRef>,
}

impl Display for DialectOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialectOp::Anchor(op) => write!(f, "{op}"),
            DialectOp::Evm(op) => write!(f, "{op}"),
            DialectOp::Move(op) => write!(f, "{op}"),
        }
    }
}

impl Display for EvmOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmOp::Builtin(op) => write!(f, "evm.{:?}({})", op.builtin, join_refs(&op.args)),
            EvmOp::InlineAsm(_) => write!(f, "evm.inline_asm"),
            EvmOp::Selfdestruct(recipient) => write!(f, "evm.selfdestruct({recipient})"),
        }
    }
}

impl Display for MoveOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveOp::BorrowGlobalMut(op) => {
                write!(f, "move.borrow_global_mut<{}>({})", op.ty, op.addr)
            }
            MoveOp::Exists(op) => write!(f, "move.exists<{}>({})", op.ty, op.addr),
            MoveOp::MoveFrom(op) => write!(f, "move.move_from<{}>({})", op.ty, op.addr),
            MoveOp::SignerAddress(signer) => write!(f, "move.signer_address({signer})"),
            MoveOp::WriteRef(op) => write!(f, "move.write_ref({}, {})", op.reference, op.value),
        }
    }
}

impl Display for AnchorOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorOp::AccountLoadMut(account) => write!(f, "anchor.account_load_mut({account})"),
            AnchorOp::FindProgramAddress(op) => write!(
                f,
                "anchor.find_program_address([{}], {})",
                join_refs(&op.seeds),
                op.program_id
            ),
            AnchorOp::SignerKey(account) => write!(f, "anchor.signer_key({account})"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Display helpers
// ═══════════════════════════════════════════════════════════════════

fn join_refs(refs: &[OpRef]) -> String {
    refs.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", ")
}

fn write_keys(f: &mut fmt::Formatter<'_>, keys: &[OpRef]) -> fmt::Result {
    for key in keys {
        write!(f, "[{key}]")?;
    }
    Ok(())
}
