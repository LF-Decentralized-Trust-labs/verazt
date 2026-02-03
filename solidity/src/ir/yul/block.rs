//! Module handling Yul IR block.

use super::YulIRStmt;
use extlib::string::StringExt;
use std::fmt::{self, Display};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRBlock {
    pub body: Vec<YulIRStmt>,
}

impl YulIRBlock {
    pub fn new(statements: Vec<YulIRStmt>) -> YulIRBlock {
        YulIRBlock { body: statements }
    }
}

impl Display for YulIRBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{{").ok();
        for stmt in self.body.iter() {
            writeln!(f, "{}", format!("{stmt}").indent(4)).ok();
        }
        write!(f, "}}")
    }
}
