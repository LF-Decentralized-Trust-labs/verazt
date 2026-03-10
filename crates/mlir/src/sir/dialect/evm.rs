//! EVM dialect for SIR.
//!
//! Covers Solidity and Vyper constructs that are specific to the EVM:
//! address types, msg/block/tx globals, storage ops, events, modifiers,
//! and inline assembly.

use crate::sir::exprs::Expr;
use crate::sir::loc::Span;
use crate::sir::stmts::Stmt;
use crate::sir::types::Type;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// EVM dialect types
// ═══════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════
// EVM dialect expressions
// ═══════════════════════════════════════════════════════════════════

/// EVM-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmExpr {
    /// `evm.msg_sender()` — `msg.sender`
    MsgSender,
    /// `evm.msg_value()` — `msg.value`
    MsgValue,
    /// `evm.timestamp()` — `block.timestamp`
    Timestamp,
    /// `evm.block_number()` — `block.number`
    BlockNumber,
    /// `evm.tx_origin()` — `tx.origin`
    TxOrigin,
    /// `evm.inline_asm` — opaque inline assembly with conservative
    /// attributes (`#sir.call_risk = {reentrancy: true}`, alias = TOP).
    InlineAsm { asm_text: String },

    // ── Vyper-specific expressions ─────────────────────────────
    /// `convert(x, T)` — Vyper's explicit type cast builtin.
    Convert { expr: Box<Expr>, to: Type },
    /// `slice(x, start, len)` — byte slice.
    Slice {
        expr: Box<Expr>,
        start: Box<Expr>,
        length: Box<Expr>,
    },
    /// `len(x)` — length of DynArray or Bytes.
    Len(Box<Expr>),
    /// `raw_call(target, data, value?, gas?)` — low-level call.
    RawCall {
        target: Box<Expr>,
        data: Box<Expr>,
        value: Option<Box<Expr>>,
        gas: Option<Box<Expr>>,
    },
    /// `send(target, value)` — Vyper's send() builtin.
    Send { target: Box<Expr>, value: Box<Expr> },
    /// `self.balance` — contract's own balance.
    SelfBalance,
    /// `empty(T)` — zero value of type T.
    Empty(Type),
    /// `concat(a, b, ...)` — byte/string concatenation.
    Concat(Vec<Expr>),

    // ── Explicit low-level call dialect nodes ──────────────────
    /// `evm.delegatecall(target, data)` — low-level delegatecall.
    Delegatecall {
        target: Box<Expr>,
        data: Box<Expr>,
        span: Option<Span>,
    },
    /// `evm.low_level_call(target, data, value?, gas?)` — `.call{value:…}(…)`.
    LowLevelCall {
        target: Box<Expr>,
        data: Box<Expr>,
        value: Option<Box<Expr>>,
        gas: Option<Box<Expr>>,
        span: Option<Span>,
    },
}

