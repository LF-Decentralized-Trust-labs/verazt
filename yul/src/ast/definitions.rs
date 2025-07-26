use crate::ast::*;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing definitions
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Identifier>,
    pub returns: Vec<Identifier>,
    pub body: Block,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VarDecl {
    pub vars: Vec<Identifier>,
    pub value: Option<Expr>,
}

//-------------------------------------------------------------------------
// Implementations for Function definition
//-------------------------------------------------------------------------

impl FuncDef {
    pub fn new(
        name: &str,
        params: Vec<Identifier>,
        returns: Vec<Identifier>,
        body: Block,
    ) -> FuncDef {
        FuncDef { name: name.to_string(), params, returns, body }
    }
}

impl Display for FuncDef {
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

impl VarDecl {
    pub fn new(variables: Vec<Identifier>, value: Option<Expr>) -> VarDecl {
        VarDecl { vars: variables, value }
    }
}

impl Display for VarDecl {
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
                Type::Unkn => v.name.to_string(),
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
