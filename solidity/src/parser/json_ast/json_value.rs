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
    /// Check whether the JSON AST node is an abstract node.
    fn is_abstract(&self) -> Result<bool>;

    /// Check whether the JSON AST node is declared anonymous.
    fn is_anonymous(&self) -> Result<bool>;

    /// Check whether the JSON AST node is a constant node.
    fn is_constant(&self) -> Result<bool>;

    /// Check whether the JSON AST node is a constructor node.
    fn is_constructor(&self) -> Result<bool>;

    /// Check whether the JSON AST node is fully implemented.
    fn is_fully_implemented(&self) -> Result<bool>;

    /// Get value of the `global` property.
    fn is_global(&self) -> Result<bool>;

    /// Get value of the `isInlineArray` property.
    fn is_inline_array(&self) -> Result<bool>;

    /// Get value of the `isLibrary` property.
    fn is_library(&self) -> Result<bool>;

    /// Check whether the JSON AST node is a state variable.
    fn is_state_variable(&self) -> Result<bool>;

    /// Check whether the JSON AST node is a virtual node.
    fn is_virtual(&self) -> Result<bool>;

    /// Get value of the `absolutePath` property.
    fn get_absolute_path(&self) -> Result<String>;

    /// Get value of the `arguments` property.
    fn get_arguments(&self) -> Result<&Value>;

    /// Get value of the `AST` property.
    fn get_ast(&self) -> Result<&Value>;

    /// Get value of the `ATTRIBUTES` property.
    fn get_attributes(&self) -> Result<&Value>;

    /// Get value of the `baseContracts` property.
    fn get_base_contracts(&self) -> Result<&Value>;

    /// Get value of the `baseExpression` property.
    fn get_base_expression(&self) -> Result<&Value>;

    /// Get value of the `baseName` property.
    fn get_base_name(&self) -> Result<&Value>;

    /// Get value of the `baseType` property.
    fn get_base_type(&self) -> Result<&Value>;

    /// Get value of the `block` property.
    fn get_block(&self) -> Result<&Value>;

    /// Get value of the `body` property.
    fn get_body(&self) -> Result<&Value>;

    /// Get value of the `canonicalName` property.
    fn get_canonical_name(&self) -> Result<String>;

    /// Get value of the `cases` property.
    fn get_cases(&self) -> Result<&Value>;

    /// Get value of the `children` property.
    fn get_children(&self) -> Result<&Value>;

    /// Get value of the `clauses` property.
    fn get_clauses(&self) -> Result<&Value>;

    /// Get value of the `commonType` property.
    fn get_common_type(&self) -> Result<&Value>;

    /// Get value of the `components` property.
    fn get_components(&self) -> Result<&Value>;

    /// Get value of the `condition` property.
    fn get_condition(&self) -> Result<&Value>;

    /// Get value of the `contractKind` property.
    fn get_contract_kind(&self) -> Result<ContractKind>;

    /// Get value of the `declarations` property.
    fn get_declarations(&self) -> Result<&Value>;

    /// Get value of the `definition` property.
    fn get_definition(&self) -> Result<&Value>;

    /// Get value of the `endExpression` property.
    fn get_end_expression(&self) -> Result<&Value>;

    /// Get value of the `errorCall` property.
    fn get_error_call(&self) -> Result<&Value>;

    /// Get value of the `errorName` property.
    fn get_error_name(&self) -> Result<String>;

    /// Get value of the `externalCall` property.
    fn get_external_call(&self) -> Result<&Value>;

    /// Get value of the `eventCall` property.
    fn get_event_call(&self) -> Result<&Value>;

    /// Get value of the `expression` property.
    fn get_expression(&self) -> Result<&Value>;

    /// Get value of the `falseBody` property.
    fn get_false_body(&self) -> Result<&Value>;

    /// Get value of the `falseExpression` property.
    fn get_false_expression(&self) -> Result<&Value>;

    /// Get value of the `foreign` property.
    fn get_foreign(&self) -> Result<&Value>;

    /// Get value of the `file` property.
    fn get_file(&self) -> Result<String>;

    /// Get value of the `function` property.
    fn get_function(&self) -> Result<&Value>;

    /// Get value of the `functionList` property.
    fn get_function_list(&self) -> Result<&Value>;

    /// Get value of the `functionName` property.
    fn get_function_name(&self) -> Result<&Value>;

    /// Get value of the `hexValue` property.
    fn get_hex_value(&self) -> Result<String>;

    /// Get value of the `id` property.
    fn get_id(&self) -> Result<isize>;

    /// Get value of the `indexed` property.
    fn get_indexed(&self) -> Result<bool>;

    /// Get value of the `indexExpression` property.
    fn get_index_expression(&self) -> Result<&Value>;

    /// Get value of the `initialValue` property.
    fn get_initial_value(&self) -> Result<&Value>;

    /// Get value of the `initializationExpression` property.
    fn get_initialization_expression(&self) -> Result<&Value>;

    /// Get value of the `keyType` property.
    fn get_key_type(&self) -> Result<&Value>;

    /// Get value of the `kind` property.
    fn get_kind(&self) -> Result<String>;

    /// Get value of the `literals` property.
    fn get_literals(&self) -> Result<&Value>;

    /// Get value of the `local` property.
    fn get_local(&self) -> Result<String>;

    /// Get value of the `loopExpression` property.
    fn get_loop_expression(&self) -> Result<&Value>;

    /// Get value of the `libraryName` property.
    fn get_library_name(&self) -> Result<&Value>;

    /// Get value of the `memberName` property.
    fn get_member_name(&self) -> Result<String>;

    /// Get value of the `members` property.
    fn get_members(&self) -> Result<&Value>;

    /// Get value of the `modifiers` property.
    fn get_modifiers(&self) -> Result<&Value>;

    /// Get value of the `modifierName` property.
    fn get_modifier_name(&self) -> Result<&Value>;

    /// Get value of the `mutability` property.
    fn get_mutability(&self) -> Result<VarMut>;

    /// Get value of the `name` property.
    fn get_name(&self) -> Result<String>;

    /// Get value of the `nameLocations` property.
    fn get_name_locs(&self, localizer: &Option<Localizer>) -> Vec<Option<Loc>>;

    /// Get value of the `nameLocations` property.
    fn get_names(&self) -> Result<Vec<String>>;

    /// Get value of the `nodeType` property.
    fn get_node_type(&self) -> Result<NodeType>;

    /// Get value of the `operator` property.
    fn get_operator(&self) -> Result<String>;

    /// Get value of the `operations` property.
    fn get_operations(&self) -> Result<String>;

    /// Get value of the `options` property.
    fn get_options(&self) -> Result<&Value>;

    /// Get value of the `overrides` property.
    fn get_overrides(&self) -> Result<&Value>;

    /// Get value of the `parameters` property.
    fn get_parameters(&self) -> Result<&Value>;

    /// Get value of the `parameterTypes` property.
    fn get_parameter_types(&self) -> Result<&Value>;

    /// Get value of the `pathNode` property.
    fn get_path_node(&self) -> Result<&Value>;

    /// Get value of the `post` property.
    fn get_post(&self) -> Result<&Value>;

    /// Get value of the `pre` property.
    fn get_pre(&self) -> Result<&Value>;

    /// Get value of the `referencedDeclaration` property.
    fn get_referenced_declaration(&self) -> Result<isize>;

    /// Get value of the `returnParameters` property.
    fn get_return_parameters(&self) -> Result<&Value>;

    /// Get value of the `returnParameterTypes` property.
    fn get_return_parameter_types(&self) -> Result<&Value>;

    /// Get value of the `returnVariables` property.
    fn get_return_variables(&self) -> Result<&Value>;

    /// Get value of the `scope` property.
    fn get_scope(&self) -> Result<isize>;

    /// Get value of the `sourceList` property.
    fn get_source_list(&self) -> Result<Vec<String>>;

    /// Get location in source code.
    ///
    /// A source location is of the following form with inclusive range:
    /// (`start_line`:`start_column` -> `end_line`:`end_column`).
    fn get_source_location(&self, localizer: &Option<Localizer>) -> Option<Loc>;

    /// Get value of the `sources`.
    fn get_sources(&self) -> Result<&Value>;

    /// Get value of the `state_mutability` property.
    ///
    /// This property often applies to function definitions.
    fn get_state_mutability(&self) -> Result<FuncMut>;

    /// Get value of the `statements` property.
    fn get_statements(&self) -> Result<&Value>;

    /// Get value of the `startExpression` property.
    fn get_start_expression(&self) -> Result<&Value>;

    /// Get value of the `storageLocation` property.
    fn get_storage_location(&self) -> Result<DataLoc>;

    /// Get value of the `subdenomination` property.
    fn get_sub_denomination(&self) -> Result<String>;

    /// Get value of the `symbolAliases` property.
    fn get_symbol_aliases(&self) -> Result<&Value>;

    /// Get value of the `trueBody` property.
    fn get_true_body(&self) -> Result<&Value>;

    /// Get value of the `trueExpression` property.
    fn get_true_expression(&self) -> Result<&Value>;

    /// Get value of the `type` property.
    fn get_type(&self) -> Result<String>;

    /// Get value of the `typeDescriptions` property.
    fn get_type_descriptions(&self) -> Result<&Value>;

    /// Get value of the `typeName` property.
    fn get_type_name(&self) -> Result<&Value>;

    /// Get value of the `typeDescriptions` property.
    fn get_type_string(&self) -> Result<&str>;

    /// Get value of the `unitAlias` property.
    fn get_unit_alias(&self) -> Result<String>;

    /// Get value of the `underlyingType` property.
    fn get_underlying_type(&self) -> Result<&Value>;

    /// Get value of the `value` property.
    fn get_value(&self) -> Result<&Value>;

    /// Get value of the `valueType` property.
    fn get_value_type(&self) -> Result<&Value>;
}

