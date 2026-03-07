//! Interface traits for ANIR ops.
//!
//! Every dialect construct that reaches ANIR implements at least one of:
//! `StorageOp`, `CallOp`, `TaintSource`, or `TaintSink`.

use crate::air::ops::{CallDialectOp, OpKind, StorageDialectOp, TaintSinkOp, TaintSourceOp};
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Storage Reference
// ═══════════════════════════════════════════════════════════════════

/// Identifies a storage location (e.g., "balances[*]").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageRef {
    /// Base name (e.g., "balances", "Balance<CoinType>").
    pub base: String,
    /// Index components.
    pub indices: Vec<StorageIndex>,
}

/// A single index component of a storage reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageIndex {
    /// Wildcard index (any key).
    Wildcard,
    /// A specific SSA operand used as key.
    Key(String), // SSA name string representation
}

impl Display for StorageRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base)?;
        for idx in &self.indices {
            match idx {
                StorageIndex::Wildcard => write!(f, "[*]")?,
                StorageIndex::Key(k) => write!(f, "[{k}]")?,
            }
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

// ═══════════════════════════════════════════════════════════════════
// Call Target and Risk
// ═══════════════════════════════════════════════════════════════════

/// Target of a call operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallTarget {
    /// A statically resolved internal function.
    Static(String),
    /// A dynamically dispatched call (external, unresolved).
    Dynamic,
}

impl Display for CallTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallTarget::Static(name) => write!(f, "{name}"),
            CallTarget::Dynamic => write!(f, "<dynamic>"),
        }
    }
}

/// Risk assessment for a call operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallRisk {
    /// Whether this call has reentrancy risk.
    pub reentrancy: bool,
    /// Whether this call transfers value.
    pub value_transfer: bool,
}

impl Default for CallRisk {
    fn default() -> Self {
        CallRisk { reentrancy: false, value_transfer: false }
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
// StorageOp trait
// ═══════════════════════════════════════════════════════════════════

/// A storage read or write operation.
pub trait StorageOp {
    fn to_storage_ref(&self) -> StorageRef;
    fn is_write(&self) -> bool;
    fn alias_group_id(&self) -> AliasGroupId;
}

impl StorageOp for StorageDialectOp {
    fn to_storage_ref(&self) -> StorageRef {
        self.storage_ref.clone()
    }
    fn is_write(&self) -> bool {
        self.is_write
    }
    fn alias_group_id(&self) -> AliasGroupId {
        self.alias_group_id.clone()
    }
}

// ═══════════════════════════════════════════════════════════════════
// CallOp trait
// ═══════════════════════════════════════════════════════════════════

/// An internal or external call operation.
pub trait CallOp {
    fn callee(&self) -> CallTarget;
    fn call_risk(&self) -> CallRisk;
}

impl CallOp for CallDialectOp {
    fn callee(&self) -> CallTarget {
        self.callee.clone()
    }
    fn call_risk(&self) -> CallRisk {
        self.call_risk.clone()
    }
}

// ═══════════════════════════════════════════════════════════════════
// TaintSource trait
// ═══════════════════════════════════════════════════════════════════

/// A value that seeds the taint graph.
pub trait TaintSource {
    fn taint_label(&self) -> TaintLabel;
}

impl TaintSource for TaintSourceOp {
    fn taint_label(&self) -> TaintLabel {
        self.label
    }
}

// ═══════════════════════════════════════════════════════════════════
// TaintSink trait
// ═══════════════════════════════════════════════════════════════════

/// A value that is a taint sink (analysed for tainted inputs).
pub trait TaintSink {
    fn sink_category(&self) -> SinkCategory;
}

impl TaintSinkOp {
    pub fn sink_category(&self) -> SinkCategory {
        self.category
    }
}

// ═══════════════════════════════════════════════════════════════════
// Query helpers on OpKind
// ═══════════════════════════════════════════════════════════════════

impl OpKind {
    /// Check if this op implements StorageOp.
    pub fn as_storage_op(&self) -> Option<&StorageDialectOp> {
        if let OpKind::Storage(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Check if this op implements CallOp.
    pub fn as_call_op(&self) -> Option<&CallDialectOp> {
        if let OpKind::Call(c) = self {
            Some(c)
        } else {
            None
        }
    }

    /// Check if this op implements TaintSource.
    pub fn as_taint_source(&self) -> Option<&TaintSourceOp> {
        if let OpKind::TaintSrc(t) = self {
            Some(t)
        } else {
            None
        }
    }

    /// Check if this op implements TaintSink.
    pub fn as_taint_sink(&self) -> Option<&TaintSinkOp> {
        if let OpKind::TaintSnk(t) = self {
            Some(t)
        } else {
            None
        }
    }

    /// Returns true if this op implements at least one ANIR interface.
    pub fn has_interface(&self) -> bool {
        matches!(
            self,
            OpKind::Storage(_) | OpKind::Call(_) | OpKind::TaintSrc(_) | OpKind::TaintSnk(_)
        )
    }
}
