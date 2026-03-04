//! CIR type system.
//!
//! Explicit bit-width integer types, composite types, and dialect extension.

use crate::dialect::DialectType;
use std::fmt::{self, Display};

/// The core CIR type system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // ── Unsigned integers ──────────────────────────
    I1,
    I8,
    I16,
    I32,
    I64,
    I128,
    I256,

    // ── Signed integers ────────────────────────────
    Si8,
    Si16,
    Si32,
    Si64,
    Si128,
    Si256,

    // ── Primitives ─────────────────────────────────
    Bool,
    String,
    Bytes,
    FixedBytes(u8), // bytes1 .. bytes32
    None,
    Index,

    // ── Composite ──────────────────────────────────
    Array(Box<Type>),
    FixedArray(Box<Type>, u64),
    Map(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Option(Box<Type>),
    Function {
        params: Vec<Type>,
        returns: Vec<Type>,
    },

    // ── Named type reference (structs, enums, contracts) ──
    TypeRef(String),

    // ── Dialect-specific types ─────────────────────
    Dialect(DialectType),
}

impl Type {
    /// Map a Solidity-style integer (bitwidth, signed) to the CIR type.
    pub fn from_int(bitwidth: Option<u16>, signed: bool) -> Self {
        match (signed, bitwidth) {
            (false, Option::None | Some(256)) => Type::I256,
            (false, Some(1)) => Type::I1,
            (false, Some(8)) => Type::I8,
            (false, Some(16)) => Type::I16,
            (false, Some(32)) => Type::I32,
            (false, Some(64)) => Type::I64,
            (false, Some(128)) => Type::I128,
            // Default to I256 for other unsigned widths
            (false, Some(_)) => Type::I256,
            (true, Option::None | Some(256)) => Type::Si256,
            (true, Some(8)) => Type::Si8,
            (true, Some(16)) => Type::Si16,
            (true, Some(32)) => Type::Si32,
            (true, Some(64)) => Type::Si64,
            (true, Some(128)) => Type::Si128,
            // Default to Si256 for other signed widths
            (true, Some(_)) => Type::Si256,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::I1
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::I128
                | Type::I256
                | Type::Si8
                | Type::Si16
                | Type::Si32
                | Type::Si64
                | Type::Si128
                | Type::Si256
        )
    }

    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Type::Si8 | Type::Si16 | Type::Si32 | Type::Si64 | Type::Si128 | Type::Si256
        )
    }

    pub fn is_function_type(&self) -> bool {
        matches!(self, Type::Function { .. })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I1 => write!(f, "u1"),
            Type::I8 => write!(f, "u8"),
            Type::I16 => write!(f, "u16"),
            Type::I32 => write!(f, "u32"),
            Type::I64 => write!(f, "u64"),
            Type::I128 => write!(f, "u128"),
            Type::I256 => write!(f, "u256"),
            Type::Si8 => write!(f, "i8"),
            Type::Si16 => write!(f, "i16"),
            Type::Si32 => write!(f, "i32"),
            Type::Si64 => write!(f, "i64"),
            Type::Si128 => write!(f, "i128"),
            Type::Si256 => write!(f, "i256"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Bytes => write!(f, "bytes"),
            Type::FixedBytes(n) => write!(f, "bytes{n}"),
            Type::None => write!(f, "none"),
            Type::Index => write!(f, "index"),
            Type::Array(base) => write!(f, "{base}[]"),
            Type::FixedArray(base, len) => write!(f, "{base}[{len}]"),
            Type::Map(k, v) => write!(f, "mapping({k} => {v})"),
            Type::Tuple(elems) => {
                let s: Vec<_> = elems.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", s.join(", "))
            }
            Type::Option(inner) => write!(f, "option<{inner}>"),
            Type::Function { params, returns } => {
                let ps: Vec<_> = params.iter().map(|t| t.to_string()).collect();
                write!(f, "function({})", ps.join(", "))?;
                if !returns.is_empty() {
                    let rs: Vec<_> = returns.iter().map(|t| t.to_string()).collect();
                    write!(f, " returns ({})", rs.join(", "))?;
                }
                Ok(())
            }
            Type::TypeRef(name) => write!(f, "{name}"),
            Type::Dialect(dt) => write!(f, "{dt}"),
        }
    }
}