impl ValueUtil for Value {
    fn is_abstract(&self) -> Result<bool> {
        let key = "abstract";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_anonymous(&self) -> Result<bool> {
        let key = "anonymous";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_constant(&self) -> Result<bool> {
        let key = "constant";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_constructor(&self) -> Result<bool> {
        let key = "isConstructor";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_fully_implemented(&self) -> Result<bool> {
        let key = "fullyImplemented";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_global(&self) -> Result<bool> {
        let key = "global";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_inline_array(&self) -> Result<bool> {
        let key = "isInlineArray";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_library(&self) -> Result<bool> {
        let key = "isLibrary";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_state_variable(&self) -> Result<bool> {
        let key = "stateVariable";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn is_virtual(&self) -> Result<bool> {
        let key = "virtual";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_absolute_path(&self) -> Result<String> {
        let key = "absolutePath";
        match self.get(key) {
            Some(Value::String(s)) => Ok(s.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_arguments(&self) -> Result<&Value> {
        let key = "arguments";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_ast(&self) -> Result<&Value> {
        let key = "AST";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_attributes(&self) -> Result<&Value> {
        let key = "attributes";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_base_contracts(&self) -> Result<&Value> {
        let key = "baseContracts";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_base_expression(&self) -> Result<&Value> {
        let key = "baseExpression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_base_name(&self) -> Result<&Value> {
        let key = "baseName";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_base_type(&self) -> Result<&Value> {
        let key = "baseType";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

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

    fn get_canonical_name(&self) -> Result<String> {
        let key = "canonicalName";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_cases(&self) -> Result<&Value> {
        let key = "cases";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_children(&self) -> Result<&Value> {
        let key = "children";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_clauses(&self) -> Result<&Value> {
        let key = "clauses";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_common_type(&self) -> Result<&Value> {
        let key = "commonType";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_components(&self) -> Result<&Value> {
        let key = "components";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_condition(&self) -> Result<&Value> {
        let key = "condition";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_contract_kind(&self) -> Result<ContractKind> {
        let key = "contractKind";
        match self.get(key) {
            Some(Value::String(s)) => ContractKind::from_string(s),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_declarations(&self) -> Result<&Value> {
        let key = "declarations";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_definition(&self) -> Result<&Value> {
        let key = "definition";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_external_call(&self) -> Result<&Value> {
        let key = "externalCall";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_error_call(&self) -> Result<&Value> {
        let key = "errorCall";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_end_expression(&self) -> Result<&Value> {
        let key = "endExpression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_error_name(&self) -> Result<String> {
        let key = "errorName";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_event_call(&self) -> Result<&Value> {
        let key = "eventCall";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_expression(&self) -> Result<&Value> {
        let key = "expression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_false_body(&self) -> Result<&Value> {
        let key = "falseBody";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_false_expression(&self) -> Result<&Value> {
        let key = "falseExpression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_foreign(&self) -> Result<&Value> {
        let key = "foreign";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_file(&self) -> Result<String> {
        let key = "file";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_function(&self) -> Result<&Value> {
        let key = "function";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_function_list(&self) -> Result<&Value> {
        let key = "functionList";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_function_name(&self) -> Result<&Value> {
        let key = "functionName";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_hex_value(&self) -> Result<String> {
        let key = "hexValue";
        match self.get(key) {
            Some(Value::String(val)) => Ok(val.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_id(&self) -> Result<isize> {
        let key = "id";
        match self.get(key) {
            Some(v) => match v.as_i64() {
                Some(id) => Ok(id as isize),
                None => bail!("ID not found: {self}"),
            },
            None => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_indexed(&self) -> Result<bool> {
        let key = "indexed";
        match self.get(key) {
            Some(Value::Bool(b)) => Ok(*b),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_index_expression(&self) -> Result<&Value> {
        let key = "indexExpression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_initial_value(&self) -> Result<&Value> {
        let key = "initialValue";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_initialization_expression(&self) -> Result<&Value> {
        let key = "initializationExpression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_key_type(&self) -> Result<&Value> {
        let key = "keyType";
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

    fn get_loop_expression(&self) -> Result<&Value> {
        let key = "loopExpression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_library_name(&self) -> Result<&Value> {
        let key = "libraryName";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_literals(&self) -> Result<&Value> {
        let key = "literals";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_local(&self) -> Result<String> {
        let key = "local";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_member_name(&self) -> Result<String> {
        let key = "memberName";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_members(&self) -> Result<&Value> {
        let key = "members";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_modifiers(&self) -> Result<&Value> {
        let key = "modifiers";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_modifier_name(&self) -> Result<&Value> {
        let key = "modifierName";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_mutability(&self) -> Result<VarMut> {
        let key = "mutability";
        match self.get(key) {
            Some(Value::String(mutability)) => VarMut::new(mutability),
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

    fn get_name_locs(&self, localizer: &Option<Localizer>) -> Vec<Option<Loc>> {
        let key = "nameLocations";
        match self.get(key) {
            Some(Value::Array(loc_nodes)) => {
                let mut locs = vec![];
                for node in loc_nodes.iter() {
                    match (node, localizer) {
                        (Value::String(s), Some(l)) => locs.push(parse_loc(l, s)),
                        _ => locs.push(None),
                    }
                }
                locs
            }
            // In older versions, there can be no `nameLocations` property
            _ => vec![],
        }
    }

    fn get_names(&self) -> Result<Vec<String>> {
        let key = "names";
        match self.get(key) {
            Some(Value::Array(nodes)) => {
                let mut names = vec![];
                for node in nodes.iter() {
                    match node.as_str() {
                        Some(name) => names.push(name.to_string()),
                        None => bail!("Invalid name: {}", node),
                    }
                }
                Ok(names)
            }
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_node_type(&self) -> Result<NodeType> {
        let key = "nodeType";
        self.get(key)
            .and_then(|v| v.as_str())
            .map(NodeType::new)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_operator(&self) -> Result<String> {
        let key = "operator";
        match self.get(key) {
            Some(Value::String(s)) => Ok(s.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_operations(&self) -> Result<String> {
        let key = "operations";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_options(&self) -> Result<&Value> {
        let key = "options";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_overrides(&self) -> Result<&Value> {
        let key = "overrides";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_parameters(&self) -> Result<&Value> {
        let key = "parameters";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_parameter_types(&self) -> Result<&Value> {
        let key = "parameterTypes";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_path_node(&self) -> Result<&Value> {
        let key = "pathNode";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_post(&self) -> Result<&Value> {
        let key = "post";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_pre(&self) -> Result<&Value> {
        let key = "pre";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_referenced_declaration(&self) -> Result<isize> {
        let key = "referencedDeclaration";
        match self.get(key) {
            Some(v) => match v.as_i64() {
                Some(id) => Ok(id as isize),
                None => {
                    bail!("Referenced declaration not found: {}", self)
                }
            },
            None => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_return_parameters(&self) -> Result<&Value> {
        let key = "returnParameters";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_return_parameter_types(&self) -> Result<&Value> {
        let key = "returnParameterTypes";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_return_variables(&self) -> Result<&Value> {
        let key = "returnVariables";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
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

    fn get_source_list(&self) -> Result<Vec<String>> {
        let key = "sourceList";
        match self.get(key) {
            Some(Value::Array(nodes)) => {
                let mut source_names = vec![];
                for node in nodes {
                    match node.as_str() {
                        Some(name) => source_names.push(name.to_string()),
                        None => {
                            bail!("Invalid source name: {}", node)
                        }
                    }
                }
                Ok(source_names)
            }
            Some(_) => bail!("Invalid source name list: {}", self),
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

    fn get_sources(&self) -> Result<&Value> {
        let key = "sources";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_state_mutability(&self) -> Result<FuncMut> {
        let key = "stateMutability";
        match self.get(key) {
            Some(Value::String(mutability)) => FuncMut::new(mutability),
            _ => Ok(FuncMut::None),
        }
    }

    fn get_statements(&self) -> Result<&Value> {
        let key = "statements";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_start_expression(&self) -> Result<&Value> {
        let key = "startExpression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_storage_location(&self) -> Result<DataLoc> {
        let key = "storageLocation";
        match self.get(key) {
            Some(Value::String(data_loc)) => DataLoc::new(data_loc),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_sub_denomination(&self) -> Result<String> {
        let key = "subdenomination";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_symbol_aliases(&self) -> Result<&Value> {
        let key = "symbolAliases";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_true_body(&self) -> Result<&Value> {
        let key = "trueBody";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_true_expression(&self) -> Result<&Value> {
        let key = "trueExpression";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_type(&self) -> Result<String> {
        let key = "type";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_type_descriptions(&self) -> Result<&Value> {
        let key = "typeDescriptions";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_type_name(&self) -> Result<&Value> {
        let key = "typeName";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_type_string(&self) -> Result<&str> {
        let key = "typeString";
        self.get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_unit_alias(&self) -> Result<String> {
        let key = "unitAlias";
        match self.get(key) {
            Some(Value::String(name)) => Ok(name.clone()),
            _ => bail!("Failed to get `{key}` property: {self}"),
        }
    }

    fn get_underlying_type(&self) -> Result<&Value> {
        let key = "underlyingType";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_value(&self) -> Result<&Value> {
        let key = "value";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

    fn get_value_type(&self) -> Result<&Value> {
        let key = "valueType";
        self.get(key)
            .ok_or_else(|| eyre!("Failed to get `{key}` property: {self}"))
    }

}
