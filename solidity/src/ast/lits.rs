use crate::ast::*;
use meta::Loc;
use num_bigint::BigInt;
use rust_decimal::Decimal;
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
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HexLit {
    pub value: String,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NumLit {
    pub value: Num,
    pub unit: Option<NumUnit>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Num {
    Int(IntNum),
    FixedNum(FixedNum),
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

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NumUnit {
    Wei,
    Gwei,   // 1 gwei = 10 ** 9 wei
    Szabo,  // 1 szabo = 10 ** 12 wei
    Finney, // (1 finney = 10 ** 15 wei
    Ether,  // An ether (1 ether = 10 ** 18 wei).
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Years,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StringLit {
    pub value: String,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UnicodeLit {
    pub value: String,
    pub typ: Type,
    pub loc: Option<Loc>,
}

//-------------------------------------------------------------------------
// Implementation for Lit
//-------------------------------------------------------------------------

impl Lit {
    pub fn loc(&self) -> Option<Loc> {
        match self {
            Lit::Bool(b) => b.loc,
            Lit::Num(n) => n.loc,
            Lit::String(s) => s.loc,
            Lit::Hex(h) => h.loc,
            Lit::Unicode(u) => u.loc,
        }
    }

    pub fn typ(&self) -> Type {
        match self {
            Lit::Bool(b) => b.typ.clone(),
            Lit::Num(n) => n.value.typ(),
            Lit::String(s) => s.typ.clone(),
            Lit::Hex(h) => h.typ.clone(),
            Lit::Unicode(u) => u.typ.clone(),
        }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        match self {
            Lit::Bool(lit) => lit.update_data_type(new_type),
            Lit::Num(lit) => lit.update_data_type(new_type),
            Lit::String(lit) => lit.update_data_type(new_type),
            Lit::Hex(lit) => lit.update_data_type(new_type),
            Lit::Unicode(lit) => lit.update_data_type(new_type),
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
// Implementation for Boolean literals
//-------------------------------------------------------------------------

impl BoolLit {
    pub fn new(value: bool, typ: Type, loc: Option<Loc>) -> Self {
        BoolLit { value, typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for BoolLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementation for Hex literals
//-------------------------------------------------------------------------

impl HexLit {
    pub fn new(value: &str, typ: Type, loc: Option<Loc>) -> Self {
        HexLit { value: value.to_string(), typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for HexLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hex\"{}\"", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementation for number literals
//-------------------------------------------------------------------------

impl NumLit {
    pub fn new(value: Num, unit: Option<NumUnit>, loc: Option<Loc>) -> Self {
        NumLit { value, unit, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.value.update_data_type(new_type)
    }
}

impl Display for NumLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.unit {
            Some(u) => write!(f, "{} {}", self.value, u),
            None => write!(f, "{}", self.value),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Num
//-------------------------------------------------------------------------

impl Num {
    pub fn typ(&self) -> Type {
        match self {
            Num::Int(num) => num.typ.clone(),
            Num::FixedNum(num) => num.typ.clone(),
            Num::Hex(hex) => hex.typ.clone(),
        }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        match self {
            Num::Int(lit) => lit.update_data_type(new_type),
            Num::FixedNum(lit) => lit.update_data_type(new_type),
            Num::Hex(lit) => lit.update_data_type(new_type),
        }
    }
}

impl From<IntNum> for Num {
    fn from(n: IntNum) -> Self {
        Num::Int(n)
    }
}

impl From<FixedNum> for Num {
    fn from(n: FixedNum) -> Self {
        Num::FixedNum(n)
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
            Num::FixedNum(n) => write!(f, "{n}"),
            Num::Hex(s) => write!(f, "{s}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for IntNum
//-------------------------------------------------------------------------

impl IntNum {
    pub fn new(value: BigInt, typ: Type) -> Self {
        Self { value, typ }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for IntNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementation for FixedNum
//-------------------------------------------------------------------------

impl FixedNum {
    pub fn new(value: Decimal, typ: Type) -> Self {
        Self { value, typ }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for FixedNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementation for HexNum
//-------------------------------------------------------------------------

impl HexNum {
    pub fn new(value: String, typ: Type) -> Self {
        Self { value, typ }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for HexNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementation for HexNum
//-------------------------------------------------------------------------

impl NumUnit {
    pub fn new(unit: &str) -> Self {
        match unit {
            "wei" => NumUnit::Wei,
            "gwei" => NumUnit::Gwei,
            "szabo" => NumUnit::Szabo,
            "finney" => NumUnit::Finney,
            "ether" => NumUnit::Ether,
            "seconds" => NumUnit::Seconds,
            "minutes" => NumUnit::Minutes,
            "hours" => NumUnit::Hours,
            "days" => NumUnit::Days,
            "weeks" => NumUnit::Weeks,
            "years" => NumUnit::Years,
            _ => panic!("Number unit not found: {unit}"),
        }
    }
}

impl Display for NumUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumUnit::Wei => write!(f, "wei"),
            NumUnit::Gwei => write!(f, "gwei"),
            NumUnit::Szabo => write!(f, "szabo"),
            NumUnit::Finney => write!(f, "finney"),
            NumUnit::Ether => write!(f, "ether"),
            NumUnit::Seconds => write!(f, "seconds"),
            NumUnit::Minutes => write!(f, "minutes"),
            NumUnit::Hours => write!(f, "hours"),
            NumUnit::Days => write!(f, "days"),
            NumUnit::Weeks => write!(f, "weeks"),
            NumUnit::Years => write!(f, "years"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for String literals
//-------------------------------------------------------------------------

impl StringLit {
    pub fn new(value: String, typ: Type, loc: Option<Loc>) -> Self {
        StringLit { value, typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for StringLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementation for Unicode literal
//-------------------------------------------------------------------------

impl UnicodeLit {
    pub fn new(value: String, typ: Type, loc: Option<Loc>) -> Self {
        UnicodeLit { value, typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for UnicodeLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unicode\"{}\"", self.value)
    }
}
