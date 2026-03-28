//! Anchor dialect expressions with enum-of-structs pattern.

use crate::sir::exprs::Expr;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each AnchorExpr variant
// ═══════════════════════════════════════════════════════════════════

/// `anchor.account_load(ctx.accounts.X)` — immutable account data load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorAccountLoad {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `anchor.account_load_mut(ctx.accounts.X)` — mutable account data load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorAccountLoadMut {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `anchor.signer_key(ctx.accounts.X)` — get pubkey of a signer account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorSignerKey {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `anchor.ok(value)` — wrap value in Anchor's Result::Ok.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorOk {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `anchor.cpi(...)` — cross-program invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorCpi {
    pub program: Box<Expr>,
    pub accounts: Vec<Expr>,
    pub data: Box<Expr>,
    pub loc: Loc,
}

/// `anchor.system_transfer(from, to, lamports)` — system program transfer CPI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorSystemTransfer {
    pub from: Box<Expr>,
    pub to: Box<Expr>,
    pub lamports: Box<Expr>,
    pub loc: Loc,
}

/// `anchor.token_transfer(from, to, authority, amount)` — SPL token transfer
/// CPI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorTokenTransfer {
    pub from: Box<Expr>,
    pub to: Box<Expr>,
    pub authority: Box<Expr>,
    pub amount: Box<Expr>,
    pub loc: Loc,
}

/// PDA derivation: `Pubkey::find_program_address(seeds, program_id)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorFindProgramAddress {
    pub seeds: Vec<Expr>,
    pub program_id: Box<Expr>,
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// AnchorExpr enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// Anchor-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorExpr {
    AccountLoad(AnchorAccountLoad),
    AccountLoadMut(AnchorAccountLoadMut),
    SignerKey(AnchorSignerKey),
    Ok(AnchorOk),
    Cpi(AnchorCpi),
    SystemTransfer(AnchorSystemTransfer),
    TokenTransfer(AnchorTokenTransfer),
    FindProgramAddress(AnchorFindProgramAddress),
}

impl Display for AnchorExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorExpr::AccountLoad(e) => write!(f, "anchor.account_load({})", e.expr),
            AnchorExpr::AccountLoadMut(e) => write!(f, "anchor.account_load_mut({})", e.expr),
            AnchorExpr::SignerKey(e) => write!(f, "anchor.signer_key({})", e.expr),
            AnchorExpr::Ok(e) => write!(f, "anchor.ok({})", e.expr),
            AnchorExpr::Cpi(e) => {
                let accs: Vec<_> = e.accounts.iter().map(|a| a.to_string()).collect();
                write!(f, "anchor.cpi({}, [{}], {})", e.program, accs.join(", "), e.data)
            }
            AnchorExpr::SystemTransfer(e) => {
                write!(f, "anchor.system_transfer({}, {}, {})", e.from, e.to, e.lamports)
            }
            AnchorExpr::TokenTransfer(e) => {
                write!(
                    f,
                    "anchor.token_transfer({}, {}, {}, {})",
                    e.from, e.to, e.authority, e.amount
                )
            }
            AnchorExpr::FindProgramAddress(e) => {
                let ss: Vec<_> = e.seeds.iter().map(|s| s.to_string()).collect();
                write!(f, "anchor.find_program_address([{}], {})", ss.join(", "), e.program_id)
            }
        }
    }
}
