//! Module handling Yul literals.

use num_bigint::BigInt;
use std::fmt::{self, Display, Formatter};

//-------------------------------------------------------------------------
// Data structures representing all literals
//-------------------------------------------------------------------------

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum YulLit {
    Num(YulNumLit),
    Bool(YulBoolLit),
    Hex(YulHexLit),
    Str(YulStringLit),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum YulNumLit {
    Hex(String), // FIXME: what is the best way to represent a hex number?
    Dec(BigInt),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulBoolLit {
    pub value: bool,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulHexLit {
    pub value: String,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulStringLit {
    pub value: String,
}

//-------------------------------------------------------------------------
// Implementations for Literal
//-------------------------------------------------------------------------

impl YulLit {}

impl From<YulNumLit> for YulLit {
    fn from(lit: YulNumLit) -> Self {
        YulLit::Num(lit)
    }
}

impl From<YulBoolLit> for YulLit {
    fn from(lit: YulBoolLit) -> Self {
        YulLit::Bool(lit)
    }
}

impl From<YulHexLit> for YulLit {
    fn from(lit: YulHexLit) -> Self {
        YulLit::Hex(lit)
    }
}

impl From<YulStringLit> for YulLit {
    fn from(lit: YulStringLit) -> Self {
        YulLit::Str(lit)
    }
}

impl Display for YulLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YulLit::Num(lit) => write!(f, "{lit}"),
            YulLit::Bool(lit) => write!(f, "{lit}"),
            YulLit::Hex(lit) => write!(f, "{lit}"),
            YulLit::Str(lit) => write!(f, "{lit}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Boolean literal
//-------------------------------------------------------------------------

impl YulBoolLit {
    pub fn new(literal: bool) -> Self {
        YulBoolLit { value: literal }
    }
}

impl Display for YulBoolLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for Hex string literal
//-------------------------------------------------------------------------

impl YulHexLit {
    pub fn new(literal: &str) -> Self {
        YulHexLit { value: literal.to_string() }
    }
}

impl Display for YulHexLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hex\"{}\"", self.value)
    }
}

//-------------------------------------------------------------------------
// Number literal
//-------------------------------------------------------------------------

impl YulNumLit {
    pub fn new_hex(hex: String) -> Self {
        Self::Hex(hex)
    }

    pub fn new_decimal(num: BigInt) -> Self {
        Self::Dec(num)
    }
}

impl Display for YulNumLit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            YulNumLit::Hex(num) => write!(f, "{num}"),
            YulNumLit::Dec(num) => write!(f, "{num}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for String literal
//-------------------------------------------------------------------------

impl YulStringLit {
    pub fn new(value: &str) -> Self {
        YulStringLit { value: value.to_string() }
    }
}

impl Display for YulStringLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}
