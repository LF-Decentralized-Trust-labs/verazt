//! Module handling Yul source unit.

use super::*;
use std::fmt::{self, Display};

#[derive(Clone, PartialEq, Debug)]
pub struct YulSourceUnit {
    pub main_object: YulObject,
}

impl YulSourceUnit {
    pub fn new(object: YulObject) -> Self {
        YulSourceUnit { main_object: object }
    }
}

impl Display for YulSourceUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.main_object)
    }
}
