//! Parser that parses Solidity AST in JSON format and produces an AST.

use crate::{
    ast::*,
    parser::{
        json_ast::json_value::{NodeType, ValueUtil},
        localizer::Localizer,
        typ::type_parser,
    },
    tool::solc::{self, JsonAst},
};
use color_eyre::eyre::{Result, bail, eyre};
use core::file::{save_to_temporary_file, save_to_temporary_files};
use itertools::izip;
use num_bigint::BigInt;
use regex::Regex;
use rust_decimal::Decimal;
use serde::Value;
use std::{ops::Deref, path::Path, str::FromStr};
use yul::{
    ast::{self as yast, Block as YBlock, IntType as YIntType, Type as YType},
    parsing::parser as YulParser,
};

//------------------------------------------------------------------
// Data structure representing JSON AST Parser
//------------------------------------------------------------------

struct AstParser {
    pub solidity_json: Option<String>,
    pub input_file: Option<String>,
    pub base_path: Option<String>,
    pub localizer: Option<Localizer>,
}

//------------------------------------------------------------------
// Implementation for AST Parser
//------------------------------------------------------------------

impl AstParser {
    pub fn new(solidity_json: &JsonAst) -> Self {
        AstParser {
            solidity_json: Some(solidity_json.json_data.clone()),
            input_file: solidity_json.file_name.clone(),
            base_path: solidity_json.base_path.clone(),
            localizer: None,
        }
    }

    pub fn parse_solidity_json(&mut self) -> Result<Vec<SourceUnit>> {
        let node: Value = match &self.solidity_json {
            Some(content) => serde::from_str(content)?,
            None => bail!("Input JSON AST not found!"),
        };

        let sources_node = node.get_sources()?;
        let source_names = node.get_source_list()?;

        let mut source_units = vec![];
        for source_name in &source_names {
            let source_node = match sources_node.get(source_name) {
                Some(source_node) => source_node,
                None => bail!("Failed to get source node of: {}", source_name),
            };
            let ast_node = source_node.get_ast()?;
            source_units.push(self.parse_ast(ast_node)?)
        }

        Ok(source_units)
    }

    //-------------------------------------------------
    // Common utilities to handle AST nodes
    //-------------------------------------------------

    /// Get node type of an AST Node
    fn get_node_type(&self, node: &Value) -> Result<NodeType> {
        node.get_node_type()
    }

