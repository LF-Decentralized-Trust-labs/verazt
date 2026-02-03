//! Module handling Yul identifier.

use super::*;
use crate::ast::{Loc, Name};
use std::fmt::{self, Display};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIdentifier {
    pub name: Name,
    pub typ: YulType,
    pub loc: Option<Loc>,
}

impl YulIdentifier {
    pub fn new(name: Name, typ: YulType, loc: Option<Loc>) -> Self {
        YulIdentifier { name, typ, loc }
    }

    pub fn set_naming_index(&mut self, index: Option<usize>) {
        self.name.index = index
    }
}

impl Display for YulIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
