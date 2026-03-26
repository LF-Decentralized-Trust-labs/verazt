//! Anchor dialect types.

use crate::sir::types::Type;
use std::fmt::{self, Display};

/// Anchor-specific types for Solana programs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnchorType {
    /// `!anchor.pubkey` — Solana public key (32 bytes).
    Pubkey,
    /// `!anchor.signer` — must-sign account.
    Signer,
    /// `!anchor.account<T>` — program-owned account holding data of type T.
    Account(Box<Type>),
    /// `!anchor.system_account` — system-program-owned account.
    SystemAccount,
    /// `!anchor.unchecked_account` — no ownership validation.
    UncheckedAccount,
    /// `!anchor.program<T>` — a program account (T = system/token/etc.).
    Program(Box<Type>),
    /// `!anchor.context<T>` — the ctx parameter type for an instruction.
    Context(Box<Type>),
    /// `!anchor.result<T>` — Anchor's Result<T> return type.
    Result(Box<Type>),
}

impl Display for AnchorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorType::Pubkey => write!(f, "!anchor.pubkey"),
            AnchorType::Signer => write!(f, "!anchor.signer"),
            AnchorType::Account(ty) => write!(f, "!anchor.account<{ty}>"),
            AnchorType::SystemAccount => write!(f, "!anchor.system_account"),
            AnchorType::UncheckedAccount => write!(f, "!anchor.unchecked_account"),
            AnchorType::Program(ty) => write!(f, "!anchor.program<{ty}>"),
            AnchorType::Context(ty) => write!(f, "!anchor.context<{ty}>"),
            AnchorType::Result(ty) => write!(f, "!anchor.result<{ty}>"),
        }
    }
}
