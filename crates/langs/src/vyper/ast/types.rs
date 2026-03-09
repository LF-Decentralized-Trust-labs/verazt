//! Vyper AST type system.

use std::fmt::{self, Display};

/// Vyper type representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // ── Primitives ──────────────────────────────────
    Bool,
    Address,
    Bytes32,

    // ── Integers ────────────────────────────────────
    Int(IntType),
    UInt(UIntType),

    // ── Fixed-point ─────────────────────────────────
    Decimal,

    // ── Bounded composite types (Vyper-specific) ────
    BoundedString(u64),
    BoundedBytes(u64),
    DynArray { elem: Box<Type>, max_len: u64 },

    // ── Fixed-size ──────────────────────────────────
    FixedArray { elem: Box<Type>, len: u64 },

    // ── Named references ────────────────────────────
    HashMap(Box<Type>, Box<Type>),
    Struct(String),
    Enum(String),
    Flag(String),
    Interface(String),

    // ── Visibility wrapper (for public state vars) ──
    Public(Box<Type>),

    // ── Tuple (for multi-returns) ───────────────────
    Tuple(Vec<Type>),
}

/// Signed integer type with bit-width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntType {
    pub bits: u16,
}

/// Unsigned integer type with bit-width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UIntType {
    pub bits: u16,
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Address => write!(f, "address"),
            Type::Bytes32 => write!(f, "bytes32"),
            Type::Int(t) => write!(f, "int{}", t.bits),
            Type::UInt(t) => write!(f, "uint{}", t.bits),
            Type::Decimal => write!(f, "decimal"),
            Type::BoundedString(n) => write!(f, "String[{n}]"),
            Type::BoundedBytes(n) => write!(f, "Bytes[{n}]"),
            Type::DynArray { elem, max_len } => write!(f, "DynArray[{elem}, {max_len}]"),
            Type::FixedArray { elem, len } => write!(f, "{elem}[{len}]"),
            Type::HashMap(k, v) => write!(f, "HashMap[{k}, {v}]"),
            Type::Struct(name) => write!(f, "{name}"),
            Type::Enum(name) => write!(f, "{name}"),
            Type::Flag(name) => write!(f, "{name}"),
            Type::Interface(name) => write!(f, "{name}"),
            Type::Public(inner) => write!(f, "public({inner})"),
            Type::Tuple(elems) => {
                let s: Vec<_> = elems.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", s.join(", "))
            }
        }
    }
}
