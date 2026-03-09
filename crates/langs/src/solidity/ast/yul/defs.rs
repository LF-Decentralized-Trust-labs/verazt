//! Module handling Yul definitions.

use super::*;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing definitions
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulFuncDef {
    pub name: String,
    pub params: Vec<YulIdentifier>,
    pub returns: Vec<YulIdentifier>,
    pub body: YulBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulVarDecl {
    pub vars: Vec<YulIdentifier>,
    pub value: Option<YulExpr>,
}

//-------------------------------------------------------------------------
// Implementations for Function definition
//-------------------------------------------------------------------------

impl YulFuncDef {
    pub fn new(
        name: &str,
        params: Vec<YulIdentifier>,
        returns: Vec<YulIdentifier>,
        body: YulBlock,
    ) -> YulFuncDef {
        YulFuncDef { name: name.to_string(), params, returns, body }
    }
}

impl Display for YulFuncDef {
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

impl YulVarDecl {
    pub fn new(variables: Vec<YulIdentifier>, value: Option<YulExpr>) -> YulVarDecl {
        YulVarDecl { vars: variables, value }
    }
}

impl Display for YulVarDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME: should validate this on the new/default trait
        if self.vars.is_empty() {
            panic!("VariableDeclaration must have identifiers")
        }
        write!(f, "let ").ok();
        let vars = self
            .vars
            .iter()
            .map(|v| match v.typ {
                YulType::Unkn => v.name.to_string(),
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
