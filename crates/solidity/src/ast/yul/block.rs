//! Module handling Yul block.

use super::YulStmt;
use extlib::string::StringExt;
use std::fmt::{self, Display};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulBlock {
    pub body: Vec<YulStmt>,
}

impl YulBlock {
    pub fn new(statements: Vec<YulStmt>) -> YulBlock {
        YulBlock { body: statements }
    }
}

impl Display for YulBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{{").ok();
        for stmt in self.body.iter() {
            writeln!(f, "{}", format!("{stmt}").indent(4)).ok();
        }
        write!(f, "}}")
    }
}
