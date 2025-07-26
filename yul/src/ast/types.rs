use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all data types
//-------------------------------------------------------------------------

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Int(IntType),
    String,
    Bool,
    Tuple(TupleType),
    Unkn,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct IntType {
    pub bitwidth: usize,
    pub is_signed: bool,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct TupleType {
    pub elems: Vec<Type>,
}

//-------------------------------------------------------------------------
// Implementations for Type
//-------------------------------------------------------------------------

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int(typ) => write!(f, "{typ}"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Tuple(typ) => write!(f, "{typ}"),
            Type::Unkn => write!(f, "unresolved_type"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Int type
//-------------------------------------------------------------------------

impl IntType {
    pub fn new(bitwidth: usize, signed: bool) -> Self {
        IntType { bitwidth, is_signed: signed }
    }
}

impl Display for IntType {
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

impl TupleType {
    pub fn new(element_types: Vec<Type>) -> Self {
        TupleType { elems: element_types }
    }
}

impl Display for TupleType {
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
