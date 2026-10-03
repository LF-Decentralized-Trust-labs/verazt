//! Chain-semantic vocabulary shared by CIR and BIR.
//!
//! These enums name the execution-environment values, external call kinds,
//! and builtin functions that SIR dialect constructs are mapped to during
//! SIR → CIR lowering, and that BIR ops carry after CFG construction.

use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// An execution-environment value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvVar {
    BlockBasefee,
    BlockChainid,
    BlockCoinbase,
    BlockDifficulty,
    BlockGaslimit,
    BlockNumber,
    /// Raw call data (`msg.data`).
    CallData,
    /// Native value sent with the transaction (`msg.value`).
    CallValue,
    /// The immediate caller (`msg.sender`).
    Caller,
    GasLeft,
    /// The transaction originator (`tx.origin`).
    Origin,
    /// The current contract's address (`this`).
    SelfAddress,
    /// The current contract's balance.
    SelfBalance,
    /// The function selector (`msg.sig`).
    Selector,
    Timestamp,
}

/// How an external call is made; determines its risk profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalKind {
    /// EVM low-level `address.call(...)`.
    Call,
    /// Solana cross-program invocation.
    Cpi,
    /// EVM `address.delegatecall(...)`.
    DelegateCall,
    /// A typed call on a contract interface, e.g. `token.transfer(...)`.
    HighLevel,
    /// EVM `address.send(amount)` (2300 gas stipend).
    Send,
    /// EVM `address.staticcall(...)`.
    StaticCall,
    /// Solana system-program lamport transfer.
    SystemTransfer,
    /// Solana token-program transfer.
    TokenTransfer,
    /// EVM `address.transfer(amount)` (2300 gas stipend).
    Transfer,
}

/// EVM builtin functions. Type parameters (conversion target, decoded
/// types) are carried by the result type of the expression or op.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvmBuiltin {
    AbiDecode,
    AbiEncode,
    AbiEncodeCall,
    AbiEncodePacked,
    AbiEncodeWithSelector,
    AbiEncodeWithSignature,
    Addmod,
    Blockhash,
    Concat,
    Convert,
    Ecrecover,
    Empty,
    Keccak256,
    Len,
    Mulmod,
    Ripemd160,
    Sha256,
    Slice,
}

// ═══════════════════════════════════════════════════════════════════
// Display
// ═══════════════════════════════════════════════════════════════════

impl Display for EnvVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
