//! Derived semantics of BIR ops.
//!
//! Generic analyses query these methods instead of matching on concrete
//! op kinds: operands, persistent-state accesses, taint sources and sinks,
//! and call risk. Every chain's dialect ops answer them, so analyses written
//! against this module work across chains.

use crate::bir::ops::{
    AnchorOp, CallOp, CallTarget, DialectOp, EnvVar, EvmBuiltin, EvmOp, ExternalKind, MoveOp,
    OpKind, OpRef, Resource,
};
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Storage references and accesses
// ═══════════════════════════════════════════════════════════════════

/// Identifies a storage location family (e.g. `@balances[*]`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageRef {
    /// Base resource name (e.g. `@balances`, `move.global<Coin>`).
    pub base: String,
    /// Number of index components; each is abstracted as a wildcard.
    pub index_count: usize,
}

impl Display for StorageRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base)?;
        for _ in 0..self.index_count {
            write!(f, "[*]")?;
        }
        Ok(())
    }
}

/// A unique identifier for an alias group.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct AliasGroupId(pub String);

impl Display for AliasGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A read or write of persistent state performed by an op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageAccess {
    pub resource: Resource,
    pub keys: Vec<OpRef>,
    pub is_write: bool,
}

impl StorageAccess {
    fn read(resource: Resource, keys: Vec<OpRef>) -> Self {
        StorageAccess { resource, keys, is_write: false }
    }

    fn write(resource: Resource, keys: Vec<OpRef>) -> Self {
        StorageAccess { resource, keys, is_write: true }
    }

    /// The accessed location family, with keys abstracted to wildcards.
    pub fn storage_ref(&self) -> StorageRef {
        StorageRef { base: self.resource.to_string(), index_count: self.keys.len() }
    }

    /// The alias group of the access: all accesses to the same location
    /// family may alias.
    pub fn alias_group_id(&self) -> AliasGroupId {
        AliasGroupId(self.storage_ref().to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════
// Taint Label
// ═══════════════════════════════════════════════════════════════════

/// A taint label describing the origin of a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaintLabel {
    /// The value is known clean (untainted).
    Clean = 0,
    /// The value was loaded from storage.
    StorageLoaded = 1,
    /// The value came from a signer argument.
    SignerArg = 2,
    /// The value came from account data.
    AccountData = 3,
    /// The value came from an external call return.
    ExternalReturn = 4,
    /// The value came from block context (timestamp, block number).
    BlockContext = 5,
    /// The value is user-controlled (msg.sender, msg.value, etc.).
    UserControlled = 6,
}

impl Display for TaintLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaintLabel::Clean => "clean",
            TaintLabel::StorageLoaded => "storage_loaded",
            TaintLabel::SignerArg => "signer_arg",
            TaintLabel::AccountData => "account_data",
            TaintLabel::ExternalReturn => "external_return",
            TaintLabel::BlockContext => "block_context",
            TaintLabel::UserControlled => "user_controlled",
        };
        write!(f, "{s}")
    }
}

impl TaintLabel {
    /// Return the maximum (most tainted) of two labels.
    pub fn max_label(a: TaintLabel, b: TaintLabel) -> TaintLabel {
        if a >= b { a } else { b }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Sink Category
// ═══════════════════════════════════════════════════════════════════

/// Category of a taint sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SinkCategory {
    /// Event log emission.
    EventLog,
    /// Storage write.
    StorageWrite,
    /// External call argument.
    ExternalCallArg,
    /// Value transfer (ETH send, token transfer).
    ValueTransfer,
    /// Assert condition.
    AssertCondition,
}

// ═══════════════════════════════════════════════════════════════════
// Derived semantics on OpKind
// ═══════════════════════════════════════════════════════════════════

impl OpKind {
    /// All SSA operands read by this op.
    pub fn operands(&self) -> Vec<OpRef> {
        match self {
            OpKind::BinOp { lhs, rhs, .. } => vec![*lhs, *rhs],
            OpKind::UnOp { operand, .. } => vec![*operand],
            OpKind::Phi(args) => args.iter().map(|(_, r)| *r).collect(),
            OpKind::Assert { cond } => vec![*cond],
            OpKind::Return(vals) => vals.clone(),
            OpKind::ExprStmt { expr } => vec![*expr],
            OpKind::Load(load) => load.keys.clone(),
            OpKind::Store(store) => store.keys.iter().copied().chain(store.value).collect(),
            OpKind::Call(call) => call_operands(call),
            OpKind::Emit(emit) => emit.args.clone(),
            OpKind::Dialect(op) => dialect_operands(op),
            OpKind::Opaque { operands, .. } => operands.clone(),
            OpKind::Const(_)
            | OpKind::Param { .. }
            | OpKind::Env(_)
            | OpKind::PseudoValue { .. } => vec![],
        }
    }

