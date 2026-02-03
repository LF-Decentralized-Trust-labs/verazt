//! Module handling Yul IR literals.

use num_bigint::BigInt;
use std::fmt::{self, Display, Formatter};

//-------------------------------------------------------------------------
// Data structures representing all literals
//-------------------------------------------------------------------------

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum YulIRLit {
    Num(YulIRNumLit),
    Bool(YulIRBoolLit),
    Hex(YulIRHexLit),
    Str(YulIRStringLit),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum YulIRNumLit {
    Hex(String),
    Dec(BigInt),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulIRBoolLit {
    pub value: bool,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulIRHexLit {
    pub value: String,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulIRStringLit {
    pub value: String,
}

//-------------------------------------------------------------------------
// Implementations for Literal
//-------------------------------------------------------------------------

impl From<YulIRNumLit> for YulIRLit {
    fn from(lit: YulIRNumLit) -> Self {
        YulIRLit::Num(lit)
    }
}

impl From<YulIRBoolLit> for YulIRLit {
    fn from(lit: YulIRBoolLit) -> Self {
        YulIRLit::Bool(lit)
    }
}

impl From<YulIRHexLit> for YulIRLit {
    fn from(lit: YulIRHexLit) -> Self {
        YulIRLit::Hex(lit)
    }
}

impl From<YulIRStringLit> for YulIRLit {
    fn from(lit: YulIRStringLit) -> Self {
        YulIRLit::Str(lit)
    }
}

impl Display for YulIRLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YulIRLit::Num(lit) => write!(f, "{lit}"),
            YulIRLit::Bool(lit) => write!(f, "{lit}"),
            YulIRLit::Hex(lit) => write!(f, "{lit}"),
            YulIRLit::Str(lit) => write!(f, "{lit}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Boolean literal
//-------------------------------------------------------------------------

impl YulIRBoolLit {
    pub fn new(literal: bool) -> Self {
        YulIRBoolLit { value: literal }
    }
}

impl Display for YulIRBoolLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for Hex string literal
//-------------------------------------------------------------------------

impl YulIRHexLit {
    pub fn new(literal: &str) -> Self {
        YulIRHexLit { value: literal.to_string() }
    }
}

impl Display for YulIRHexLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hex\"{}\"", self.value)
    }
}

//-------------------------------------------------------------------------
// Number literal
//-------------------------------------------------------------------------

impl YulIRNumLit {
    pub fn new_hex(hex: String) -> Self {
        Self::Hex(hex)
    }

    pub fn new_decimal(num: BigInt) -> Self {
        Self::Dec(num)
    }
}

impl Display for YulIRNumLit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            YulIRNumLit::Hex(num) => write!(f, "{num}"),
            YulIRNumLit::Dec(num) => write!(f, "{num}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for String literal
//-------------------------------------------------------------------------

impl YulIRStringLit {
    pub fn new(value: &str) -> Self {
        YulIRStringLit { value: value.to_string() }
    }
}

impl Display for YulIRStringLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}
