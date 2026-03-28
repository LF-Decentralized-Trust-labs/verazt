//! Anchor dialect member declarations with enum-of-structs pattern.

use crate::sir::exprs::Expr;
use crate::sir::types::Type;
use common::loc::Loc;
use std::fmt::{self, Display};

use super::types::AnchorType;

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each AnchorMemberDecl variant
// ═══════════════════════════════════════════════════════════════════

/// `anchor.accounts_context` — `#[derive(Accounts)]` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorAccountsContext {
    pub name: String,
    pub accounts: Vec<AnchorAccountField>,
    pub loc: Loc,
}

/// `anchor.account_struct` — `#[account]` data struct (serialized to account
/// data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorAccountStruct {
    pub name: String,
    pub discriminator_size: u8,
    pub fields: Vec<(String, Type)>,
    pub loc: Loc,
}

/// `anchor.event_struct` — `#[event]` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorEventStruct {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub loc: Loc,
}

/// `anchor.error_code` — `#[error_code]` enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorErrorCode {
    pub name: String,
    pub variants: Vec<(String, String)>,
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// AnchorMemberDecl enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// Anchor-specific member declarations inside a contract/program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorMemberDecl {
    AccountsContext(AnchorAccountsContext),
    AccountStruct(AnchorAccountStruct),
    EventStruct(AnchorEventStruct),
    ErrorCode(AnchorErrorCode),
}

/// A field in an `#[derive(Accounts)]` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorAccountField {
    pub name: String,
    pub ty: AnchorType,
    pub is_mut: bool,
    pub constraint: Option<String>,
    pub payer: Option<String>,
    pub space: Option<Expr>,
    pub check_note: Option<String>,
}

impl Display for AnchorMemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorMemberDecl::AccountsContext(e) => {
                writeln!(f, "#[derive(Accounts)]")?;
                writeln!(f, "struct {} {{", e.name)?;
                for acc in &e.accounts {
                    write!(f, "    {}: {}", acc.name, acc.ty)?;
                    if acc.is_mut {
                        write!(f, " /* mut */")?;
                    }
                    if let Some(c) = &acc.constraint {
                        write!(f, " /* {c} */")?;
                    }
                    writeln!(f, ",")?;
                }
                write!(f, "}}")
            }
            AnchorMemberDecl::AccountStruct(e) => {
                writeln!(f, "#[account]")?;
                writeln!(f, "struct {} {{", e.name)?;
                for (n, t) in &e.fields {
                    writeln!(f, "    {n}: {t},")?;
                }
                write!(f, "}}")
            }
            AnchorMemberDecl::EventStruct(e) => {
                writeln!(f, "#[event]")?;
                writeln!(f, "struct {} {{", e.name)?;
                for (n, t) in &e.fields {
                    writeln!(f, "    {n}: {t},")?;
                }
                write!(f, "}}")
            }
            AnchorMemberDecl::ErrorCode(e) => {
                writeln!(f, "#[error_code]")?;
                writeln!(f, "enum {} {{", e.name)?;
                for (v, msg) in &e.variants {
                    writeln!(f, "    #[msg(\"{msg}\")]")?;
                    writeln!(f, "    {v},")?;
                }
                write!(f, "}}")
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Anchor dialect-level declarations (future use)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorDecl {
    /// Placeholder for future anchor-level declarations.
    _Placeholder,
}

impl Display for AnchorDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorDecl::_Placeholder => write!(f, "// anchor decl placeholder"),
        }
    }
}
