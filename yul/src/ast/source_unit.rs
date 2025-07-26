use crate::ast::*;
use std::fmt::{self, Display};

#[derive(Clone, PartialEq, Debug)]
pub struct SourceUnit {
    pub main_object: Object,
}

impl SourceUnit {
    pub fn new(object: Object) -> Self {
        SourceUnit { main_object: object }
    }
}

impl Display for SourceUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.main_object)
    }
}
