use super::{Block, HexLit, StringLit};
use base::string::StringExt;
use either::Either::{self, Left, Right};
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing Yul sections
//-------------------------------------------------------------------------

/// An object section in a Yul source unit.
#[derive(Clone, PartialEq, Debug)]
pub struct Object {
    pub name: String,
    pub code: Code,
    pub children: Vec<Either<Object, Data>>,
}

/// A code section in a Yul source unit.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Code {
    pub body: Block,
}

/// A data section in a Yul source unit.
#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct Data {
    pub name: StringLit,
    pub content: Either<HexLit, StringLit>,
}

/// A comment section in Yul source unit.
#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct Comment {
    pub content: String,
}

//-------------------------------------------------------------------------
// Implementations for Object section
//-------------------------------------------------------------------------

impl Object {
    pub fn new(name: String, code: Code, children: Vec<Either<Object, Data>>) -> Self {
        Object { name, code, children }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "object {} {{", self.name).ok();
        write!(f, "{}", format!("{}", self.code).indent(4)).ok();
        for child in &self.children {
            let child = match child {
                Left(object) => format!("{object}").indent(4),
                Right(data) => format!("{data}").indent(4),
            };
            write!(f, "\n{child}").ok();
        }
        write!(f, "\n}}")
    }
}

//-------------------------------------------------------------------------
// Implementations for Code section
//-------------------------------------------------------------------------

impl Code {
    pub fn new(body: Block) -> Self {
        Code { body }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "code {}", self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for Data section
//-------------------------------------------------------------------------

impl Data {
    pub fn new(name: StringLit, content: Either<HexLit, StringLit>) -> Self {
        Data { name, content }
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data {} ", self.name).ok();
        match &self.content {
            Left(hex) => write!(f, "{hex}"),
            Right(string) => write!(f, "{string}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Comment section
//-------------------------------------------------------------------------

impl Comment {
    pub fn new(comment: String) -> Self {
        Comment { content: comment }
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
