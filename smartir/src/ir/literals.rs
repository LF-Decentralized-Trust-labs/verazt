use num_bigint::BigInt;
use num_traits::One;
use rust_decimal::Decimal;

use crate::ir::*;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all literals
//-------------------------------------------------------------------------

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
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NumLit {
    pub value: Num,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StringLit {
    pub value: String,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HexLit {
    pub value: String,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UnicodeLit {
    pub value: String,
    pub loc: Option<Loc>,
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
    // REVIEW: should be restricted to [`IntType`]?
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

//-------------------------------------------------------------------------
// Implementations for Literals
//-------------------------------------------------------------------------

impl Lit {
    pub fn one(loc: Option<Loc>) -> Self {
        Lit::from(NumLit::new(Num::one(), loc))
    }

    pub fn typ(&self) -> Type {
        match self {
            Lit::Bool(_) => Type::Bool,
            Lit::Num(n) => n.typ(),
            Lit::String(s) => s.typ(),
            Lit::Hex(h) => h.typ(),
            Lit::Unicode(u) => u.typ(),
        }
    }

    pub fn loc(&self) -> Option<Loc> {
        match self {
            Lit::Bool(b) => b.loc,
            Lit::Num(n) => n.loc,
            Lit::String(s) => s.loc,
            Lit::Hex(h) => h.loc,
            Lit::Unicode(u) => u.loc,
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
            Lit::Hex(s) => write!(f, "{s}"),
            Lit::Unicode(s) => write!(f, "{s}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Boolean literals
//-------------------------------------------------------------------------

impl BoolLit {
    pub fn new(value: bool, loc: Option<Loc>) -> Self {
        BoolLit { value, loc }
    }
}

impl Display for BoolLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for Hex literals
//-------------------------------------------------------------------------

impl HexLit {
    pub fn new(value: String, loc: Option<Loc>) -> Self {
        HexLit { value, loc }
    }

    pub fn typ(&self) -> Type {
        // REVIEW: temporarily set data location to Memory.
        //
        // what is the correct data location of string literal?
        StringType::new(Some(DataLoc::Memory), false).into()
    }
}

impl Display for HexLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hex\"{}\"", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for Num literals
//-------------------------------------------------------------------------

impl NumLit {
    pub fn new(value: Num, loc: Option<Loc>) -> Self {
        NumLit { value, loc }
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

//-------------------------------------------------------------------------
// Implementations for Num
//-------------------------------------------------------------------------

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

//-------------------------------------------------------------------------
// Implementations for IntNum
//-------------------------------------------------------------------------

impl IntNum {
    pub fn new(value: BigInt, typ: Type) -> Self {
        Self { value, typ }
    }

    pub fn one() -> Self {
        let uint_type = Type::Int(IntType::new(None, false));
        Self::new(One::one(), uint_type)
    }
}

impl Display for IntNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for FixedNum
//-------------------------------------------------------------------------

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

//-------------------------------------------------------------------------
// Implementations for HexNum
//-------------------------------------------------------------------------

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

//-------------------------------------------------------------------------
// Implementations for String literal
//-------------------------------------------------------------------------

impl StringLit {
    pub fn new(value: String, loc: Option<Loc>) -> Self {
        StringLit { value, loc }
    }

    pub fn typ(&self) -> Type {
        // REVIEW: temporarily set data location to Memory.
        //
        // what is the correct data location of string literal?
        StringType::new(Some(DataLoc::Memory), false).into()
    }
}

impl Display for StringLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for Unicode literal
//-------------------------------------------------------------------------

impl UnicodeLit {
    pub fn new(value: String, loc: Option<Loc>) -> Self {
        UnicodeLit { value, loc }
    }

    pub fn typ(&self) -> Type {
        // REVIEW: temporarily set data location to Memory.
        //
        // what is the correct data location of string literal?
        StringType::new(Some(DataLoc::Memory), false).into()
    }
}

impl Display for UnicodeLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unicode\"{}\"", self.value)
    }
}
