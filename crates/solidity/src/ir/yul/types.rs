//! Module handling Yul IR types.

use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all data types
//-------------------------------------------------------------------------

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum YulIRType {
    Int(YulIRIntType),
    String,
    Bool,
    Tuple(YulIRTupleType),
    Unkn,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulIRIntType {
    pub bitwidth: usize,
    pub is_signed: bool,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulIRTupleType {
    pub elems: Vec<YulIRType>,
}

//-------------------------------------------------------------------------
// Implementations for Type
//-------------------------------------------------------------------------

impl Display for YulIRType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YulIRType::Int(typ) => write!(f, "{typ}"),
            YulIRType::String => write!(f, "string"),
            YulIRType::Bool => write!(f, "bool"),
            YulIRType::Tuple(typ) => write!(f, "{typ}"),
            YulIRType::Unkn => write!(f, "unresolved_type"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Int type
//-------------------------------------------------------------------------

impl YulIRIntType {
    pub fn new(bitwidth: usize, signed: bool) -> Self {
        YulIRIntType { bitwidth, is_signed: signed }
    }
}

impl Display for YulIRIntType {
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

impl YulIRTupleType {
    pub fn new(element_types: Vec<YulIRType>) -> Self {
        YulIRTupleType { elems: element_types }
    }
}

impl Display for YulIRTupleType {
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
