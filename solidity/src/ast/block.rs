use crate::ast::*;
use extlib::string::StringExt;
use crate::ast::Loc;
use std::fmt::{self, Display};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Block {
    pub id: Option<isize>,
    pub body: Vec<Stmt>,
    pub unchecked: bool,
    pub loc: Option<Loc>,
}

impl Block {
    pub fn new(id: Option<isize>, body: Vec<Stmt>, unchecked: bool, loc: Option<Loc>) -> Self {
        Self { id, body, unchecked, loc }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.unchecked {
            write!(f, "unchecked ").ok();
        }

        if self.body.is_empty() {
            write!(f, "{{}}")
        } else {
            writeln!(f, "{{").ok();

            let stmts = self
                .body
                .iter()
                .map(|stmt| format!("{}", stmt).indent(4))
                .collect::<Vec<String>>()
                .join("\n");

            write!(f, "{}\n}}", stmts)
        }
    }
}
