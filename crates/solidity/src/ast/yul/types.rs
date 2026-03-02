//! Module handling Yul types.

use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all data types
//-------------------------------------------------------------------------

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum YulType {
    Int(YulIntType),
    String,
    Bool,
    Tuple(YulTupleType),
    Unkn,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulIntType {
    pub bitwidth: usize,
    pub is_signed: bool,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulTupleType {
    pub elems: Vec<YulType>,
}

//-------------------------------------------------------------------------
// Implementations for Type
//-------------------------------------------------------------------------

impl Display for YulType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YulType::Int(typ) => write!(f, "{typ}"),
            YulType::String => write!(f, "string"),
            YulType::Bool => write!(f, "bool"),
            YulType::Tuple(typ) => write!(f, "{typ}"),
            YulType::Unkn => write!(f, "unresolved_type"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Int type
//-------------------------------------------------------------------------

impl YulIntType {
    pub fn new(bitwidth: usize, signed: bool) -> Self {
        YulIntType { bitwidth, is_signed: signed }
    }
}

impl Display for YulIntType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.is_signed {
            true => write!(f, "int{}", self.bitwidth),
            false => write!(f, "uint{}", self.bitwidth),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Tuple type
//-------------------------------------------------------------------------

impl YulTupleType {
    pub fn new(element_types: Vec<YulType>) -> Self {
        YulTupleType { elems: element_types }
    }
}

impl Display for YulTupleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let elems = self
            .elems
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "({elems})")
    }
}
