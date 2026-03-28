//! EVM dialect types.

use crate::sir::types::Type;
use std::fmt::{self, Display};

/// EVM-specific types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EvmType {
    /// `!evm.address` — a 20-byte Ethereum address.
    Address,
    /// `!evm.address_payable` — a payable Ethereum address.
    AddressPayable,
    /// `!evm.slot` — a storage slot reference.
    Slot,
    /// Vyper's bounded dynamic array — `DynArray[T, N]`.
    DynArray { elem: Box<Type>, max_len: u64 },
    /// Vyper's bounded byte string — `Bytes[N]`.
    BoundedBytes(u64),
    /// Vyper's bounded string — `String[N]`.
    BoundedString(u64),
}

impl Display for EvmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmType::Address => write!(f, "address"),
            EvmType::AddressPayable => write!(f, "address payable"),
            EvmType::Slot => write!(f, "!evm.slot"),
            EvmType::DynArray { elem, max_len } => {
                write!(f, "DynArray[{elem}, {max_len}]")
            }
            EvmType::BoundedBytes(n) => write!(f, "Bytes[{n}]"),
            EvmType::BoundedString(n) => write!(f, "String[{n}]"),
        }
    }
}
