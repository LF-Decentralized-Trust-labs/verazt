use crate::ast::Loc;

use crate::ir::*;
use std::fmt::{self, Display};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub typ: Type,
    pub loc: Option<Loc>,
}

impl Variable {
    pub fn new(name: String, typ: Type, loc: Option<Loc>) -> Self {
        Variable { name, typ, loc }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
