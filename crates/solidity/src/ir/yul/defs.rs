//! Module handling Yul IR definitions.

use super::*;
use crate::ast::Name;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing definitions
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRFuncDef {
    pub name: String,
    pub params: Vec<YulIRIdentifier>,
    pub returns: Vec<YulIRIdentifier>,
    pub body: YulIRBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRVarDecl {
    pub vars: Vec<YulIRIdentifier>,
    pub value: Option<YulIRExpr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRIdentifier {
    pub name: Name,
    pub typ: YulIRType,
}

//-------------------------------------------------------------------------
// Implementations for Function definition
//-------------------------------------------------------------------------

impl YulIRFuncDef {
    pub fn new(
        name: &str,
        params: Vec<YulIRIdentifier>,
        returns: Vec<YulIRIdentifier>,
        body: YulIRBlock,
    ) -> YulIRFuncDef {
        YulIRFuncDef { name: name.to_string(), params, returns, body }
    }
}

impl Display for YulIRFuncDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f).ok();
        let params = self
            .params
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "function {}({}) ", self.name, params).ok();
        let returns = self
            .returns
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        if !self.returns.is_empty() {
            write!(f, "-> {returns}").ok();
        }
        write!(f, "{}", self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for Variable declaration
//-------------------------------------------------------------------------

impl YulIRVarDecl {
    pub fn new(variables: Vec<YulIRIdentifier>, value: Option<YulIRExpr>) -> YulIRVarDecl {
        YulIRVarDecl { vars: variables, value }
    }
}

impl Display for YulIRVarDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.vars.is_empty() {
            panic!("VariableDeclaration must have identifiers")
        }
        write!(f, "let ").ok();
        let vars = self
            .vars
            .iter()
            .map(|v| match v.typ {
                YulIRType::Unkn => v.name.to_string(),
                _ => format!("{}: {}", v.name, v.typ),
            })
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{vars}").ok();
        if let Some(expr) = &self.value {
            write!(f, " := {expr}").ok();
        }
        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementations for Identifier
//-------------------------------------------------------------------------

impl YulIRIdentifier {
    pub fn new(name: Name, typ: YulIRType) -> Self {
        YulIRIdentifier { name, typ }
    }
}

impl Display for YulIRIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
