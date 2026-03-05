//! Vyper AST definition nodes (FuncDef, EventDef, StructDef, etc.).

use super::exprs::Expr;
use super::loc::Loc;
use super::stmts::Stmt;
use super::types::Type;

/// Function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub decorators: Vec<FuncDecorator>,
    pub doc_string: Option<String>,
    pub body: Vec<Stmt>,
    pub loc: Option<Loc>,
}

/// Function decorators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuncDecorator {
    Deploy,
    External,
    Internal,
    View,
    Pure,
    Payable,
    NonReentrant(Option<String>),
    Custom(String),
}

/// State variable declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateVarDecl {
    pub name: String,
    pub typ: Type,
    pub constant: bool,
    pub immutable: bool,
    pub doc_string: Option<String>,
    pub loc: Option<Loc>,
}

/// Event definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDef {
    pub name: String,
    pub fields: Vec<EventField>,
    pub loc: Option<Loc>,
}

/// Event field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventField {
    pub name: String,
    pub typ: Type,
    pub indexed: bool,
}

/// Struct definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub loc: Option<Loc>,
}

/// Struct field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub typ: Type,
}

/// Interface definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDef {
    pub name: String,
    pub funcs: Vec<InterfaceFunc>,
    pub loc: Option<Loc>,
}

/// Interface function stub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceFunc {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub mutability: Option<String>,
}

/// Enum definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<String>,
    pub loc: Option<Loc>,
}

/// Flag definition (Vyper's bit-flag enum).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagDef {
    pub name: String,
    pub variants: Vec<String>,
    pub loc: Option<Loc>,
}

/// Type alias: `type Foo = Bar`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias {
    pub name: String,
    pub typ: Type,
    pub loc: Option<Loc>,
}

/// Import statement (Vyper 0.4+).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStmt {
    pub module: String,
    pub alias: Option<String>,
    pub loc: Option<Loc>,
}

/// Version annotation: `# @version ^0.3.9`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionAnnotation {
    pub version: String,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub typ: Type,
    pub default: Option<Expr>,
    pub loc: Option<Loc>,
}
