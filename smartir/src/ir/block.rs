use crate::ir::*;
use core::stdext::stringext::StringExt;
use std::fmt::{self, Display};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub is_unchecked: bool,
    pub loc: Option<Loc>,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, is_unchecked: bool, loc: Option<Loc>) -> Self {
        Self { is_unchecked, stmts, loc }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_unchecked {
            write!(f, "unchecked ").ok();
        }
        if self.stmts.is_empty() {
            write!(f, "{{}}")
        } else {
            writeln!(f, "{{").ok();
            let stmts = self
                .stmts
                .iter()
                .map(|stmt| format!("{stmt}").indent(4))
                .collect::<Vec<String>>()
                .join("\n");
            write!(f, "{stmts}\n}}")
        }
    }
}