    /// The persistent-state access performed by this op, if any.
    pub fn storage_access(&self) -> Option<StorageAccess> {
        match self {
            OpKind::Load(load) => {
                Some(StorageAccess::read(load.resource.clone(), load.keys.clone()))
            }
            OpKind::Store(store) => {
                Some(StorageAccess::write(store.resource.clone(), store.keys.clone()))
            }
            OpKind::Dialect(DialectOp::Move(MoveOp::BorrowGlobalMut(op)))
            | OpKind::Dialect(DialectOp::Move(MoveOp::MoveFrom(op))) => Some(
                StorageAccess::write(Resource::MoveGlobal(op.ty.clone()), vec![op.addr]),
            ),
            OpKind::Dialect(DialectOp::Move(MoveOp::Exists(op))) => Some(StorageAccess::read(
                Resource::MoveGlobal(op.ty.clone()),
                vec![op.addr],
            )),
            OpKind::Dialect(DialectOp::Anchor(AnchorOp::AccountLoadMut(account))) => {
                Some(StorageAccess::write(Resource::AnchorAccount(*account), vec![]))
            }
            // A Move `write_ref` writes through a reference whose resource is
            // only known after alias analysis.
            OpKind::Dialect(DialectOp::Move(MoveOp::SignerAddress(_)))
            | OpKind::Dialect(DialectOp::Move(MoveOp::WriteRef(_)))
            | OpKind::Dialect(DialectOp::Anchor(AnchorOp::FindProgramAddress(_)))
            | OpKind::Dialect(DialectOp::Anchor(AnchorOp::SignerKey(_)))
            | OpKind::Dialect(DialectOp::Evm(_))
            | OpKind::Const(_)
            | OpKind::BinOp { .. }
            | OpKind::UnOp { .. }
            | OpKind::Phi(_)
            | OpKind::Assert { .. }
            | OpKind::Return(_)
            | OpKind::Param { .. }
            | OpKind::ExprStmt { .. }
            | OpKind::Call(_)
            | OpKind::Env(_)
            | OpKind::Emit(_)
            | OpKind::PseudoValue { .. }
            | OpKind::Opaque { .. } => None,
        }
    }

    /// The taint label this op introduces, if it is a taint source.
    pub fn taint_source(&self) -> Option<TaintLabel> {
        match self {
            OpKind::Env(var) => env_taint(*var),
            OpKind::Load(load) => Some(match load.resource {
                Resource::AnchorAccount(_) => TaintLabel::AccountData,
                Resource::MoveGlobal(_) | Resource::StateVar(_) => TaintLabel::StorageLoaded,
            }),
            OpKind::Call(call) if call.is_external() => Some(TaintLabel::ExternalReturn),
            OpKind::Dialect(op) => dialect_taint(op),
            OpKind::Call(_)
            | OpKind::Const(_)
            | OpKind::BinOp { .. }
            | OpKind::UnOp { .. }
            | OpKind::Phi(_)
            | OpKind::Assert { .. }
            | OpKind::Return(_)
            | OpKind::Param { .. }
            | OpKind::ExprStmt { .. }
            | OpKind::Store(_)
            | OpKind::Emit(_)
            | OpKind::PseudoValue { .. }
            | OpKind::Opaque { .. } => None,
        }
    }

