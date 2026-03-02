use crate::ast::*;
use crate::ast::{Loc, Name};
use std::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone)]
pub struct Identifier {
    pub id: Option<isize>,
    pub name: Name,
    pub typ: Type,
    pub loc: Option<Loc>,
}

// FIXME: should not implement a specific hash
impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

// FIXME: should not implement a specific Eq trait
impl Eq for Identifier {}

// FIXME: should not implement a specific PartialEq trait
impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        // Compare only name
        self.name == other.name
    }
}

impl Identifier {
    /// Constructor.
    pub fn new(id: Option<isize>, name: Name, typ: Type, loc: Option<Loc>) -> Self {
        Identifier { id, name, typ, loc }
    }

    pub fn set_naming_index(&mut self, index: Option<usize>) {
        self.name.index = index
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
