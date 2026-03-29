//! Literal values in SIR.

use crate::sir::types::Type;
use common::loc::Loc;
use num_bigint::BigInt;
use num_traits::One;
use rust_decimal::Decimal;
use std::fmt::{self, Display};

/// A literal value.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Lit {
    Bool(BoolLit),
    Num(NumLit),
    String(StringLit),
    Hex(HexLit),
    Unicode(UnicodeLit),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoolLit {
    pub value: bool,
    pub span: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NumLit {
    pub value: Num,
    pub span: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StringLit {
    pub value: String,
    pub span: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HexLit {
    pub value: String,
    pub span: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UnicodeLit {
    pub value: String,
    pub span: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Num {
    Int(IntNum),
    Fixed(FixedNum),
    Hex(HexNum),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IntNum {
    pub value: BigInt,
    pub typ: Type,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FixedNum {
    pub value: Decimal,
    pub typ: Type,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HexNum {
    pub value: String,
    pub typ: Type,
}

// ─── Implementations ───────────────────────────────────────────────

impl Lit {
    pub fn one(span: Option<Loc>) -> Self {
        Lit::from(NumLit::new(Num::one(), span))
    }

    pub fn typ(&self) -> Type {
        match self {
            Lit::Bool(_) => Type::Bool,
            Lit::Num(n) => n.typ(),
            Lit::String(_) => Type::String,
            Lit::Hex(_) => Type::Bytes,
            Lit::Unicode(_) => Type::String,
        }
    }

    pub fn span(&self) -> Option<&Loc> {
        match self {
            Lit::Bool(b) => b.span.as_ref(),
            Lit::Num(n) => n.span.as_ref(),
            Lit::String(s) => s.span.as_ref(),
            Lit::Hex(h) => h.span.as_ref(),
            Lit::Unicode(u) => u.span.as_ref(),
        }
    }
}

impl From<BoolLit> for Lit {
    fn from(lit: BoolLit) -> Lit {
        Lit::Bool(lit)
    }
}
impl From<NumLit> for Lit {
    fn from(lit: NumLit) -> Lit {
        Lit::Num(lit)
    }
}
impl From<StringLit> for Lit {
    fn from(lit: StringLit) -> Lit {
        Lit::String(lit)
    }
}
impl From<HexLit> for Lit {
    fn from(lit: HexLit) -> Lit {
        Lit::Hex(lit)
    }
}
impl From<UnicodeLit> for Lit {
    fn from(lit: UnicodeLit) -> Lit {
        Lit::Unicode(lit)
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lit::Bool(b) => write!(f, "{b}"),
            Lit::Num(n) => write!(f, "{n}"),
            Lit::String(s) => write!(f, "{s}"),
            Lit::Hex(h) => write!(f, "{h}"),
            Lit::Unicode(u) => write!(f, "{u}"),
        }
    }
}

// ─── BoolLit ───────────────────────────────────────────────────────

impl BoolLit {
    pub fn new(value: bool, span: Option<Loc>) -> Self {
        BoolLit { value, span }
    }
}

impl Display for BoolLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// ─── NumLit ────────────────────────────────────────────────────────

impl NumLit {
    pub fn new(value: Num, span: Option<Loc>) -> Self {
        NumLit { value, span }
    }

    pub fn typ(&self) -> Type {
        self.value.typ()
    }
}

impl Display for NumLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// ─── Num ───────────────────────────────────────────────────────────

impl Num {
    pub fn typ(&self) -> Type {
        match self {
            Num::Int(n) => n.typ.clone(),
            Num::Fixed(n) => n.typ.clone(),
            Num::Hex(n) => n.typ.clone(),
        }
    }

    pub fn one() -> Self {
        Num::Int(IntNum::one())
    }
}

impl From<IntNum> for Num {
    fn from(n: IntNum) -> Self {
        Num::Int(n)
    }
}
impl From<FixedNum> for Num {
    fn from(n: FixedNum) -> Self {
        Num::Fixed(n)
    }
}
impl From<HexNum> for Num {
    fn from(n: HexNum) -> Self {
        Num::Hex(n)
    }
}

impl Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Num::Int(n) => write!(f, "{n}"),
            Num::Fixed(n) => write!(f, "{n}"),
            Num::Hex(n) => write!(f, "{n}"),
        }
    }
}

// ─── IntNum ────────────────────────────────────────────────────────

impl IntNum {
    pub fn new(value: BigInt, typ: Type) -> Self {
        Self { value, typ }
    }

    pub fn one() -> Self {
        Self::new(One::one(), Type::I256)
    }
}

impl Display for IntNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// ─── FixedNum ──────────────────────────────────────────────────────

impl FixedNum {
    pub fn new(value: Decimal, typ: Type) -> Self {
        Self { value, typ }
    }
}

impl Display for FixedNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// ─── HexNum ────────────────────────────────────────────────────────

impl HexNum {
    pub fn new(value: String, typ: Type) -> Self {
        Self { value, typ }
    }
}

impl Display for HexNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// ─── StringLit ─────────────────────────────────────────────────────

impl StringLit {
    pub fn new(value: String, span: Option<Loc>) -> Self {
        StringLit { value, span }
    }
}

impl Display for StringLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

// ─── HexLit ────────────────────────────────────────────────────────

impl HexLit {
    pub fn new(value: String, span: Option<Loc>) -> Self {
        HexLit { value, span }
    }
}

impl Display for HexLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hex\"{}\"", self.value)
    }
}

// ─── UnicodeLit ────────────────────────────────────────────────────

impl UnicodeLit {
    pub fn new(value: String, span: Option<Loc>) -> Self {
        UnicodeLit { value, span }
    }
}

impl Display for UnicodeLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unicode\"{}\"", self.value)
    }
}