    /// The sink category of this op, if its operands reach a sensitive use.
    pub fn sink_category(&self) -> Option<SinkCategory> {
        match self {
            OpKind::Store(_) | OpKind::Dialect(DialectOp::Move(MoveOp::WriteRef(_))) => {
                Some(SinkCategory::StorageWrite)
            }
            OpKind::Call(call) if call.transfers_value() => Some(SinkCategory::ValueTransfer),
            OpKind::Call(call) if call.is_external() => Some(SinkCategory::ExternalCallArg),
            OpKind::Dialect(DialectOp::Evm(EvmOp::Selfdestruct(_))) => {
                Some(SinkCategory::ValueTransfer)
            }
            OpKind::Emit(_) => Some(SinkCategory::EventLog),
            OpKind::Assert { .. } => Some(SinkCategory::AssertCondition),
            OpKind::Call(_)
            | OpKind::Dialect(DialectOp::Anchor(_))
            | OpKind::Dialect(DialectOp::Evm(EvmOp::Builtin(_)))
            | OpKind::Dialect(DialectOp::Evm(EvmOp::InlineAsm(_)))
            | OpKind::Dialect(DialectOp::Move(MoveOp::BorrowGlobalMut(_)))
            | OpKind::Dialect(DialectOp::Move(MoveOp::Exists(_)))
            | OpKind::Dialect(DialectOp::Move(MoveOp::MoveFrom(_)))
            | OpKind::Dialect(DialectOp::Move(MoveOp::SignerAddress(_)))
            | OpKind::Const(_)
            | OpKind::BinOp { .. }
            | OpKind::UnOp { .. }
            | OpKind::Phi(_)
            | OpKind::Return(_)
            | OpKind::Param { .. }
            | OpKind::ExprStmt { .. }
            | OpKind::Load(_)
            | OpKind::Env(_)
            | OpKind::PseudoValue { .. }
            | OpKind::Opaque { .. } => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Derived semantics on CallOp
// ═══════════════════════════════════════════════════════════════════

impl CallOp {
    /// Returns `true` if the call leaves the current contract/program.
    pub fn is_external(&self) -> bool {
        matches!(self.target, CallTarget::External(_))
    }

    /// Returns `true` if the callee can run arbitrary code that may call
    /// back into the current contract.
    pub fn may_reenter(&self) -> bool {
        match &self.target {
            CallTarget::External(callee) => match callee.kind {
                ExternalKind::Call
                | ExternalKind::Cpi
                | ExternalKind::DelegateCall
                | ExternalKind::HighLevel => true,
                ExternalKind::Send
                | ExternalKind::StaticCall
                | ExternalKind::SystemTransfer
                | ExternalKind::TokenTransfer
                | ExternalKind::Transfer => false,
            },
            CallTarget::Internal(_) => false,
        }
    }

    /// Returns `true` if the call moves native value or tokens.
    pub fn transfers_value(&self) -> bool {
        if self.value.is_some() {
            return true;
        }
        match &self.target {
            CallTarget::External(callee) => match callee.kind {
                ExternalKind::Send
                | ExternalKind::SystemTransfer
                | ExternalKind::TokenTransfer
                | ExternalKind::Transfer => true,
                ExternalKind::Call
                | ExternalKind::Cpi
                | ExternalKind::DelegateCall
                | ExternalKind::HighLevel
                | ExternalKind::StaticCall => false,
            },
            CallTarget::Internal(_) => false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn call_operands(call: &CallOp) -> Vec<OpRef> {
    let address = match &call.target {
        CallTarget::External(callee) => callee.address,
        CallTarget::Internal(_) => None,
    };
    address.into_iter().chain(call.args.iter().copied()).chain(call.value).collect()
}

fn dialect_operands(op: &DialectOp) -> Vec<OpRef> {
    match op {
        DialectOp::Anchor(AnchorOp::AccountLoadMut(account))
        | DialectOp::Anchor(AnchorOp::SignerKey(account)) => vec![*account],
        DialectOp::Anchor(AnchorOp::FindProgramAddress(pda)) => {
            pda.seeds.iter().copied().chain([pda.program_id]).collect()
        }
        DialectOp::Evm(EvmOp::Builtin(builtin)) => builtin.args.clone(),
        DialectOp::Evm(EvmOp::InlineAsm(_)) => vec![],
        DialectOp::Evm(EvmOp::Selfdestruct(recipient)) => vec![*recipient],
        DialectOp::Move(MoveOp::BorrowGlobalMut(op))
        | DialectOp::Move(MoveOp::Exists(op))
        | DialectOp::Move(MoveOp::MoveFrom(op)) => vec![op.addr],
        DialectOp::Move(MoveOp::SignerAddress(signer)) => vec![*signer],
        DialectOp::Move(MoveOp::WriteRef(op)) => vec![op.reference, op.value],
    }
}

fn env_taint(var: EnvVar) -> Option<TaintLabel> {
    match var {
        EnvVar::CallData
        | EnvVar::CallValue
        | EnvVar::Caller
        | EnvVar::Origin
        | EnvVar::Selector => Some(TaintLabel::UserControlled),
        EnvVar::BlockBasefee
        | EnvVar::BlockCoinbase
        | EnvVar::BlockDifficulty
        | EnvVar::BlockGaslimit
        | EnvVar::BlockNumber
        | EnvVar::GasLeft
        | EnvVar::Timestamp => Some(TaintLabel::BlockContext),
        EnvVar::BlockChainid | EnvVar::SelfAddress | EnvVar::SelfBalance => None,
    }
}

fn dialect_taint(op: &DialectOp) -> Option<TaintLabel> {
    match op {
        DialectOp::Anchor(AnchorOp::AccountLoadMut(_)) => Some(TaintLabel::AccountData),
        DialectOp::Anchor(AnchorOp::SignerKey(_))
        | DialectOp::Move(MoveOp::SignerAddress(_)) => Some(TaintLabel::SignerArg),
        DialectOp::Evm(EvmOp::Builtin(op)) if op.builtin == EvmBuiltin::Blockhash => {
            Some(TaintLabel::BlockContext)
        }
        DialectOp::Move(MoveOp::BorrowGlobalMut(_))
        | DialectOp::Move(MoveOp::Exists(_))
        | DialectOp::Move(MoveOp::MoveFrom(_)) => Some(TaintLabel::StorageLoaded),
        DialectOp::Anchor(AnchorOp::FindProgramAddress(_))
        | DialectOp::Evm(_)
        | DialectOp::Move(MoveOp::WriteRef(_)) => None,
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bir::ops::{ExternalCallee, OpId};

    fn external(kind: ExternalKind, value: Option<OpRef>) -> CallOp {
        let callee = ExternalCallee { kind, address: Some(OpRef(OpId(0))) };
        CallOp { target: CallTarget::External(callee), args: vec![], value }
    }

    #[test]
    fn test_call_risk_by_external_kind() {
        // Gas-stipend transfers move value but cannot re-enter.
        let transfer = external(ExternalKind::Transfer, Some(OpRef(OpId(1))));
        assert!(!transfer.may_reenter());
        assert!(transfer.transfers_value());

        // A plain low-level call without value can re-enter but moves nothing.
        let call = external(ExternalKind::Call, None);
        assert!(call.may_reenter());
        assert!(!call.transfers_value());
        assert_eq!(OpKind::Call(call).sink_category(), Some(SinkCategory::ExternalCallArg));

        // Attaching value makes a low-level call a value-transfer sink.
        let paid = external(ExternalKind::Call, Some(OpRef(OpId(1))));
        assert_eq!(OpKind::Call(paid).sink_category(), Some(SinkCategory::ValueTransfer));
    }
}
