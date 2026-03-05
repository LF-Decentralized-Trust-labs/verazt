//! Anchor (Solana) dialect for CIR.
//!
//! Covers Anchor framework constructs: account types, CPIs (Cross-Program
//! Invocations), PDA derivation, events, errors, and account context
//! definitions. The naming conventions follow the architecture document.

use crate::exprs::Expr;
use crate::loc::Span;
use crate::types::Type;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Anchor dialect types
// ═══════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════
// Anchor dialect expressions
// ═══════════════════════════════════════════════════════════════════

/// Anchor-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorExpr {
    /// `anchor.account_load(ctx.accounts.X)` — immutable account data load.
    AccountLoad(Box<Expr>),
    /// `anchor.account_load_mut(ctx.accounts.X)` — mutable account data load.
    AccountLoadMut(Box<Expr>),
    /// `anchor.signer_key(ctx.accounts.X)` — get pubkey of a signer account.
    SignerKey(Box<Expr>),
    /// `anchor.ok(value)` — wrap value in Anchor's Result::Ok.
    Ok(Box<Expr>),
    /// `anchor.cpi(...)` — cross-program invocation.
    Cpi {
        program: Box<Expr>,
        accounts: Vec<Expr>,
        data: Box<Expr>,
    },
    /// `anchor.system_transfer(from, to, lamports)` — system program transfer
    /// CPI.
    SystemTransfer {
        from: Box<Expr>,
        to: Box<Expr>,
        lamports: Box<Expr>,
    },
    /// `anchor.token_transfer(from, to, authority, amount)` — SPL token
    /// transfer CPI.
    TokenTransfer {
        from: Box<Expr>,
        to: Box<Expr>,
        authority: Box<Expr>,
        amount: Box<Expr>,
    },
    /// PDA derivation: `Pubkey::find_program_address(seeds, program_id)`.
    FindProgramAddress {
        seeds: Vec<Expr>,
        program_id: Box<Expr>,
    },
}

impl Display for AnchorExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorExpr::AccountLoad(e) => write!(f, "anchor.account_load({e})"),
            AnchorExpr::AccountLoadMut(e) => write!(f, "anchor.account_load_mut({e})"),
            AnchorExpr::SignerKey(e) => write!(f, "anchor.signer_key({e})"),
            AnchorExpr::Ok(e) => write!(f, "anchor.ok({e})"),
            AnchorExpr::Cpi { program, accounts, data } => {
                let accs: Vec<_> = accounts.iter().map(|a| a.to_string()).collect();
                write!(f, "anchor.cpi({program}, [{}], {data})", accs.join(", "))
            }
            AnchorExpr::SystemTransfer { from, to, lamports } => {
                write!(f, "anchor.system_transfer({from}, {to}, {lamports})")
            }
            AnchorExpr::TokenTransfer { from, to, authority, amount } => {
                write!(f, "anchor.token_transfer({from}, {to}, {authority}, {amount})")
            }
            AnchorExpr::FindProgramAddress { seeds, program_id } => {
                let ss: Vec<_> = seeds.iter().map(|s| s.to_string()).collect();
                write!(f, "anchor.find_program_address([{}], {program_id})", ss.join(", "))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Anchor dialect statements
// ═══════════════════════════════════════════════════════════════════

/// Anchor-specific statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorStmt {
    /// `anchor.emit_event` — event emission via Anchor's `emit!` macro.
    EmitEvent {
        event: String,
        fields: Vec<(String, Expr)>,
        span: Option<Span>,
    },
}

impl Display for AnchorStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorStmt::EmitEvent { event, fields, .. } => {
                let fs: Vec<_> = fields.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                write!(f, "emit!({event} {{ {} }});", fs.join(", "))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Anchor dialect member declarations
// ═══════════════════════════════════════════════════════════════════

/// Anchor-specific member declarations inside a contract/program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorMemberDecl {
    /// `anchor.accounts_context` — `#[derive(Accounts)]` struct.
    AccountsContext {
        name: String,
        accounts: Vec<AnchorAccountField>,
    },
    /// `anchor.account_struct` — `#[account]` data struct (serialized to
    /// account data).
    AccountStruct {
        name: String,
        discriminator_size: u8,
        fields: Vec<(String, Type)>,
    },
    /// `anchor.event_struct` — `#[event]` struct.
    EventStruct {
        name: String,
        fields: Vec<(String, Type)>,
    },
    /// `anchor.error_code` — `#[error_code]` enum.
    ErrorCode {
        name: String,
        variants: Vec<(String, String)>,
    },
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
            AnchorMemberDecl::AccountsContext { name, accounts } => {
                writeln!(f, "#[derive(Accounts)]")?;
                writeln!(f, "struct {name} {{")?;
                for acc in accounts {
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
            AnchorMemberDecl::AccountStruct { name, fields, .. } => {
                writeln!(f, "#[account]")?;
                writeln!(f, "struct {name} {{")?;
                for (n, t) in fields {
                    writeln!(f, "    {n}: {t},")?;
                }
                write!(f, "}}")
            }
            AnchorMemberDecl::EventStruct { name, fields } => {
                writeln!(f, "#[event]")?;
                writeln!(f, "struct {name} {{")?;
                for (n, t) in fields {
                    writeln!(f, "    {n}: {t},")?;
                }
                write!(f, "}}")
            }
            AnchorMemberDecl::ErrorCode { name, variants } => {
                writeln!(f, "#[error_code]")?;
                writeln!(f, "enum {name} {{")?;
                for (v, msg) in variants {
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