impl Display for EvmExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmExpr::MsgSender => write!(f, "evm.msg_sender()"),
            EvmExpr::MsgValue => write!(f, "evm.msg_value()"),
            EvmExpr::Timestamp => write!(f, "evm.timestamp()"),
            EvmExpr::BlockNumber => write!(f, "evm.block_number()"),
            EvmExpr::TxOrigin => write!(f, "evm.tx_origin()"),
            EvmExpr::InlineAsm { asm_text } => write!(f, "evm.inline_asm({asm_text})"),
            EvmExpr::Convert { expr, to } => write!(f, "evm.convert({expr}, {to})"),
            EvmExpr::Slice { expr, start, length } => {
                write!(f, "evm.slice({expr}, {start}, {length})")
            }
            EvmExpr::Len(e) => write!(f, "evm.len({e})"),
            EvmExpr::RawCall { target, data, value, gas } => {
                write!(f, "evm.raw_call({target}, {data}")?;
                if let Some(v) = value {
                    write!(f, ", value={v}")?;
                }
                if let Some(g) = gas {
                    write!(f, ", gas={g}")?;
                }
                write!(f, ")")
            }
            EvmExpr::Send { target, value } => {
                write!(f, "evm.send({target}, {value})")
            }
            EvmExpr::SelfBalance => write!(f, "evm.self_balance()"),
            EvmExpr::Empty(ty) => write!(f, "evm.empty({ty})"),
            EvmExpr::Concat(exprs) => {
                let es: Vec<_> = exprs.iter().map(|e| e.to_string()).collect();
                write!(f, "evm.concat({})", es.join(", "))
            }
            EvmExpr::Delegatecall { target, data, .. } => {
                write!(f, "evm.delegatecall({target}, {data})")
            }
            EvmExpr::LowLevelCall { target, data, value, gas, .. } => {
                write!(f, "evm.low_level_call({target}, {data}")?;
                if let Some(v) = value {
                    write!(f, ", value={v}")?;
                }
                if let Some(g) = gas {
                    write!(f, ", gas={g}")?;
                }
                write!(f, ")")
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// EVM dialect statements
// ═══════════════════════════════════════════════════════════════════

/// EVM-specific statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmStmt {
    /// `evm.emit_event(EventName, arg1, arg2, ...)`
    EmitEvent {
        event: String,
        args: Vec<Expr>,
        span: Option<Span>,
    },
    /// `evm.try_catch` — Solidity try/catch statement.
    TryCatch {
        guarded_expr: Expr,
        returns: Vec<(String, Type)>,
        body: Vec<Stmt>,
        catch_clauses: Vec<EvmCatchClause>,
        span: Option<Span>,
    },
}

/// A catch clause in a Solidity try/catch statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmCatchClause {
    pub error: Option<String>,
    pub params: Vec<(String, Type)>,
    pub body: Vec<Stmt>,
    pub span: Option<Span>,
}

impl Display for EvmStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmStmt::EmitEvent { event, args, .. } => {
                let a: Vec<_> = args.iter().map(|e| e.to_string()).collect();
                write!(f, "emit {event}({});", a.join(", "))
            }
            EvmStmt::TryCatch { guarded_expr, body: _, .. } => {
                write!(f, "try {guarded_expr} {{ ... }}")
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// EVM dialect member declarations
// ═══════════════════════════════════════════════════════════════════

/// EVM-specific member declarations inside a contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmMemberDecl {
    /// An event definition.
    EventDef {
        name: String,
        params: Vec<(String, Type)>,
        indexed: Vec<bool>,
        anonymous: bool,
    },
    /// A modifier definition.
    ModifierDef {
        name: String,
        params: Vec<(String, Type)>,
        body: Vec<Stmt>,
    },
    /// An error definition.
    ErrorDef {
        name: String,
        params: Vec<(String, Type)>,
    },
    /// An enum definition.
    EnumDef { name: String, variants: Vec<String> },
    /// A struct definition.
    StructDef {
        name: String,
        fields: Vec<(String, Type)>,
    },
}

impl Display for EvmMemberDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmMemberDecl::EventDef { name, params, .. } => {
                let ps: Vec<_> = params.iter().map(|(n, t)| format!("{t} {n}")).collect();
                write!(f, "event {name}({});", ps.join(", "))
            }
            EvmMemberDecl::ModifierDef { name, .. } => {
                write!(f, "modifier {name}(...) {{ ... }}")
            }
            EvmMemberDecl::ErrorDef { name, params } => {
                let ps: Vec<_> = params.iter().map(|(n, t)| format!("{t} {n}")).collect();
                write!(f, "error {name}({});", ps.join(", "))
            }
            EvmMemberDecl::EnumDef { name, variants } => {
                write!(f, "enum {name} {{ {} }}", variants.join(", "))
            }
            EvmMemberDecl::StructDef { name, fields } => {
                let fs: Vec<_> = fields.iter().map(|(n, t)| format!("{t} {n}")).collect();
                write!(f, "struct {name} {{ {} }}", fs.join("; "))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// EVM dialect-level declarations (future use)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmDecl {
    // Future: interface declarations, library bindings, etc.
    _Placeholder,
}

impl Display for EvmDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmDecl::_Placeholder => write!(f, "// evm decl placeholder"),
        }
    }
}
