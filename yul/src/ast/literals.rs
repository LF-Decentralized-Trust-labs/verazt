use num_bigint::BigInt;
use std::fmt::{self, Display, Formatter};

//-------------------------------------------------------------------------
// Data structures representing all literals
//-------------------------------------------------------------------------

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum Lit {
    Num(NumLit),
    Bool(BoolLit),
    Hex(HexLit),
    Str(StringLit),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum NumLit {
    Hex(String), // FIXME: what is the best way to represent a hex number?
    Dec(BigInt),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct BoolLit {
    pub value: bool,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct HexLit {
    pub value: String,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct StringLit {
    pub value: String,
}

//-------------------------------------------------------------------------
// Implementations for Literal
//-------------------------------------------------------------------------

impl Lit {}

impl From<NumLit> for Lit {
    fn from(lit: NumLit) -> Self {
        Lit::Num(lit)
    }
}

impl From<BoolLit> for Lit {
    fn from(lit: BoolLit) -> Self {
        Lit::Bool(lit)
    }
}

impl From<HexLit> for Lit {
    fn from(lit: HexLit) -> Self {
        Lit::Hex(lit)
    }
}

impl From<StringLit> for Lit {
    fn from(lit: StringLit) -> Self {
        Lit::Str(lit)
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lit::Num(lit) => write!(f, "{lit}"),
            Lit::Bool(lit) => write!(f, "{lit}"),
            Lit::Hex(lit) => write!(f, "{lit}"),
            Lit::Str(lit) => write!(f, "{lit}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Boolean literal
//-------------------------------------------------------------------------

impl BoolLit {
    pub fn new(literal: bool) -> Self {
        BoolLit { value: literal }
    }
}

impl Display for BoolLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for Hex string literal
//-------------------------------------------------------------------------

impl HexLit {
    pub fn new(literal: &str) -> Self {
        HexLit { value: literal.to_string() }
    }
}

impl Display for HexLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hex\"{}\"", self.value)
    }
}

//-------------------------------------------------------------------------
// Number literal
//-------------------------------------------------------------------------

impl NumLit {
    pub fn new_hex(hex: String) -> Self {
        Self::Hex(hex)
    }

    pub fn new_decimal(num: BigInt) -> Self {
        Self::Dec(num)
    }
}

impl Display for NumLit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            NumLit::Hex(num) => write!(f, "{num}"),
            NumLit::Dec(num) => write!(f, "{num}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for String literal
//-------------------------------------------------------------------------

impl StringLit {
    pub fn new(value: &str) -> Self {
        StringLit { value: value.to_string() }
    }
}

impl Display for StringLit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}
