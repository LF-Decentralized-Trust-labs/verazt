//! Module handling Yul sections (Object, Code, Data).

use super::{YulBlock, YulHexLit, YulStringLit};
use either::Either::{self, Left, Right};
use extlib::string::StringExt;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing Yul sections
//-------------------------------------------------------------------------

/// An object section in a Yul source unit.
#[derive(Clone, PartialEq, Debug)]
pub struct YulObject {
    pub name: String,
    pub code: YulCode,
    pub children: Vec<Either<YulObject, YulData>>,
}

/// A code section in a Yul source unit.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulCode {
    pub body: YulBlock,
}

/// A data section in a Yul source unit.
#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulData {
    pub name: YulStringLit,
    pub content: Either<YulHexLit, YulStringLit>,
}

/// A comment section in Yul source unit.
#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct YulComment {
    pub content: String,
}

//-------------------------------------------------------------------------
// Implementations for Object section
//-------------------------------------------------------------------------

impl YulObject {
    pub fn new(name: String, code: YulCode, children: Vec<Either<YulObject, YulData>>) -> Self {
        YulObject { name, code, children }
    }
}

impl Display for YulObject {
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

impl YulCode {
    pub fn new(body: YulBlock) -> Self {
        YulCode { body }
    }
}

impl Display for YulCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "code {}", self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for Data section
//-------------------------------------------------------------------------

impl YulData {
    pub fn new(name: YulStringLit, content: Either<YulHexLit, YulStringLit>) -> Self {
        YulData { name, content }
    }
}

impl Display for YulData {
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

impl YulComment {
    pub fn new(comment: String) -> Self {
        YulComment { content: comment }
    }
}

impl Display for YulComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}
