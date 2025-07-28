//! Module handling JSON Value.

use core::metadata::DataLoc;

use crate::{ast::*, parser::localizer::Localizer};
use color_eyre::eyre::{Result, bail, eyre};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Value;

/// Data structure describing node type.
#[derive(PartialEq, Eq, Debug, Clone)]
#[remain::sorted]
pub enum NodeType {
    ArrayTypeName,
    Assign,
    BinaryOperation,
    Block,
    BreakStmt,
    Conditional,
    ConstructorDef,
    ContinueStmt,
    ContractDef,
    DoWhileStmt,
    ElementaryTypeName,
    ElementaryTypeNameExpr,
    EmitStmt,
    EnumDef,
    ErrorDef,
    EventDef,
    ExprStmt,
    FallbackFuncDef,
    ForStmt,
    FuncCall,
    FuncCallOpts,
    FuncDef,
    FuncTypeName,
    Ident,
    IdentPath,
    IfStmt,
    Import,
    IndexAccess,
    IndexRangeAccess,
    InheritanceSpec,
    InlineAssembly,
    InterfaceDef,
    LibraryDef,
    Literal,
    Mapping,
    MemberAccess,
    ModifierDef,
    ModifierInvoc,
    NewExpr,
    OverrideSpecifier,
    ParameterList,
    PlaceholderStmt,
    Pragma,
    ReceiveFuncDef,
    ReturnStmt,
    RevertStmt,
    SourceUnit,
    StateVarDecl,
    StructDef,
    ThrowStmt,
    TryStmt,
    TupleExpr,
    UnaryOperation,
    UncheckedBlock,
    UserDefinedTypeName,
    UserDefinedValueTypeDef,
    Using,
    VarDecl,
    VarDeclStmt,
    WhileStmt,
    YulAssign,
    YulBlock,
    YulBreakStmt,
    YulContinueStmt,
    YulExprStmt,
    YulForLoop,
    YulFuncCall,
    YulFuncDef,
    YulIdent,
    YulIfStmt,
    YulLeaveStmt,
    YulLiteral,
    YulSwitchStmt,
    YulVarDecl,
}

impl NodeType {
    pub fn new(node_type: &str) -> Self {
        use NodeType::*;
        match node_type {
            "ArrayTypeName" => ArrayTypeName,
            "Assignment" => Assign,
            "BinaryOperation" => BinaryOperation,
            "Block" => Block,
            "Break" => BreakStmt,
            "Conditional" => Conditional,
            "ConstructorDefinition" => ConstructorDef,
            "Continue" => ContinueStmt,
            "ContractDefinition" => ContractDef,
            "DoWhileStatement" => DoWhileStmt,
            "EmitStatement" => EmitStmt,
            "ElementaryTypeName" => ElementaryTypeName,
            "ElementaryTypeNameExpression" => ElementaryTypeNameExpr,
            "EnumDefinition" => EnumDef,
            "ErrorDefinition" => ErrorDef,
            "EventDefinition" => EventDef,
            "ExpressionStatement" => ExprStmt,
            "FallbackFunctionDefinition" => FallbackFuncDef,
            "ForStatement" => ForStmt,
            "FunctionCall" => FuncCall,
            "FunctionCallOptions" => FuncCallOpts,
            "FunctionDefinition" => FuncDef,
            "FunctionTypeName" => FuncTypeName,
            "Identifier" => Ident,
            "IdentifierPath" => IdentPath,
            "IfStatement" => IfStmt,
            "ImportDirective" => Import,
            "IndexAccess" => IndexAccess,
            "IndexRangeAccess" => IndexRangeAccess,
            "InheritanceSpecifier" => InheritanceSpec,
            "InlineAssembly" => InlineAssembly,
            "InterfaceDefinition" => InterfaceDef,
            "LibraryDefinition" => LibraryDef,
            "Literal" => Literal,
            "Mapping" => Mapping,
            "MemberAccess" => MemberAccess,
            "ModifierDefinition" => ModifierDef,
            "ModifierInvocation" => ModifierInvoc,
            "NewExpression" => NewExpr,
            "OverrideSpecifier" => OverrideSpecifier,
            "ParameterList" => ParameterList,
            "PlaceholderStatement" => PlaceholderStmt,
            "PragmaDirective" => Pragma,
            "ReceiveFunctionDefinition" => ReceiveFuncDef,
            "Return" => ReturnStmt,
            "RevertStatement" => RevertStmt,
            "SourceUnit" => SourceUnit,
            "StateVariableDeclaration" => StateVarDecl,
            "StructDefinition" => StructDef,
            "Throw" => ThrowStmt,
            "TupleExpression" => TupleExpr,
            "TryStatement" => TryStmt,
            "UnaryOperation" => UnaryOperation,
            "UncheckedBlock" => UncheckedBlock,
            "UserDefinedTypeName" => UserDefinedTypeName,
            "UserDefinedValueTypeDefinition" => UserDefinedValueTypeDef,
            "UsingForDirective" => Using,
            "VariableDeclaration" => VarDecl,
            "VariableDeclarationStatement" => VarDeclStmt,
            "WhileStatement" => WhileStmt,
            "YulAssignment" => YulAssign,
            "YulBlock" => YulBlock,
            "YulBreak" => YulBreakStmt,
            "YulContinue" => YulContinueStmt,
            "YulForLoop" => YulForLoop,
            "YulFunctionCall" => YulFuncCall,
            "YulExpressionStatement" => YulExprStmt,
            "YulFunctionDefinition" => YulFuncDef,
            "YulIdentifier" => YulIdent,
            "YulIf" => YulIfStmt,
            "YulLiteral" => YulLiteral,
            "YulLeave" => YulLeaveStmt,
            "YulSwitch" => YulSwitchStmt,
            "YulVariableDeclaration" => YulVarDecl,
            _ => panic!("Unknown node type: {node_type}"),
        }
    }
}