    /// Get the nth child of an AST Node
    fn get_child_node<'a>(&self, node: &'a Value, i: usize) -> Result<&'a Value> {
        match node.get_children() {
            Ok(Value::Array(nodes)) if i < nodes.len() => Ok(&nodes[i]),
            _ => bail!("Failed to get child node: {}\n{}", i, node),
        }
    }

    /// Get the nth child of an AST Node
    fn get_children_nodes<'a>(&self, node: &'a Value) -> Result<&'a Vec<Value>> {
        match node.get_children() {
            Ok(Value::Array(nodes)) => Ok(nodes),
            _ => bail!("Failed to get children nodes: {}", node),
        }
    }

    /// Get value of the `parameters` property in an AST node.
    fn get_parameters_node<'a>(&self, node: &'a Value) -> Result<&'a Value> {
        node.get_parameters()
    }

    /// Get value of the `literals` property in an AST node.
    fn get_literals<'a>(&self, node: &'a Value) -> Result<&'a Value> {
        node.get_literals()
    }

    /// Get value of the `operator` property in an AST node.
    fn get_operator(&self, node: &Value) -> Result<String> {
        node.get_operator()
    }

    /// Get value of the `value` property in an AST node.
    fn get_value<'a>(&self, node: &'a Value) -> Result<&'a Value> {
        node.get_value()
    }

    /// Get value of the `hexValue` property in an AST node.
    fn get_hex_value(&self, node: &Value) -> Result<String> {
        node.get_hex_value()
    }

    //-------------------------------------------------
    // Combined JSON
    //-------------------------------------------------

    /// Parse a source unit from a JSON AST node.
    fn parse_ast(&mut self, node: &Value) -> Result<SourceUnit> {
        let node_type = self.get_node_type(node)?;
        match node_type {
            NodeType::SourceUnit => self.parse_source_unit(node),
            _ => bail!("Source unit not found: {}", node),
        }
    }

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    /// Parse a source unit from a JSON AST node.
    fn parse_source_unit(&mut self, node: &Value) -> Result<SourceUnit> {
        let id = node.get_id().ok();
        let path = self.parse_source_unit_path(node)?;
        self.localizer = Localizer::new(path.to_string());
        let elems = node
            .get("nodes")
            .ok_or_else(|| eyre!("Source unit elements not found: {}", node))?
            .as_array()
            .ok_or_else(|| eyre!("Source unit elements invalid: {}", node))?
            .iter()
            .map(|elem_node| self.parse_source_unit_element(elem_node))
            .collect::<Result<Vec<SourceUnitElem>>>()?;
        Ok(SourceUnit::new(id, path, elems))
    }

    /// Parse source unit file path.
    ///
    /// Input AST node must be a node representing a source unit.
    fn parse_source_unit_path(&mut self, node: &Value) -> Result<String> {
        assert!(matches!(self.get_node_type(node), Ok(NodeType::SourceUnit)));
        let source_file = match node.get_absolute_path() {
            Ok(file_path) => file_path,
            Err(err) => match &self.input_file {
                Some(file_path) => file_path.clone(),
                None => bail!(err),
            },
        };
        let dir_path = match &self.base_path {
            Some(dir) => Some(Path::new(dir)),
            None => match &self.input_file {
                Some(file) => Path::new(file).parent(),
                _ => None,
            },
        };
        let path = match dir_path {
            None => source_file,
            Some(dir) => {
                let path = dir
                    .join(&source_file)
                    .as_os_str()
                    .to_os_string()
                    .into_string();
                match path {
                    Ok(source_path) => source_path,
                    Err(_) => source_file,
                }
            }
        };
        Ok(path)
    }

    /// Parse source unit element from a JSON AST node.
    fn parse_source_unit_element(&mut self, node: &Value) -> Result<SourceUnitElem> {
        use NodeType::*;
        match self.get_node_type(node)? {
            Pragma => self.parse_pragma(node).map(SourceUnitElem::from),
            Import => self.parse_import(node).map(SourceUnitElem::from),
            Using => self.parse_using(node).map(SourceUnitElem::from),
            ErrorDef => self.parse_error_def(node).map(SourceUnitElem::from),
            StructDef => self.parse_struct_def(node).map(SourceUnitElem::from),
            FuncDef => self
                .parse_function_definition(node)
                .map(SourceUnitElem::from),
            UserDefinedValueTypeDef => self
                .parse_user_defined_value_type_def(node)
                .map(SourceUnitElem::from),
            EnumDef => self.parse_enum_def(node).map(SourceUnitElem::from),
            ContractDef => self.parse_contract_def(node).map(SourceUnitElem::from),
            VarDecl => self.parse_var_decl(node).map(SourceUnitElem::from),
            _ => bail!("Failed to parse source element: {}", node),
        }
    }

    //-------------------------------------------------
    // Pragma directives.
    //-------------------------------------------------

    /// Parse pragma directive from a JSON AST node.
    fn parse_pragma(&mut self, node: &Value) -> Result<PragmaDir> {
        let id = node.get_id().ok();
        let pragma_lits = match self.get_literals(node)? {
            Value::String(s) => vec![s.clone()],
            Value::Array(arr) => arr
                .iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect::<Vec<String>>(),
            _ => {
                bail!("Pragma literals not found!")
            }
        };
        let kind = match pragma_lits.split_first() {
            Some((first, tail)) => match first.as_str() {
                "solidity" => {
                    let version = tail.join("");
                    PragmaKind::new_version(version)
                }
                "abicoder" => match tail.first() {
                    Some(s) => PragmaKind::new_abi_coder(s.to_string()),
                    None => bail!("Pragma abicoder not found!"),
                },
                "experimental" => match tail.first() {
                    Some(s) => PragmaKind::new_experimental(s.to_string()),
                    None => bail!("Pragma experimental not found!"),
                },
                _ => bail!("Pragma not supported: {}", first),
            },
            None => bail!("Pragma not found!"),
        };
        let loc = node.get_source_location(&self.localizer);
        Ok(PragmaDir::new(id, kind, loc))
    }

    //-------------------------------------------------
    // Import directives.
    //-------------------------------------------------

    /// Parse import directive from a JSON AST node.
    fn parse_import(&self, node: &Value) -> Result<ImportDir> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        let file_path = node.get_file()?;
        let abs_path = node.get_absolute_path()?;
        let mut symbol_aliases = vec![];
        match node.get_symbol_aliases()? {
            Value::Array(symbol_alias_nodes) => {
                for symbol_alias_node in symbol_alias_nodes {
                    symbol_aliases.push(self.parse_symbol_alias(symbol_alias_node)?);
                }
            }
            _ => bail!("Parse import directive: invalid symbol aliases: {}", node),
        }
        let unit_alias = node.get_unit_alias()?;
        let kind = match (&symbol_aliases[..], unit_alias.as_str()) {
            ([], "") => ImportSourceUnit::new(abs_path, file_path, None, loc).into(),
            (_, "") => ImportSymbols::new(abs_path, file_path, symbol_aliases, loc).into(),
            ([], _) => ImportSourceUnit::new(abs_path, file_path, Some(unit_alias), loc).into(),
            _ => bail!("TODO: parse both symbol and unit aliases: {}", node),
        };
        Ok(ImportDir::new(id, kind))
    }

    /// Parse `SymbolAlias` from a JSON AST node.
    fn parse_symbol_alias(&self, node: &Value) -> Result<ImportSymbol> {
        let foreign_node = node.get_foreign()?;
        let symbol = foreign_node.get_name()?;
        let local_node = node.get_local();
        let alias = match local_node {
            Ok(local_name) => Some(local_name),
            Err(_) => None,
        };
        let loc = node.get_source_location(&self.localizer);
        Ok(ImportSymbol::new(symbol, alias, loc))
    }

    //-------------------------------------------------
    // Using directives.
    //-------------------------------------------------

    /// Parse pragma directive from a JSON AST node.
    fn parse_using(&mut self, node: &Value) -> Result<UsingDir> {
        let id = node.get_id().ok();
        let global = node.is_global().unwrap_or(false);
        let loc = node.get_source_location(&self.localizer);
        let kind = match node.get_function_list() {
            Ok(Value::Array(func_nodes)) => {
                let mut ufuncs = vec![];
                for func_node in func_nodes {
                    if let Ok(op) = func_node.get_operator() {
                        let func_name = func_node.get_definition()?.get_name()?;
                        let ufunc = UsingFunc::new(&func_name, Some(op));
                        ufuncs.push(ufunc);
                    } else {
                        let func_name = func_node.get_function()?.get_name()?;
                        let ufunc = UsingFunc::new(&func_name, None);
                        ufuncs.push(ufunc);
                    }
                }
                UsingKind::UsingFunc(ufuncs)
            }
            Ok(node) => bail!("Implement parse_using: {}", node),
            Err(_) => {
                let lib_name = self.get_child_node(node, 0)?.get_name()?;
                let ulib = UsingLib::new(&lib_name);
                UsingKind::UsingLib(ulib)
            }
        };
        let typ = match node.get_type_name() {
            Ok(Value::Null) => None,
            Ok(type_node) => Some(self.parse_data_type(type_node)?),
            _ => None,
        };
        Ok(UsingDir::new(id, kind, typ, global, loc))
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    /// Parse contract definition from a JSON AST node.
    fn parse_contract_def(&mut self, node: &Value) -> Result<ContractDef> {
        let id = node.get_id().ok();
        let scope = node.get_scope().ok();
        let name = Name::new(node.get_name()?, None);
        let kind = self.parse_contract_kind(node)?;
        let abstract_ = self.is_contract_abstract(node)?;
        let bases = self.parse_base_contracts(node)?;
        let loc = node.get_source_location(&self.localizer);
        let elems: Vec<ContractElem> = node
            .get("nodes")
            .ok_or_else(|| eyre!("Contract elements not found: {}", node))?
            .as_array()
            .ok_or_else(|| eyre!("Contract elements invalid: {}", node))?
            .iter()
            .map(|v| self.parse_contract_element(v))
            .collect::<Result<Vec<ContractElem>>>()?;
        Ok(ContractDef::new(id, scope, name, kind, abstract_, bases, elems, loc))
    }

    /// Parse kind of a contract
    fn parse_contract_kind(&self, node: &Value) -> Result<ContractKind> {
        node.get_contract_kind()
    }

    /// Check whether the current AST Node represents an abstract contract.
    fn is_contract_abstract(&self, node: &Value) -> Result<bool> {
        match node.is_abstract() {
            Ok(b) => Ok(b),
            _ => match node.is_fully_implemented() {
                Ok(b) => Ok(!b),
                Err(_) => Ok(false),
            },
        }
    }

    /// Parse base contracts from a JSON AST node.
    fn parse_base_contracts(&mut self, node: &Value) -> Result<Vec<BaseContract>> {
        let mut base_contracts: Vec<BaseContract> = vec![];
        if let Ok(Value::Array(nodes)) = node.get_base_contracts() {
            for base_node in nodes.iter() {
                base_contracts.push(self.parse_base_contract(base_node)?);
            }
        }
        Ok(base_contracts)
    }

    /// Parse a base contract from a JSON AST node.
    fn parse_base_contract(&mut self, node: &Value) -> Result<BaseContract> {
        let contract_name: Name = node.get_base_name()?.get_name()?.into();
        let arguments = match node.get_arguments() {
            Ok(Value::Array(arg_nodes)) => {
                let mut args = vec![];
                for arg_node in arg_nodes {
                    args.push(self.parse_expr(arg_node)?);
                }
                args
            }
            Ok(arg) => match arg {
                Value::Null => vec![],
                _ => bail!("parse_base_contract err: {}", node),
            },
            Err(_) => vec![],
        };
        let loc = node.get_source_location(&self.localizer);
        Ok(BaseContract::new(contract_name, arguments, loc))
    }

    /// Parse a contract element from a JSON AST node.
    fn parse_contract_element(&mut self, node: &Value) -> Result<ContractElem> {
        let node_type = self.get_node_type(node)?;
        match node_type {
            NodeType::StructDef => self.parse_struct_def(node).map(|def| def.into()),
            NodeType::EventDef => self.parse_event_def(node).map(|def| def.into()),
            NodeType::ErrorDef => self.parse_error_def(node).map(|def| def.into()),
            NodeType::EnumDef => self.parse_enum_def(node).map(|def| def.into()),
            NodeType::VarDecl => self.parse_var_decl(node).map(|decl| decl.into()),
            NodeType::FuncDef => self.parse_function_definition(node).map(|def| def.into()),
            NodeType::ModifierDef => self.parse_modifier_def(node).map(|def| def.into()),
            NodeType::UserDefinedValueTypeDef => self
                .parse_user_defined_value_type_def(node)
                .map(|def| def.into()),
            NodeType::Using => self.parse_using(node).map(|dir| dir.into()),
            _ => bail!("Need to parse: {:?}", node_type),
        }
    }

    //-------------------------------------------------
    // Type name definition.
    //-------------------------------------------------

    /// Parse a type name definition from a JSON AST node.
    fn parse_user_defined_value_type_def(&mut self, node: &Value) -> Result<UserTypeDef> {
        let id = node.get_id().ok();
        let scope = node.get_scope().ok();
        let name: Name = node.get_name()?.into();
        let typ = self.parse_data_type(node.get_underlying_type()?)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(UserTypeDef::new(id, scope, name, typ, loc))
    }

    //-------------------------------------------------
    // Struct definition.
    //-------------------------------------------------

    /// Parse a struct definition from a JSON AST node.
    fn parse_struct_def(&mut self, node: &Value) -> Result<StructDef> {
        let id = node.get_id().ok();
        let scope = node.get_scope().ok();
        let name: Name = node.get_name()?.into();
        let mut fields: Vec<StructField> = vec![];
        for member_node in self.get_struct_field_nodes(node)? {
            fields.push(self.parse_struct_field(member_node)?)
        }
        let loc = node.get_source_location(&self.localizer);
        Ok(StructDef::new(id, scope, name, fields, loc))
    }

    /// Get AST nodes corresponding to fields of a struct definition.
    fn get_struct_field_nodes<'a>(&mut self, node: &'a Value) -> Result<&'a Vec<Value>> {
        match node.get_members() {
            Ok(Value::Array(nodes)) => Ok(nodes),
            Ok(_) => bail!("Failed to get struct member nodes: {}", node),
            Err(err) => bail!(err),
        }
    }

    /// Parse a struct field from a JSON AST node.
    fn parse_struct_field(&mut self, node: &Value) -> Result<StructField> {
        let id = node.get_id().ok();
        let name = node.get_name()?;
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(StructField::new(id, name, typ, loc))
    }

    //-------------------------------------------------
    // Enum definition.
    //-------------------------------------------------

    /// Parse an enum definition from a JSON AST node.
    fn parse_enum_def(&self, node: &Value) -> Result<EnumDef> {
        let id = node.get_id().ok();
        let scope = node.get_scope().ok();
        let name: Name = node.get_name()?.into();
        let loc = node.get_source_location(&self.localizer);
        let mut elems = vec![];
        match node.get_members() {
            Ok(Value::Array(member_nodes)) => {
                for member_node in member_nodes.iter() {
                    let member_name = member_node.get_name()?;
                    elems.push(member_name);
                }
            }
            Ok(node) => bail!("Implement parse_enum_definition: {}", node),
            Err(err) => bail!(err),
        }
        Ok(EnumDef::new(id, scope, name, elems, loc))
    }

    //-------------------------------------------------
    // Event definition.
    //-------------------------------------------------

    /// Parse an event definition from a JSON AST node.
    fn parse_event_def(&mut self, node: &Value) -> Result<EventDef> {
        let name: Name = node.get_name()?.into();
        let params_node = node.get_parameters()?;
        let params = self.parse_parameters(params_node)?;
        let anonymous = node.is_anonymous()?;
        let loc = node.get_source_location(&self.localizer);
        Ok(EventDef::new(name, anonymous, params, loc))
    }

    //-------------------------------------------------
    // Error definition.
    //-------------------------------------------------

    /// Parse an error definition from a JSON AST node.
    fn parse_error_def(&mut self, node: &Value) -> Result<ErrorDef> {
        let name: Name = node.get_name()?.into();
        let params = self.parse_parameters(self.get_parameters_node(node)?)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(ErrorDef::new(name, params, loc))
    }

    //-------------------------------------------------
    // Modifier definition.
    //-------------------------------------------------

    /// Parse a modifier definition from a JSON AST node.
    fn parse_modifier_def(&mut self, node: &Value) -> Result<FunctionDef> {
        let id = node.get_id().ok();
        let scope = node.get_scope().ok();
        let name: Name = node.get_name()?.into();
        let is_virtual = node.is_virtual().unwrap_or(false);
        let loc = node.get_source_location(&self.localizer);
        let overriding = self.parse_overriding(node)?;
        let body = self.parse_modifier_body(node);
        let params = self.parse_modifier_params(node)?;
        Ok(FunctionDef::new(
            id,
            scope,
            name,
            FuncKind::Modifier,
            body,
            is_virtual,
            FuncVis::None,
            FuncMut::None,
            params,
            vec![],
            overriding,
            vec![],
            loc,
            None,
        ))
    }

    /// Parser parameters of a modifier declaration.
    fn parse_modifier_params(&mut self, node: &Value) -> Result<Vec<VariableDecl>> {
        self.parse_parameters(node.get_parameters()?)
    }

    /// Parse body of a modifier
    fn parse_modifier_body(&mut self, node: &Value) -> Option<Block> {
        let body_node = match node.get_body() {
            Ok(body_node) => body_node,
            Err(_) => return None,
        };

        match self.parse_block(body_node, false) {
            Ok(block) => Some(block),
            Err(_) => None,
        }
    }

    //-------------------------------------------------
    // Function definition.
    //-------------------------------------------------

    /// Parse a function definition from a JSON AST node.
    fn parse_function_definition(&mut self, node: &Value) -> Result<FunctionDef> {
        let id = node.get_id().ok();
        let scope = node.get_scope().ok();
        let name = Name::new(node.get_name()?, None);
        let params = self.parse_function_parameters(node)?;
        let returns = self.parse_function_returns(node)?;
        let kind = match self.parse_function_kind(node) {
            Ok(kind) => kind,
            Err(_) => {
                if let Ok(true) = node.is_constructor() {
                    FuncKind::Constructor
                } else if name.is_empty() {
                    // Unnamed function is a fallback function in older Solidity
                    // <https://docs.soliditylang.org/en/v0.4.24/contracts.html#fallback-function>
                    FuncKind::Fallback
                } else {
                    FuncKind::ContractFunc
                }
            }
        };
        let is_virtual = node.is_virtual().unwrap_or(false);
        let fvis = self.parse_function_visibility(node)?;
        let fmut = node.get_state_mutability()?;
        let body = self.parse_function_body(node)?;
        let modifiers = self.parse_function_modifier_invocations(node)?;
        let overriding = self.parse_overriding(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(FunctionDef::new(
            id, scope, name, kind, body, is_virtual, fvis, fmut, params, modifiers, overriding,
            returns, loc, None,
        ))
    }

    /// Parse function kind information.
    ///
    /// The input JSON AST node should be a function definition node.
    fn parse_function_kind(&self, node: &Value) -> Result<FuncKind> {
        match node.get_kind() {
            Ok(kind) => Ok(FuncKind::new(&kind)),
            Err(err) => bail!(err),
        }
    }

    /// Parse function visibility information.
    ///
    /// The input JSON AST node should be a function definition node.
    fn parse_function_visibility(&self, node: &Value) -> Result<FuncVis> {
        match node.get_visibility() {
            Ok(visibility) => Ok(FuncVis::new(&visibility)),
            Err(_) => Ok(FuncVis::None),
        }
    }

    /// Parser parameters of a function declaration.
    fn parse_function_parameters(&mut self, node: &Value) -> Result<Vec<VariableDecl>> {
        let params_node = node.get_parameters()?;
        self.parse_parameters(params_node)
    }

    /// Parser return parameters of a function declaration.
    fn parse_function_returns(&mut self, node: &Value) -> Result<Vec<VariableDecl>> {
        let return_params_node = node.get_return_parameters()?;
        self.parse_parameters(return_params_node)
    }

    /// Parse body of a function definition.
    ///
    /// Return `Ok` if the function body if empty or is parsed successfully.
    ///
    /// Return `Err`  if it cannot be parsed.
    fn parse_function_body(&mut self, node: &Value) -> Result<Option<Block>> {
        let body_node = match node.get_body() {
            Ok(body_node) => body_node,
            Err(_) => return Ok(None),
        };

        match body_node {
            Value::Null => Ok(None),
            _ => match self.parse_block(body_node, false) {
                Ok(block) => Ok(Some(block)),
                Err(err) => bail!(err),
            },
        }
    }

    //-------------------------------------------------
    // Parameter list
    //-------------------------------------------------

    /// Parse parameters of a function, event, or error definition.
    fn parse_parameters(&mut self, node: &Value) -> Result<Vec<VariableDecl>> {
        if node.is_null() {
            return Ok(vec![]);
        }

        if !matches!(self.get_node_type(node), Ok(NodeType::ParameterList)) {
            bail!("Parameter list not found: {}", node);
        }

        let param_nodes = match self.get_parameters_node(node) {
            Ok(Value::Array(param_nodes)) => param_nodes,
            Ok(_) => bail!("Parameters not found: {}", node),
            Err(err) => bail!(err),
        };

        let mut params: Vec<VariableDecl> = vec![];
        for param_node in param_nodes.iter() {
            params.push(self.parse_var_decl(param_node)?);
        }

        Ok(params)
    }

    //-------------------------------------------------
    // Overriding specifiers.
    //-------------------------------------------------

    /// Parse an override specifier from a JSON AST node.
    fn parse_overriding(&self, node: &Value) -> Result<Overriding> {
        match node.get_overrides() {
            Ok(override_node) => {
                let override_nodes = match override_node.get_overrides() {
                    Ok(Value::Array(nodes)) => nodes,
                    Ok(v) => bail!("Override specifiers not found: {}", v),
                    Err(err) => bail!(err),
                };
                let mut contract_names: Vec<Name> = vec![];
                for node in override_nodes.iter() {
                    contract_names.push(node.get_name()?.into());
                }
                Ok(Overriding::Some(contract_names))
            }
            Err(_) => Ok(Overriding::None),
        }
    }

    //-------------------------------------------------
    // Function modifiers.
    //-------------------------------------------------

    /// Parse modifier invocations of a function definition.
    fn parse_function_modifier_invocations(&mut self, node: &Value) -> Result<Vec<CallExpr>> {
        let mut modifiers: Vec<CallExpr> = vec![];
        match node.get_modifiers() {
            Ok(Value::Array(m_nodes)) => {
                for m_node in m_nodes {
                    modifiers.push(self.parse_modifier_invocation(m_node)?);
                }
            }
            Ok(v) => bail!("Need to parse modifiers: {}", v),
            Err(err) => bail!(err),
        }
        Ok(modifiers)
    }

    /// Parse modifier invocations of a function from a JSON AST node.
    fn parse_modifier_invocation(&mut self, node: &Value) -> Result<CallExpr> {
        let id = node.get_id().ok();
        let name: Name = node.get_modifier_name()?.get_name()?.into();
        // Parse argument values
        let arg_nodes = match node.get_arguments() {
            Ok(Value::Null) => vec![],
            Ok(Value::Array(nodes)) => nodes.clone(),
            Ok(v) => vec![v.clone()],
            Err(_) => vec![],
        };
        let mut args: Vec<Expr> = vec![];
        for arg_node in arg_nodes {
            match self.parse_expr(&arg_node) {
                Ok(arg) => args.push(arg),
                Err(err) => bail!(err),
            }
        }

        let kind = match node.get_kind().as_deref() {
            Ok("modifierInvocation") => CallKind::ModifierInvoc,
            Ok("baseConstructorSpecifier") => CallKind::BaseConstructorCall,
            kind => bail!("Unknown modifier invocation Kind: {:?}!", kind),
        };

        let arg_typs: Vec<Type> = args.iter().map(|arg| arg.typ()).collect();
        let typ: Type = FunctionType::new(arg_typs, vec![], FuncVis::None, FuncMut::None).into();
        let loc = node.get_source_location(&self.localizer);
        let callee: Expr = Identifier::new(None, name, typ.clone(), loc).into();

        Ok(CallExpr::new_call_unnamed_args(id, callee, vec![], args, kind, typ, loc))
    }

    //-------------------------------------------------
    // Blocks
    //-------------------------------------------------

    /// Parse a block.
    fn parse_block(&mut self, node: &Value, is_unchecked: bool) -> Result<Block> {
        let id = node.get_id().ok();
        let statement_nodes = match node.get_statements()? {
            Value::Array(stmt_nodes) => stmt_nodes,
            _ => bail!("Failed to get statement nodes: {}", node),
        };

        let mut statements: Vec<Stmt> = vec![];
        for stmt_node in statement_nodes {
            statements.push(self.parse_stmt(stmt_node)?);
        }
        let loc = node.get_source_location(&self.localizer);
        Ok(Block::new(id, statements, is_unchecked, loc))
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    /// Parse a statement.
    fn parse_stmt(&mut self, node: &Value) -> Result<Stmt> {
        match self.get_node_type(node)? {
            NodeType::Block => self.parse_block(node, false).map(|blk| blk.into()),
            NodeType::UncheckedBlock => self.parse_block(node, true).map(|blk| blk.into()),
            NodeType::InlineAssembly => self.parse_inline_asm_stmt(node),
            NodeType::BreakStmt => self.parse_break_stmt(node),
            NodeType::ContinueStmt => self.parse_continue_stmt(node),
            NodeType::DoWhileStmt => self.parse_do_while_stmt(node),
            NodeType::ExprStmt => self.parse_expr_stmt(node),
            NodeType::EmitStmt => self.parse_emit_stmt(node),
            NodeType::ForStmt => self.parse_for_stmt(node),
            NodeType::IfStmt => self.parse_if_stmt(node),
            NodeType::ReturnStmt => self.parse_return_stmt(node),
            NodeType::PlaceholderStmt => self.parse_place_holder_stmt(node),
            NodeType::RevertStmt => self.parse_revert_stmt(node),
            NodeType::ThrowStmt => self.parse_throw_stmt(node),
            NodeType::TryStmt => self.parse_try_stmt(node),
            NodeType::VarDeclStmt => self.parse_var_decl_stmt(node),
            NodeType::WhileStmt => self.parse_while_stmt(node),
            _ => bail!("Parsing statement: {}", node),
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    /// Parse an inline assembly statement
    fn parse_inline_asm_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        let blk = match node.get_ast() {
            // Solidity 0.6 to 0.8
            Ok(ast_node) => self.parse_yul_block(ast_node)?,
            // Solidity 0.4, 0.5
            Err(_) => match node.get_operations() {
                Ok(operations) => YulParser::parse_inline_assembly_block(operations)?,
                _ => YBlock::new(vec![]),
            },
        };
        Ok(AsmStmt::new(id, false, vec![], blk.body, loc).into())
    }

    //-------------------------------------------------
    // Break statement
    //-------------------------------------------------

    /// Parse a `break` statement.
    fn parse_break_stmt(&self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        Ok(BreakStmt::new(id, loc).into())
    }

    //-------------------------------------------------
    // Throw statement in Solidity 0.4
    //-------------------------------------------------

    /// Parse a `throw` statement.
    fn parse_throw_stmt(&self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        Ok(ThrowStmt::new(id, loc).into())
    }

    //-------------------------------------------------
    // Continue statement
    //-------------------------------------------------

    /// Parse a `continue` statement.
    fn parse_continue_stmt(&self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        Ok(ContinueStmt::new(id, loc).into())
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    /// Parse an expression statement.
    fn parse_expr_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let expr_node = node.get_expression()?;
        let expr = self.parse_expr(expr_node)?;
        let loc = node.get_source_location(&self.localizer);
        // Refine to handle some built-in function calls
        if let Expr::Call(call) = &expr {
            let callee_name = &call.callee.to_string();
            if callee_name.eq("revert") {
                debug!("Converted to revert statement");
                return Ok(RevertStmt::new(id, None, call.args.clone(), loc).into());
            }
        }
        Ok(ExprStmt::new(id, expr, loc).into())
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    /// Parse an `if` statement.
    fn parse_if_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let condition_node = node.get_condition()?;
        let cond = self.parse_expr(condition_node)?;
        let true_body_node = node.get_true_body()?;
        let true_br = self.parse_stmt(true_body_node)?;
        let false_body_node = node.get_false_body().ok();
        let false_br = match false_body_node {
            Some(false_body_node) => self.parse_stmt(false_body_node).ok(),
            _ => None,
        };

        let loc = node.get_source_location(&self.localizer);
        Ok(IfStmt::new(id, cond, true_br, false_br, loc).into())
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    /// Parse a `for` statement.
    fn parse_for_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let pre_node = node.get_initialization_expression();
        let pre = match pre_node {
            Ok(Value::Null) => None,
            Ok(v) => self.parse_stmt(v).ok(),
            Err(_) => None,
        };
        let condition_node = node.get_condition();
        let cond = match condition_node {
            Ok(Value::Null) => None,
            Ok(v) => self.parse_expr(v).ok(),
            Err(_) => None,
        };
        let post_node = node.get_loop_expression();
        let post = match post_node {
            Ok(Value::Null) => None,
            Ok(v) => self.parse_stmt(v).ok(),
            Err(_) => None,
        };
        let body_node = node.get_body()?;
        let body = self.parse_stmt(body_node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(ForStmt::new(id, pre, cond, post, body, loc).into())
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    /// Parse a `while` statement.
    fn parse_while_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let condition_node = node.get_condition()?;
        let cond = self.parse_expr(condition_node)?;
        let body_node = node.get_body()?;
        let body = self.parse_stmt(body_node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(WhileStmt::new(id, cond, body, loc).into())
    }

    //-------------------------------------------------
    // Do-While statement
    //-------------------------------------------------

    /// Parse a `do_while` statement.
    fn parse_do_while_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let condition_node = node.get_condition()?;
        let cond = self.parse_expr(condition_node)?;
        let body_node = node.get_body()?;
        let body = self.parse_stmt(body_node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(DoWhileStmt::new(id, cond, body, loc).into())
    }

    //-------------------------------------------------
    // Place holder statement
    //-------------------------------------------------

    /// Parse a `place-holder` statement.
    fn parse_place_holder_stmt(&self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        Ok(PlaceholderStmt::new(id, loc).into())
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    /// Parse a `return` statement.
    fn parse_return_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let expr_node = node.get_expression();
        let expr = match expr_node {
            Ok(Value::Null) => None,
            Ok(expr_node) => Some(self.parse_expr(expr_node)?),
            Err(_) => None,
        };

        let loc = node.get_source_location(&self.localizer);
        Ok(ReturnStmt::new(id, expr, loc).into())
    }

    //-------------------------------------------------
    // Try-catch statement
    //-------------------------------------------------

    /// Parse a `try` statement.
    fn parse_try_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        let expr = self.parse_expr(node.get_external_call()?)?;

        let clause_nodes = match node.get_clauses() {
            Ok(Value::Array(clause_nodes)) => clause_nodes,
            _ => bail!("Implement parse_try_statement: {}", node),
        };
        match clause_nodes.split_first() {
            // `try` clause + `catch` clauses
            Some((try_cls_node, catch_clauses)) => {
                let try_blk = self.parse_block(try_cls_node.get_block()?, false)?;
                let params = match self.get_parameters_node(try_cls_node) {
                    Ok(params_node) => self.parse_parameters(params_node)?,
                    Err(_) => vec![],
                };
                let mut catch_cls = vec![];
                for cls in catch_clauses {
                    let clause = self.parse_catch_clause(cls)?;
                    catch_cls.push(clause);
                }
                Ok(TryStmt::new(id, expr, params, try_blk, catch_cls, loc).into())
            }
            None => bail!("Implement parse_try_statement: {}", node),
        }
    }

    /// Parse a `catch` clause in a `try` statement.
    fn parse_catch_clause(&mut self, node: &Value) -> Result<CatchClause> {
        let id = node.get_id().ok();
        let block = self.parse_block(node.get_block()?, false)?;
        let error = {
            let error_name = node.get_error_name()?;
            match error_name.is_empty() {
                true => None,
                false => Some(error_name),
            }
        };
        let params = match self.get_parameters_node(node) {
            Ok(params_node) => self.parse_parameters(params_node)?,
            Err(_) => vec![],
        };
        let loc = node.get_source_location(&self.localizer);
        Ok(CatchClause::new(id, error, params, block, loc))
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    /// Parse a `revert` statement.
    fn parse_revert_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        let error_call = self.parse_function_call(node.get_error_call()?)?;
        let error = error_call.callee.deref().clone();
        let args = error_call.args;
        Ok(RevertStmt::new(id, Some(error), args, loc).into())
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    /// Parse an `emit` statement.
    fn parse_emit_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let loc = node.get_source_location(&self.localizer);
        let event_call = self.parse_function_call(node.get_event_call()?)?;
        let event = event_call.callee.deref().clone();
        let args = event_call.args;
        Ok(EmitStmt::new(id, event, args, loc).into())
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    /// Parse a variable declaration statement.
    fn parse_var_decl_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = node.get_id().ok();
        let vdecls_node = node.get_declarations();
        let vdecl_nodes = match vdecls_node {
            Ok(Value::Array(nodes)) => nodes.clone(),
            Ok(v) => vec![v.clone()],
            Err(err) => bail!(err),
        };
        let mut vars: Vec<Option<VariableDecl>> = vec![];
        for vdecl_node in vdecl_nodes.iter() {
            match vdecl_node {
                Value::Null => vars.push(None),
                _ => match self.parse_var_decl(vdecl_node) {
                    Ok(vdecl) => vars.push(Some(vdecl)),
                    Err(err) => bail!(err),
                },
            }
        }
        let value_node = node.get_initial_value();
        let value = match value_node {
            Ok(Value::Null) => None,
            Ok(v) => {
                let expr = self.parse_expr(v)?;
                Some(expr)
            }
            Err(_) => None,
        };
        let loc = node.get_source_location(&self.localizer);

        Ok(VarDeclStmt::new(id, vars, value, loc).into())
    }

    //-------------------------------------------------
    // Expressions
    //-------------------------------------------------

    /// Parse an expression.
    fn parse_expr(&mut self, node: &Value) -> Result<Expr> {
        match self.get_node_type(node)? {
            NodeType::Literal => self.parse_lit(node).map(|lit| lit.into()),
            NodeType::Ident => self.parse_ident(node).map(|id| id.into()),
            NodeType::UnaryOperation => self.parse_unary_expr(node).map(Expr::from),
            NodeType::BinaryOperation => self.parse_binary_expr(node).map(Expr::from),
            NodeType::Assign => self.parse_assign_expr(node).map(Expr::from),
            NodeType::FuncCall => self.parse_function_call(node).map(Expr::from),
            NodeType::FuncCallOpts => self.parse_func_call_opts(node).map(Expr::from),
            NodeType::MemberAccess => self.parse_member_expr(node).map(Expr::from),
            NodeType::IndexAccess => self.parse_index_expr(node).map(Expr::from),
            NodeType::TupleExpr => match node.is_inline_array()? {
                true => self.parse_inline_array_expr(node).map(Expr::from),
                false => self.parse_tuple_expr(node).map(Expr::from),
            },
            NodeType::ElementaryTypeNameExpr => self.parse_type_name_expr(node).map(Expr::from),
            NodeType::NewExpr => self.parse_new_expr(node).map(Expr::from),
            NodeType::Conditional => self.parse_conditional_expr(node).map(Expr::from),
            NodeType::IndexRangeAccess => self.parse_slice_expr(node).map(Expr::from),
            _ => {
                bail!("Implement parse_expression: {}", node)
            }
        }
    }

    //-------------------------------------------------
    // Unary expressions
    //-------------------------------------------------

    /// Parse unary expression.
    fn parse_unary_expr(&mut self, node: &Value) -> Result<UnaryExpr> {
        let id = node.get_id().ok();
        let operand_node = node.get_sub_expression()?;
        let operand = self.parse_expr(operand_node)?;
        let is_prefix_op = node.get_prefix()?.as_bool().unwrap_or(false);
        let operator = UnaryOp::new(&self.get_operator(node)?, is_prefix_op)?;
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(UnaryExpr::new(id, operator, operand, typ, loc))
    }

    //-------------------------------------------------
    // Binary expressions
    //-------------------------------------------------

    /// Parse binary expression.
    fn parse_binary_expr(&mut self, node: &Value) -> Result<BinaryExpr> {
        let id = node.get_id().ok();

        let lhs_node = node.get_left_expression()?;
        let mut lhs = self.parse_expr(lhs_node)?;
        let rhs_node = node.get_right_expression()?;
        let mut rhs = self.parse_expr(rhs_node)?;
        if let Ok(common_type_node) = node.get_common_type() {
            let type_string = common_type_node.get_type_string()?;
            let common_type = type_parser::parse_data_type(type_string)?;
            lhs.update_data_type(common_type.clone());
            rhs.update_data_type(common_type);
        }
        let op = BinOp::new(&self.get_operator(node)?)?;
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(BinaryExpr::new(id, op, lhs, rhs, typ, loc))
    }

    //-------------------------------------------------
    // Assignment expressions
    //-------------------------------------------------

    /// Parse assignment expression.
    fn parse_assign_expr(&mut self, node: &Value) -> Result<AssignExpr> {
        let id = node.get_id().ok();
        let lhs_node = node.get_left_hand_side()?;
        let lhs = self.parse_expr(lhs_node)?;
        let rhs_node = node.get_right_hand_side()?;
        let rhs = self.parse_expr(rhs_node)?;
        let op = AssignOp::new(&self.get_operator(node)?)?;
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);

        Ok(AssignExpr::new(id, op, lhs, rhs, typ, loc))
    }

    //-------------------------------------------------
    // Call expressions
    //-------------------------------------------------

    /// Parse a call expression.
    fn parse_function_call(&mut self, node: &Value) -> Result<CallExpr> {
        let id = node.get_id().ok();

        let (callee, call_opts) = match self.parse_function_call_callee(node)? {
            Expr::CallOpts(exp) => (exp.callee.deref().clone(), exp.call_opts),
            exp => (exp, vec![]),
        };

        // Parse argument values, names, and name locations
        let (arg_values, arg_names, arg_locs) = self.parse_function_call_arguments(node)?;

        let kind = match node.get_kind()?.as_str() {
            "functionCall" => CallKind::FuncCall,
            "structConstructorCall" => CallKind::StructConstructorCall,
            "typeConversion" => CallKind::TypeConversionCall,
            kind => bail!("Unknown call kind: {}", kind),
        };

        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);

        // Return unnamed arguments
        if arg_names.is_empty() && arg_locs.is_empty() {
            return Ok(CallExpr::new_call_unnamed_args(
                id, callee, call_opts, arg_values, kind, typ, loc,
            ));
        }

        // Return named arguments
        assert!(
            arg_values.len() == arg_names.len(),
            "Invalid named arguments of function call: {}",
            node
        );

        let arg_locs = match arg_locs.is_empty() {
            true => vec![loc; arg_names.len()],
            false => arg_locs,
        };

        let mut args = vec![];
        for (value, name, aloc) in izip!(arg_values, arg_names, arg_locs) {
            let arg = NamedArg::new(name, value, aloc);
            args.push(arg)
        }

        Ok(CallExpr::new_call_named_args(id, callee, call_opts, args, kind, typ, loc))
    }

    /// Parse callee of a function call
    fn parse_function_call_callee(&mut self, node: &Value) -> Result<Expr> {
        let callee_node = node.get_expression()?;
        self.parse_expr(callee_node)
    }

    /// Parse arguments of a function call, which include argument values,
    /// names, and name locations.
    fn parse_function_call_arguments(
        &mut self,
        node: &Value,
    ) -> Result<(Vec<Expr>, Vec<String>, Vec<Option<Loc>>)> {
        // Parse argument values
        let arg_value_nodes = match node.get_arguments() {
            Ok(Value::Array(nodes)) => nodes.clone(),
            Ok(_) => {
                bail!("Need to parse function call arguments: {}", node)
            }
            Err(err) => bail!(err),
        };
        let mut arg_values: Vec<Expr> = vec![];
        for arg_node in arg_value_nodes {
            match self.parse_expr(&arg_node) {
                Ok(arg) => arg_values.push(arg),
                Err(err) => bail!(err),
            }
        }
        // Parse argument names and name locations
        let arg_names = node.get_names()?;
        let arg_name_locs = node.get_name_locs(&self.localizer);
        Ok((arg_values, arg_names, arg_name_locs))
    }

    //-------------------------------------------------
    // Function call options expressions
    //-------------------------------------------------

    /// Parse a `FunctionCallOptions` expression.
    fn parse_func_call_opts(&mut self, node: &Value) -> Result<CallOptsExpr> {
        let id = node.get_id().ok();
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        let callee = self.parse_expr(node.get_expression()?)?;
        let call_opt_names = node.get_names()?;
        let mut call_opt_values = vec![];
        match node.get_options() {
            Ok(Value::Array(opt_nodes)) => {
                for n in opt_nodes {
                    let expr = self.parse_expr(n)?;
                    let l = n.get_source_location(&self.localizer);
                    call_opt_values.push((expr, l));
                }
            }
            _ => {
                bail!("parse_function_call_options failed: {}", node)
            }
        }
        let call_opts = call_opt_names
            .iter()
            .zip(call_opt_values.iter())
            .map(|(name, (value, l))| CallOpt::new(name.to_string(), value.clone(), *l))
            .collect();
        Ok(CallOptsExpr::new(id, callee, call_opts, typ, loc))
    }

    //-------------------------------------------------
    // Member-access expression
    //-------------------------------------------------

    /// Parse a member access expression.
    fn parse_member_expr(&mut self, node: &Value) -> Result<MemberExpr> {
        let id = node.get_id().ok();
        let base_node = node.get_expression()?;
        let base = self.parse_expr(base_node)?;
        let member = Name::new(node.get_member_name()?, None);
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(MemberExpr::new(id, base, member, typ, loc))
    }

    //-------------------------------------------------
    // Index-access expression.
    //-------------------------------------------------

    /// Parse an index access expression.
    fn parse_index_expr(&mut self, node: &Value) -> Result<IndexExpr> {
        let id = node.get_id().ok();
        let base_node = node.get_base_expression()?;
        let base = self.parse_expr(base_node)?;
        let index_node = node.get_index_expression();
        let index = match index_node {
            Ok(Value::Null) => None,
            Ok(v) => {
                let index_expr = self.parse_expr(v)?;
                Some(index_expr)
            }
            Err(_) => None,
        };
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(IndexExpr::new(id, base, index, typ, loc))
    }

    //-------------------------------------------------
    // The slice expression
    //-------------------------------------------------

    /// Parse a slice expression
    fn parse_slice_expr(&mut self, node: &Value) -> Result<SliceExpr> {
        let id = node.get_id().ok();
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        let base = self.parse_expr(node.get_base_expression()?)?;
        let start_idx = match node.get_start_expression() {
            Ok(idx_node) => match idx_node {
                Value::Null => None,
                _ => self.parse_expr(idx_node).ok(),
            },
            Err(_) => None,
        };
        let end_idx = match node.get_end_expression() {
            Ok(idx_node) => match idx_node {
                Value::Null => None,
                _ => self.parse_expr(idx_node).ok(),
            },
            Err(_) => None,
        };
        Ok(SliceExpr::new(id, base, start_idx, end_idx, typ, loc))
    }

    //-------------------------------------------------
    // Tuple expression.
    //-------------------------------------------------

    /// Parse a tuple expression.
    fn parse_tuple_expr(&mut self, node: &Value) -> Result<TupleExpr> {
        let id = node.get_id().ok();
        let components_node = node.get_components()?;
        let mut elems = vec![];
        match components_node {
            Value::Array(nodes) => {
                for n in nodes {
                    match n {
                        Value::Null => elems.push(None),
                        _ => elems.push(self.parse_expr(n).ok()),
                    }
                }
            }
            _ => bail!("Implement parse_tuple_expression: {}", node),
        }
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(TupleExpr::new(id, elems, typ, loc))
    }

    //-------------------------------------------------
    // Inline array expression.
    //-------------------------------------------------

    /// Parse an inline array expression.
    fn parse_inline_array_expr(&mut self, node: &Value) -> Result<InlineArrayExpr> {
        let id = node.get_id().ok();
        let mut elems = vec![];
        match node.get_components()? {
            Value::Array(component_nodes) => {
                for component_node in component_nodes {
                    elems.push(self.parse_expr(component_node)?)
                }
            }
            _ => bail!("parse_inline_array_expression err: {}", node),
        }
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(InlineArrayExpr::new(id, elems, typ, loc))
    }

    //-------------------------------------------------
    // Elementary type name expression
    //-------------------------------------------------

    /// Parse an elementary type name expression.
    fn parse_type_name_expr(&mut self, node: &Value) -> Result<TypeNameExpr> {
        let id = node.get_id().ok();
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(TypeNameExpr::new(id, typ, loc))
    }

    //-------------------------------------------------
    // The `new` expression
    //-------------------------------------------------

    /// Parse a `new` expression.
    fn parse_new_expr(&mut self, node: &Value) -> Result<NewExpr> {
        let id = node.get_id().ok();
        let typ = self.parse_data_type(node.get_type_name()?)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(NewExpr::new(id, typ, loc))
    }

    //-------------------------------------------------
    // The conditional expression
    //-------------------------------------------------

    /// Parse a conditional expression.
    fn parse_conditional_expr(&mut self, node: &Value) -> Result<ConditionalExpr> {
        let id = node.get_id().ok();
        let cond = self.parse_expr(node.get_condition()?)?;
        let true_br = self.parse_expr(node.get_true_expression()?)?;
        let false_br = self.parse_expr(node.get_false_expression()?)?;
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(ConditionalExpr::new(id, cond, true_br, false_br, typ, loc))
    }

    //-------------------------------------------------
    // Parsing variable declaration.
    //-------------------------------------------------

    /// Parse a variable declaration from a JSON AST node.
    fn parse_var_decl(&mut self, node: &Value) -> Result<VariableDecl> {
        let id = node.get_id().ok();
        let scope = node.get_scope().ok();
        let name = Name::new(node.get_name()?, None);
        let value = match self.get_value(node) {
            Ok(value_node) => match value_node {
                Value::Null => None,
                _ => self.parse_expr(value_node).ok(),
            },
            Err(_) => None,
        };
        let mutability = match node.get_mutability() {
            Ok(mutability) => mutability,
            Err(_) => match self.parse_variable_constant_attribute(node)? {
                true => VarMut::Constant,
                false => VarMut::Mutable,
            },
        };
        let is_state_var = node.is_state_variable().unwrap_or(false);
        let visibility = self.parse_variable_visibility(node)?;
        let overriding = match node.get_overrides() {
            Ok(override_node) => match override_node {
                Value::Null => Overriding::None,
                _ => self.parse_overriding(node)?,
            },
            Err(_) => Overriding::None,
        };
        let data_loc = node.get_storage_location().ok();
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(VariableDecl::new(
            id,
            scope,
            name,
            typ,
            value,
            mutability,
            is_state_var,
            visibility,
            data_loc,
            overriding,
            loc,
        ))
    }

    /// Parse the constant attribute of a variable declaration.
    fn parse_variable_constant_attribute(&self, node: &Value) -> Result<bool> {
        node.is_constant()
    }

    /// Parse variable visibility information.
    ///
    /// The input JSON AST node should be a variable declaration node.
    fn parse_variable_visibility(&self, node: &Value) -> Result<VarVis> {
        match node.get_visibility() {
            Ok(vis) => Ok(VarVis::new(&vis)),
            Err(err) => bail!(err),
        }
    }

    //-------------------------------------------------
    // Parsing identifier.
    //-------------------------------------------------

    /// Parse an identifier from a JSON AST node.
    fn parse_ident(&mut self, node: &Value) -> Result<Identifier> {
        let id = node.get_id().ok();
        let name = Name::new(node.get_name()?, None);
        let typ = self.parse_data_type(node)?;
        let loc = node.get_source_location(&self.localizer);
        Ok(Identifier::new(id, name, typ, loc))
    }

    //-------------------------------------------------
    // Parsing literal expressions
    //-------------------------------------------------

    /// Parse literal expression.
    fn parse_lit(&mut self, node: &Value) -> Result<Lit> {
        let loc = node.get_source_location(&self.localizer);
        let typ = self.parse_data_type(node)?;
        let value_node = self.get_value(node);
        let hex_value_node = self.get_hex_value(node);

        match typ {
            Type::Bool => {
                let value = self.parse_bool_lit(value_node?)?;
                Ok(BoolLit::new(value, typ, loc).into())
            }

            Type::Int(_) => {
                let value_node = value_node?;
                let value = match self.parse_int_lit(value_node) {
                    Ok(num) => IntNum::new(num, typ).into(),
                    Err(_) => {
                        let hex = self.parse_string_lit(value_node)?;
                        HexNum::new(hex, typ).into()
                    }
                };
                let unit = node.get_sub_denomination().ok().map(NumUnit::new);
                Ok(NumLit::new(value, unit, loc).into())
            }

            Type::Fixed(_) => {
                let value = self.parse_rational_lit(value_node?)?;
                let number = FixedNum::new(value, typ).into();
                Ok(NumLit::new(number, None, loc).into())
            }

            Type::Address(_) => {
                let hex_value = self.parse_string_lit(value_node?)?;
                let value = HexNum::new(hex_value, typ).into();
                Ok(NumLit::new(value, None, loc).into())
            }

            Type::String(_) => match node.get_kind().as_deref() {
                Ok("hexString") => Ok(HexLit::new(hex_value_node?, typ, loc).into()),

                Ok("unicodeString") => match value_node? {
                    Value::String(value) => {
                        Ok(UnicodeLit::new(value.to_string(), typ, loc).into())
                    }
                    _ => bail!("Failed to parse unicode string: {}", node),
                },

                // REVIEW: why not parsing to normal string first?
                Ok("string") => match (value_node, hex_value_node) {
                    (Ok(value), _) => {
                        let value = self.parse_string_lit(value)?;
                        if value.is_ascii() {
                            Ok(StringLit::new(value, typ, loc).into())
                        } else {
                            Ok(UnicodeLit::new(value, typ, loc).into())
                        }
                    }
                    (_, Ok(hex)) => Ok(HexLit::new(hex, typ, loc).into()),
                    _ => bail!("Failed to parse string literal: {}", node),
                },

                Ok(kind) => bail!("Need to parse literal kind: {:?}", kind),

                Err(_) => bail!("Literal kind not found: {:?}", node),
            },

            _ => bail!("Need to parse literal type: {}", typ),
        }
    }

    /// Parse a bool literal.
    fn parse_bool_lit(&self, node: &Value) -> Result<bool> {
        match node.as_str() {
            Some(s) => s.parse::<bool>().map_err(|err| eyre!("{}", err)),
            None => bail!("Failed to parse bool literal: {}", node),
        }
    }

    /// Parse an integer literal to big integer.
    fn parse_int_lit(&self, node: &Value) -> Result<BigInt> {
        match node.as_str() {
            Some(s) => s.parse::<BigInt>().map_err(|err| eyre!("{}", err)),
            None => {
                bail!("Failed to parse integer literal: {}", node)
            }
        }
    }

    /// Parse a rational literal to a fixed-point number.
    fn parse_rational_lit(&self, node: &Value) -> Result<Decimal> {
        match node.as_str() {
            Some(s) => match Decimal::from_str(s) {
                Ok(n) => Ok(n),
                Err(_) => bail!("Failed to parse decimal: {}", s),
            },
            None => bail!("Failed to parse rational literal: {}", node),
        }
    }

    /// Parse a string literal and escape some special characters.
    fn parse_string_lit(&self, node: &Value) -> Result<String> {
        match node.as_str() {
            Some(s) => {
                // Escape special characters
                let nstr = s.replace('\\', "\\\\");
                let nstr = nstr.replace('\"', "\\\"");
                let nstr = nstr.replace('\'', "\\\'");
                let nstr = nstr.replace('\n', "\\n");
                let nstr = nstr.replace('\r', "\\r");
                let nstr = nstr.replace('\t', "\\t");

                // Escape control characters:
                // https://www.ascii-code.com/characters/control-characters
                let nstr = nstr.replace('\x00', "\\x00");
                let nstr = nstr.replace('\x01', "\\x01");
                let nstr = nstr.replace('\x02', "\\x02");
                let nstr = nstr.replace('\x03', "\\x03");
                let nstr = nstr.replace('\x04', "\\x04");
                let nstr = nstr.replace('\x05', "\\x05");
                let nstr = nstr.replace('\x06', "\\x06");
                let nstr = nstr.replace('\x07', "\\x07");
                let nstr = nstr.replace('\x08', "\\x08");
                let nstr = nstr.replace('\x09', "\\x09");
                let nstr = nstr.replace('\x0A', "\\x0A");
                let nstr = nstr.replace('\x0B', "\\x0B");
                let nstr = nstr.replace('\x0C', "\\x0C");
                let nstr = nstr.replace('\x0D', "\\x0D");
                let nstr = nstr.replace('\x0E', "\\x0E");
                let nstr = nstr.replace('\x0F', "\\x0F");
                let nstr = nstr.replace('\x10', "\\x10");
                let nstr = nstr.replace('\x11', "\\x11");
                let nstr = nstr.replace('\x12', "\\x12");
                let nstr = nstr.replace('\x13', "\\x13");
                let nstr = nstr.replace('\x14', "\\x14");
                let nstr = nstr.replace('\x15', "\\x15");
                let nstr = nstr.replace('\x16', "\\x16");
                let nstr = nstr.replace('\x17', "\\x17");
                let nstr = nstr.replace('\x18', "\\x18");
                let nstr = nstr.replace('\x19', "\\x19");
                let nstr = nstr.replace('\x1A', "\\x1A");
                let nstr = nstr.replace('\x1B', "\\x1B");
                let nstr = nstr.replace('\x1C', "\\x1C");
                let nstr = nstr.replace('\x1D', "\\x1D");
                let nstr = nstr.replace('\x1E', "\\x1E");
                let nstr = nstr.replace('\x1F', "\\x1F");
                let nstr = nstr.replace('\x7F', "\\x7F");

                // Return
                Ok(nstr)
            }
            None => bail!("Failed to parse string literal: {}", node),
        }
    }

    //-------------------------------------------------
    // Data types
    //-------------------------------------------------

    /// Get data type of a JSON AST node.
    fn parse_data_type(&mut self, node: &Value) -> Result<Type> {
        let data_loc = node.get_storage_location().ok();

        // First, parse data type from the `typeName` information.
        if let Ok(type_name_node) = node.get_type_name() {
            if let Ok(mut output_typ) = self.parse_type_name(type_name_node) {
                // Update data location if it is specified and not default
                if data_loc.is_some() {
                    output_typ.set_data_loc(data_loc);
                }
                return Ok(output_typ);
            }
        }

        // If not successful, parse data type from the `typeDescriptions` string.
        let type_description_node = node.get_type_descriptions()?;
        let type_string = type_description_node.get_type_string()?;
        let mut output_typ = type_parser::parse_data_type(type_string)?;

        // Update data location if it is specified and not default
        if data_loc.is_some() {
            output_typ.set_data_loc(data_loc);
        }
        Ok(output_typ)
    }

    //-------------------------------------------------
    // Type descriptions
    //-------------------------------------------------

    /// Parse type from type descriptions.
    fn parse_type_descriptions(&self, node: &Value) -> Result<Type> {
        match node.get_type_descriptions() {
            Ok(type_descriptions) => {
                let type_string = type_descriptions.get_type_string()?;
                let mut output_typ = type_parser::parse_data_type(type_string)?;
                // Update data location if it is specified and not default
                let data_loc = node.get_storage_location().ok();
                if data_loc.is_some() {
                    output_typ.set_data_loc(data_loc);
                }
                Ok(output_typ)
            }
            Err(err) => bail!(err),
        }
    }

    //-------------------------------------------------
    // Type name
    //-------------------------------------------------

    /// Get the type of a JSON AST node from the `typeName` field.
    fn parse_type_name(&mut self, node: &Value) -> Result<Type> {
        let output_typ = match self.get_node_type(node)? {
            NodeType::ArrayTypeName => self.parse_array_type_name(node),
            NodeType::FuncTypeName => self.parse_function_type_name(node),
            // NodeType::UserDefinedTypeName => self.parse_user_defined_type_name(node),
            _ => Ok(self.parse_type_descriptions(node)?),
        };
        match output_typ {
            Ok(mut typ) => {
                // Update type name and its scope when possible
                if let Ok(path_node) = node.get_path_node() {
                    if let Ok(name_path) = path_node.get_name() {
                        let names: Vec<Name> = name_path.split('.').map(Name::from).collect();
                        let (scope, typ_name) = match &names[..] {
                            [] => bail!("Parse type name: empty name!"),
                            [contract_name, name] => (Some(contract_name.clone()), name.clone()),
                            _ => bail!("TODO: parse type name: {}!", name_path),
                        };
                        typ.update_name(typ_name);
                        typ.update_scope(scope);
                    }
                }
                Ok(typ)
            }
            Err(err) => bail!(err),
        }
    }

    /// Parse a function type name
    fn parse_function_type_name(&mut self, node: &Value) -> Result<Type> {
        let fvis = self.parse_function_visibility(node)?;
        let fmut = node.get_state_mutability()?;

        let param_node = node.get_parameter_types()?;
        let mut params = vec![];
        match self.get_parameters_node(param_node) {
            Ok(Value::Array(param_nodes)) => {
                for param_node in param_nodes {
                    params.push(self.parse_type_name(param_node)?);
                }
            }
            _ => bail!("Implement parse_function_type: {}", node),
        }

        let return_node = node.get_return_parameter_types()?;
        let mut returns = vec![];
        match self.get_parameters_node(return_node) {
            Ok(Value::Array(return_nodes)) => {
                for node in return_nodes {
                    let return_type = self.parse_type_name(node)?;
                    returns.push(return_type);
                }
            }
            _ => bail!("Implement parse_function_type: {}", node),
        }

        Ok(FunctionType::new(params, returns, fvis, fmut).into())
    }

    /// Parse an array type from the `typeName` field.
    fn parse_array_type_name(&mut self, node: &Value) -> Result<Type> {
        let base = self.parse_type_name(node.get_base_type()?)?;
        match self.parse_type_descriptions(node)? {
            Type::Array(typ) => {
                Ok(ArrayType::new(base, typ.length, typ.data_loc, typ.is_ptr).into())
            }
            _ => bail!("Fail to parse array type"),
        }
    }

    // /// Parse a user-defined type from the `typeName` field.
    // fn parse_user_defined_type_name(&mut self, node: &Value) -> Result<Type> {
    //     // Parse type name from the path node
    //     let path_node = node.get_path_node()?;
    //     let path_node_name = path_node.get_name()?.to_string();
    //     type_parser::parse_data_type(&path_node_name)

    //     // Parse type name through type description.
    // }

    //-------------------------------------------------
    // Yul function definition
    //-------------------------------------------------

    /// Parse a Yul function definition from a JSON AST node.
    fn parse_yul_func_def(&mut self, node: &Value) -> Result<yast::FuncDef> {
        let name = node.get_name()?;
        let body = self.parse_yul_block(node.get_body()?)?;
        let params = match self.get_parameters_node(node) {
            Ok(Value::Array(param_nodes)) => {
                let mut params = vec![];
                for param_node in param_nodes {
                    params.push(self.parse_yul_ident(param_node)?);
                }
                params
            }
            Ok(_) => bail!("parse_yul_func_def: {}", node),
            Err(_) => vec![],
        };
        let ret_vars = match node.get_return_variables() {
            Ok(Value::Array(return_nodes)) => {
                let mut returns_vars = vec![];
                for return_node in return_nodes {
                    returns_vars.push(self.parse_yul_ident(return_node)?);
                }
                returns_vars
            }
            Ok(_) => {
                bail!("parse_yul_func_def failed: {}", node)
            }
            Err(_) => vec![],
        };
        Ok(yast::FuncDef::new(&name, params, ret_vars, body))
    }

    //-------------------------------------------------
    // Yul block
    //-------------------------------------------------

    /// Parse a Yul block.
    fn parse_yul_block(&mut self, node: &Value) -> Result<yast::Block> {
        let mut stmts: Vec<yast::Stmt> = vec![];
        match node.get_statements()? {
            Value::Array(stmt_nodes) => {
                for stmt_node in stmt_nodes.iter() {
                    stmts.push(self.parse_yul_stmt(stmt_node)?);
                }
            }
            _ => bail!("To parse statements of Yul block: {}", node),
        }

        Ok(yast::Block::new(stmts))
    }

    //-------------------------------------------------
    // Yul variable declaration
    //-------------------------------------------------

    /// Parse a Yul variable declaration statement.
    fn parse_yul_var_decl_stmt(&mut self, node: &Value) -> Result<yast::VarDecl> {
        let mut vars: Vec<yast::Identifier> = vec![];
        match node.get_variables()? {
            Value::Array(vdecl_nodes) => {
                for node in vdecl_nodes.iter() {
                    let identifier = self.parse_yul_ident(node)?;
                    vars.push(identifier);
                }
            }
            v => bail!("Parse variable declarations: {}", v),
        }

        let value = match node.get_value() {
            Ok(value_node) => match value_node {
                Value::Null => None,
                _ => match self.parse_yul_expr(value_node) {
                    Ok(value) => Some(value),
                    Err(err) => bail!(err),
                },
            },
            Err(_) => None,
        };

        Ok(yast::VarDecl::new(vars, value))
    }

    //-------------------------------------------------
    // Yul statement
    //-------------------------------------------------

    /// Parse a Yul statement.
    fn parse_yul_stmt(&mut self, node: &Value) -> Result<yast::Stmt> {
        let node_type = self.get_node_type(node)?;

        match node_type {
            NodeType::YulVarDecl => self.parse_yul_var_decl_stmt(node).map(|stmt| stmt.into()),

            NodeType::YulExprStmt => self.parse_yul_expr_stmt(node).map(|stmt| stmt.into()),

            NodeType::YulFuncDef => self.parse_yul_func_def(node).map(|stmt| stmt.into()),

            NodeType::YulAssign => self.parse_yul_assign_stmt(node).map(|stmt| stmt.into()),

            NodeType::YulIfStmt => self.parse_yul_if_stmt(node).map(|stmt| stmt.into()),

            NodeType::YulForLoop => self.parse_yul_for_loop_stmt(node).map(|stmt| stmt.into()),

            NodeType::YulSwitchStmt => self.parse_yul_switch_stmt(node).map(|stmt| stmt.into()),

            NodeType::YulBlock => self.parse_yul_block(node).map(|stmt| stmt.into()),

            NodeType::YulLeaveStmt => Ok(yast::Stmt::Leave),

            NodeType::YulContinueStmt => Ok(yast::Stmt::Continue),

            NodeType::YulBreakStmt => Ok(yast::Stmt::Break),

            _ => bail!("To implement parse_yul_statement"),
        }
    }

    //-------------------------------------------------
    // Yul assignment statement
    //-------------------------------------------------

    /// Parse a Yul assignment statement
    fn parse_yul_assign_stmt(&mut self, node: &Value) -> Result<yast::AssignStmt> {
        let value = self.parse_yul_expr(node.get_value()?)?;
        let mut vars = vec![];
        match node.get_variable_names() {
            Ok(Value::Array(nodes)) => {
                for var_node in nodes {
                    let var = self.parse_yul_ident(var_node)?;
                    vars.push(var);
                }
            }
            Ok(_) => bail!("parse_yul_assignment failed: {}", node),
            Err(_) => {}
        }

        Ok(yast::AssignStmt::new(vars, value))
    }

    //-------------------------------------------------
    // Yul expression statement
    //-------------------------------------------------

    /// Parse a Yul expression statement.
    fn parse_yul_expr_stmt(&mut self, node: &Value) -> Result<yast::Expr> {
        self.parse_yul_expr(node.get_expression()?)
    }

    //-------------------------------------------------
    // Yul if statement
    //-------------------------------------------------

    /// Parse a Yul if statement.
    fn parse_yul_if_stmt(&mut self, node: &Value) -> Result<yast::IfStmt> {
        let body = self.parse_yul_block(node.get_body()?)?;
        let cond = self.parse_yul_expr(node.get_condition()?)?;
        Ok(yast::IfStmt::new(cond, body))
    }

    //-------------------------------------------------
    // Yul for loop statement
    //-------------------------------------------------

    /// Parse a Yul for loop.
    fn parse_yul_for_loop_stmt(&mut self, node: &Value) -> Result<yast::ForStmt> {
        let pre = self.parse_yul_block(node.get_pre()?)?;
        let body = self.parse_yul_block(node.get_body()?)?;
        let post = self.parse_yul_block(node.get_post()?)?;
        let cond = self.parse_yul_expr(node.get_condition()?)?;
        Ok(yast::ForStmt::new(pre, cond, post, body))
    }

    //-------------------------------------------------
    // Yul switch statement
    //-------------------------------------------------

    /// Parse a Yul switch statement
    fn parse_yul_switch_stmt(&mut self, node: &Value) -> Result<yast::SwitchStmt> {
        let expr = self.parse_yul_expr(node.get_expression()?)?;

        let mut cases = vec![];
        let mut switch_defaults = vec![];
        match node.get_cases() {
            Ok(Value::Array(nodes)) => {
                for node in nodes {
                    if !self.is_yul_switch_default(node) {
                        let case = self.parse_yul_switch_value(node)?;
                        cases.push(case);
                    } else {
                        let case = self.parse_yul_switch_default(node)?;
                        switch_defaults.push(case);
                    }
                }
            }
            Ok(_) => {
                bail!("Failed to parse Yul switch_statement: {}", node)
            }
            Err(_) => {}
        }

        let default = switch_defaults.first();
        if switch_defaults.len() < 2 {
            Ok(yast::SwitchStmt::new(expr, cases, default.cloned()))
        } else {
            bail!("YulSwitch has more than one default case")
        }
    }

    /// Parse a Yul switch case
    fn parse_yul_switch_value(&mut self, node: &Value) -> Result<yast::SwitchValue> {
        let body = self.parse_yul_block(node.get_body()?)?;
        let value = self.parse_yul_lit(node.get_value()?)?;
        Ok(yast::SwitchValue::new(value, body))
    }

    /// Check is it a default switch case
    fn is_yul_switch_default(&self, node: &Value) -> bool {
        match node.get_value() {
            Ok(Value::String(str_val)) => str_val.eq("default"),
            _ => false,
        }
    }

    /// Parse a Yul default switch case
    fn parse_yul_switch_default(&mut self, node: &Value) -> Result<yast::SwitchDefault> {
        let body = self.parse_yul_block(node.get_body()?)?;
        Ok(yast::SwitchDefault::new(body))
    }

    //-------------------------------------------------
    // Yul expression
    //-------------------------------------------------

    /// Parse a Yul expression.
    fn parse_yul_expr(&mut self, node: &Value) -> Result<yast::Expr> {
        match self.get_node_type(node)? {
            NodeType::YulLiteral => self.parse_yul_lit(node).map(|exp| exp.into()),
            NodeType::YulIdent => self.parse_yul_ident_or_member_expr(node),
            NodeType::YulFuncCall => self.parse_yul_function_call(node).map(|exp| exp.into()),
            _ => bail!("Implement Yul parse_expression: {}", node),
        }
    }

    //-------------------------------------------------
    // Yul function call
    //-------------------------------------------------

    /// Parse a Yul function call expression.
    fn parse_yul_function_call(&mut self, node: &Value) -> Result<yast::CallExpr> {
        let callee = node.get_function_name()?.get_name()?;
        let typ = match node.get_type() {
            Ok(type_str) => self.parse_yul_type(&type_str)?,
            Err(_) => YType::Unkn,
        };
        let loc = node.get_source_location(&self.localizer);
        let fn_name = yast::Identifier::new(Name::new(callee, None), typ, loc);

        let mut arg_values: Vec<yast::Expr> = vec![];
        match node.get_arguments() {
            Ok(Value::Array(arg_nodes)) => {
                for arg_node in arg_nodes.iter() {
                    match self.parse_yul_expr(arg_node) {
                        Ok(arg) => arg_values.push(arg),
                        Err(err) => bail!(err),
                    }
                }
            }
            Ok(v) => todo!("Parse Yul function call args: {}", v),
            Err(err) => bail!(err),
        }

        Ok(yast::CallExpr::new(fn_name, arg_values))
    }

    //-------------------------------------------------
    // Yul identifier
    //-------------------------------------------------

    fn parse_yul_ident(&self, node: &Value) -> Result<yast::Identifier> {
        let name = node.get_name()?;
        let loc = node.get_source_location(&self.localizer);
        let typ = match node.get_type() {
            Ok(typ_str) => self.parse_yul_type(&typ_str)?,
            Err(_) => YType::Unkn,
        };
        Ok(yast::Identifier::new(Name::new(name.to_string(), None), typ, loc))
    }

    fn parse_yul_ident_or_member_expr(&self, node: &Value) -> Result<yast::Expr> {
        let name = node.get_name()?;
        let loc = node.get_source_location(&self.localizer);
        let typ = match node.get_type() {
            Ok(typ_str) => self.parse_yul_type(&typ_str)?,
            Err(_) => YType::Unkn,
        };

        let components = name.split('.').collect::<Vec<&str>>();
        match components[..] {
            [name] => {
                Ok(yast::Identifier::new(Name::new(name.to_string(), None), typ, loc).into())
            }
            [name1, name2] => {
                let base = Name::new(name1.to_string(), None);
                let member = Name::new(name2.to_string(), None);
                Ok(yast::MemberExpr::new(base, member, loc).into())
            }
            _ => bail!("Failed to parse Yul identifier: {}", node),
        }
    }

    //-------------------------------------------------
    // Yul literals
    //-------------------------------------------------

    fn parse_yul_lit(&self, node: &Value) -> Result<yast::Lit> {
        match node.get_kind()?.as_str() {
            "number" => match node.get_value() {
                Ok(Value::String(s)) if s.starts_with("0x") => {
                    Ok(yast::NumLit::Hex(s.to_string()).into())
                }
                Ok(v) => {
                    let number = self.parse_int_lit(v)?;
                    Ok(yast::NumLit::Dec(number).into())
                }
                _ => bail!("Failed to parse number literal"),
            },
            _ => match self.parse_yul_hex_lit(node) {
                Ok(lit) => Ok(lit),
                _ => self.parse_yul_string_lit(node).map(|lit| lit.into()),
            },
        }
    }

    fn parse_yul_string_lit(&self, node: &Value) -> Result<yast::StringLit> {
        match node.get_value()? {
            Value::String(s) => Ok(yast::StringLit::new(s)),
            _ => bail!("Need to parse Yul literal: {}", node),
        }
    }

    fn parse_yul_hex_lit(&self, node: &Value) -> Result<yast::Lit> {
        let hex = node.get_hex_value()?;
        Ok(yast::HexLit::new(&hex).into())
    }

    //-------------------------------------------------
    // Yul type
    //-------------------------------------------------

    fn parse_yul_type(&self, data_type: &str) -> Result<YType> {
        match data_type {
            "bool" => Ok(YType::Bool),
            "string" => Ok(YType::String),
            "uint" | "int" => {
                let regex = match Regex::new(r"(\d+)") {
                    Ok(re) => re,
                    Err(_) => bail!("Invalid regexp!"),
                };
                let bitwidth = match regex.captures(data_type) {
                    Some(capture) => match capture.get(1) {
                        Some(m) => {
                            let value = m.as_str();
                            match value.parse::<usize>() {
                                Ok(bw) => bw,
                                Err(_) => {
                                    bail!("Invalid bitwidth: {}", value)
                                }
                            }
                        }
                        None => 256,
                    },
                    None => 256,
                };
                let signed = data_type.starts_with("int");
                Ok(YType::Int(YIntType::new(bitwidth, signed)))
            }
            _ => Ok(YType::Unkn),
        }
    }
}

//------------------------------------------------------------------
// Public functions for parsing Solidity and AST files
//------------------------------------------------------------------

/// Function to parse a Solidity source code file to internal AST
///
/// The two inputs `base_path` and `include_path` are similar to the inputs of
/// Solc.
pub fn parse_solidity_file(
    input_file: &str,
    base_path: Option<&str>,
    include_paths: &[String],
    solc_ver: &str,
) -> Result<Vec<SourceUnit>> {
    // Compile it to JSON AST using Solc
    let json = solc::compile_solidity_file(input_file, base_path, include_paths, solc_ver)?;

    // Parse the JSON AST to internal AST.
    let mut parser = AstParser::new(&json);
    match parser.parse_solidity_json() {
        Ok(source_units) => Ok(source_units),
        Err(err) => bail!(err),
    }
}

/// Function to parse a Solidity source code string to internal AST.
///
/// `solc_ver` is the Solidity version, empty string means unknown version.
pub fn parse_solidity_code(source_code: &str, solc_ver: &str) -> Result<Vec<SourceUnit>> {
    // Save the source code to a temporarily Solidity file
    let solidity_file = match save_to_temporary_file(source_code, "contract.sol") {
        Ok(filename) => filename,
        Err(_) => bail!("Failed to save input contract to file"),
    };

    // Parse the Solidity file to internal AST.
    parse_solidity_file(&solidity_file, None, &[], solc_ver)
}

/// Function to parse multiple Solidity source code strings to internal AST.
pub fn parse_contract_info(
    file_name_and_contents: &[(&str, &str)],
    solc_ver: &str,
) -> Result<Vec<SourceUnit>> {
    // Save the source code to a temporarily Solidity file
    let solidity_files = match save_to_temporary_files(file_name_and_contents) {
        Ok(files) => files,
        Err(_) => bail!("Failed to save input contract to files"),
    };

    // Parse Solidity files to internal AST.
    let mut output_sunits: Vec<SourceUnit> = vec![];
    for input_file in solidity_files {
        let sunits = parse_solidity_file(&input_file, None, &[], solc_ver)?;
        sunits.iter().for_each(|sunit| {
            if !output_sunits.iter().any(|sunit2| sunit.path == sunit2.path) {
                output_sunits.push(sunit.clone())
            }
        })
    }

    // Return result.
    Ok(output_sunits)
}
