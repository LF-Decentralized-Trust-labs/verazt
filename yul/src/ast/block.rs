//! Module handling Yul block.

use super::Stmt;
use core::stdext::string::StringExt;
use std::fmt::{self, Display};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Block {
    pub body: Vec<Stmt>,
}

impl Block {
    pub fn new(statements: Vec<Stmt>) -> Block {
        Block { body: statements }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{{").ok();
        for stmt in self.body.iter() {
            writeln!(f, "{}", format!("{stmt}").indent(4)).ok();
        }
        write!(f, "}}")
    }
}