lazy_static! {
    /// Regular expression to parse source code location in JSON AST data.
    pub static ref LOCATION_REGEX: Regex = Regex::new(r"(\d+):(\d+):(\d+)")
        .unwrap_or_else(|_| panic!("Invalid regular expression!"));
}

/// Construct location from source location string in JSON AST.
///
/// The input `location` string is of the form: `position:length:others`.
pub fn parse_loc(localizer: &Localizer, location: &str) -> Option<Loc> {
    LOCATION_REGEX.captures(location).and_then(|capture| {
        match (capture.get(1), capture.get(2)) {
            (Some(pos), Some(len)) => {
                let begin_pos: usize = match pos.as_str().parse() {
                    Ok(n) => n,
                    Err(err) => panic!("Error while getting source posistion: {err}"),
                };
                let len: usize = match len.as_str().parse() {
                    Ok(n) => n,
                    Err(err) => panic!("Error while getting source length: {err}"),
                };
                let end_pos = begin_pos + len - 1;
                // Get line column information.
                let (l1, c1) = match localizer.get_line_column(begin_pos) {
                    Some((l, c)) => (l as isize, c as isize),
                    None => return None,
                };
                let (l2, c2) = match localizer.get_line_column(end_pos) {
                    Some((l, c)) => (l as isize, c as isize),
                    None => return None,
                };
                // Rectify line number from zero-based to one-based.
                let (l1, l2) = (l1 + 1, l2 + 1);
                Some(Loc::new(l1, c1, l2, c2))
            }
            _ => None,
        }
    })
}

/// Trait to provide more utilities to hand a JSON AST node.
pub trait ValueUtil {
    /// Get value of the `block` property.
    fn get_block(&self) -> Result<&Value>;

    /// Get value of the `body` property.
    fn get_body(&self) -> Result<&Value>;

    /// Get value of the `cases` property.
    fn get_cases(&self) -> Result<&Value>;

    /// Get value of the `expression` property.
    fn get_expression(&self) -> Result<&Value>;

    /// Get value of the `kind` property.
    fn get_kind(&self) -> Result<String>;

    /// Get value of the `name` property.
    fn get_name(&self) -> Result<String>;

    /// Get value of the `scope` property.
    fn get_scope(&self) -> Result<isize>;

    /// Get location in source code.
    ///
    /// A source location is of the following form with inclusive range:
    /// (`start_line`:`start_column` -> `end_line`:`end_column`).
    fn get_source_location(&self, localizer: &Option<Localizer>) -> Option<Loc>;
}

impl ValueUtil for Value {
    fn get_block(&self) -> Result<&Value> {
        let key = "block";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_body(&self) -> Result<&Value> {
        let key = "body";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_cases(&self) -> Result<&Value> {
        let key = "cases";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_expression(&self) -> Result<&Value> {
        let key = "expression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_kind(&self) -> Result<String> {
        let key = "kind";
        match self.get(key) {
            Some(Value::String(s)) => Ok(s.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_name(&self) -> Result<String> {
        let key = "name";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_scope(&self) -> Result<isize> {
        let key = "id";
        match self.get(key) {
            Some(v) => match v.as_i64() {
                Some(id) => Ok(id as isize),
                None => bail!("ID not found: {}", self),
            },
            None => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_source_location(&self, localizer: &Option<Localizer>) -> Option<Loc> {
        let key = "src";
        match (self.get(key), localizer) {
            (Some(Value::String(s)), Some(localizer)) => parse_loc(localizer, s),
            _ => None,
        }
    }
}
