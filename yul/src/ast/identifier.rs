use crate::ast::*;
use std::fmt::{self, Display};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Identifier {
    pub name: Name,
    pub typ: Type,
    pub loc: Option<Loc>,
}

impl Identifier {
    pub fn new(name: Name, typ: Type, loc: Option<Loc>) -> Self {
        Identifier { name, typ, loc }
    }

    pub fn set_naming_index(&mut self, index: Option<usize>) {
        self.name.index = index
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
