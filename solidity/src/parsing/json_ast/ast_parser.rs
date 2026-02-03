//! Parser that parses Solidity AST in JSON format and produces an AST.

use crate::{ast::*, parsing::typ::type_parser};
use crate::ast::yul as yast;
use crate::parsing::yul as yul_parser;
use codespan_reporting::files::{Files, SimpleFiles};
use color_eyre::eyre::Result;
use extlib::{error, fail};
use itertools::izip;
use lazy_static::lazy_static;
use crate::ast::{DataLoc, Loc, Name};
use num_bigint::BigInt;
use regex::Regex;
use rust_decimal::Decimal;
use serde::Value;
use std::{fs, ops::Deref, path::Path, str::FromStr};

//------------------------------------------------------------------
// Static variables
//------------------------------------------------------------------

lazy_static! {
    /// Regular expression to parse source code location in JSON AST data.
    pub static ref LOCATION_REGEX: Regex = Regex::new(r"(\d+):(\d+):(\d+)")
        .unwrap_or_else(|_| panic!("Invalid regular expression!"));
}

//------------------------------------------------------------------
// Data structure representing JSON AST Parser
//------------------------------------------------------------------

pub struct AstParser {
    pub solidity_json: Option<String>,
    pub input_file: Option<String>,
    pub base_path: Option<String>,
    file_dictionary: SimpleFiles<String, String>,
    current_file_id: usize,
}

pub struct JsonAst {
    pub json_data: String, // JSON content
    pub file_name: Option<String>,
    pub base_path: Option<String>, // Base path that is used to look for source tree.
}

//------------------------------------------------------------------
// Implementations for JSON AST
//------------------------------------------------------------------

impl JsonAst {
    pub fn new(json_data: &str, input_file: Option<&str>, base_path: Option<&str>) -> Self {
        JsonAst {
            json_data: json_data.to_string(),
            file_name: input_file.map(|s| s.to_string()),
            base_path: base_path.map(|s| s.to_string()),
        }
    }
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
            file_dictionary: SimpleFiles::new(),
            current_file_id: 0,
        }
    }

    pub fn parse_solidity_json(&mut self) -> Result<Vec<SourceUnit>> {
        let node: Value = match &self.solidity_json {
            Some(content) => serde::from_str(content)?,
            None => fail!("Input JSON AST not found!"),
        };
        let sources_node = node
            .get("sources")
            .ok_or_else(|| error!("Sources node not found in JSON AST: {node}"))?;
        let source_names = node
            .get("sourceList")
            .ok_or_else(|| error!("Source list not found in JSON AST: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Source list is not an array: {node}"))?
            .iter()
            .map(|v| v.as_str().ok_or_else(|| error!("Source name invalid: {v}")))
            .collect::<Result<Vec<&str>>>()?;
        let mut source_units = vec![];
        for source_name in &source_names {
            let source_node = match sources_node.get(source_name) {
                Some(source_node) => source_node,
                None => fail!("Failed to get source node of: {}", source_name),
            };
            let ast_node = source_node
                .get("AST")
                .ok_or_else(|| error!("AST node not found for source: {}", source_name))?;
            source_units.push(self.parse_ast(ast_node)?)
        }
        Ok(source_units)
    }

    //-------------------------------------------------
    // Common utilities to handle AST nodes
    //-------------------------------------------------

    /// Parse the node type of an AST Node
    fn get_node_type(&self, node: &Value) -> Result<String> {
        node.get("nodeType")
            .ok_or_else(|| error!("AST node type not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("AST node type invalid: {node}"))
            .map(|s| s.to_string())
    }

    /// Parse id of an AST Node
    fn parse_id(&self, node: &Value) -> Result<isize> {
        node.get("id")
            .ok_or_else(|| error!("AST node id not found: {node}"))?
            .as_i64()
            .map(|id| id as isize)
            .ok_or_else(|| error!("AST node id invalid: {node}"))
    }

    /// Parse scope of an AST node
    fn parse_scope(&self, node: &Value) -> Result<isize> {
        node.get("scope")
            .ok_or_else(|| error!("AST node scope not found: {node}"))?
            .as_i64()
            .map(|id| id as isize)
            .ok_or_else(|| error!("AST node scope invalid: {node}"))
    }

    /// Parse name of an AST node
    fn parse_name(&self, node: &Value) -> Result<String> {
        node.get("name")
            .ok_or_else(|| error!("AST node name not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("AST node name invalid: {node}"))
            .map(|s| s.to_string())
    }

    /// Parse source location
    fn parse_source_location(&self, node: &Value) -> Option<Loc> {
        let src_loc_info = node.get("src")?.as_str()?;
        LOCATION_REGEX.captures(src_loc_info).and_then(|capture| {
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
                    let (l1, c1) = match self
                        .file_dictionary
                        .location(self.current_file_id, begin_pos)
                    {
                        Ok(loc) => (loc.line_number, loc.column_number),
                        _ => return None,
                    };
                    let (l2, c2) =
                        match self.file_dictionary.location(self.current_file_id, end_pos) {
                            Ok(loc) => (loc.line_number, loc.column_number),
                            _ => return None,
                        };
                    Some(Loc::new(l1, c1, l2, c2))
                }
                _ => None,
            }
        })
    }

    //-------------------------------------------------
    // Combined JSON
    //-------------------------------------------------

    /// Parse a source unit from a JSON AST node.
    fn parse_ast(&mut self, node: &Value) -> Result<SourceUnit> {
        match self.get_node_type(node)?.as_str() {
            "SourceUnit" => self.parse_source_unit(node),
            _ => fail!("Source unit not found: {node}"),
        }
    }

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    /// Parse a source unit from a JSON AST node.
    fn parse_source_unit(&mut self, node: &Value) -> Result<SourceUnit> {
        let id = self.parse_id(node).ok();
        let file_path = self.parse_source_unit_path(node)?;
        let file_source = fs::read_to_string(&file_path)?;
        self.current_file_id = self.file_dictionary.add(file_path.clone(), file_source);
        let elems = node
            .get("nodes")
            .ok_or_else(|| error!("Source unit elements not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Source unit elements invalid: {node}"))?
            .iter()
            .map(|elem_node| self.parse_source_unit_element(elem_node))
            .collect::<Result<Vec<SourceUnitElem>>>()?;
        Ok(SourceUnit::new(id, file_path, elems))
    }

    /// Parse source unit file path.
    ///
    /// Input AST node must be a node representing a source unit.
    fn parse_source_unit_path(&mut self, node: &Value) -> Result<String> {
        let source_file_abs = node
            .get("absolutePath")
            .ok_or_else(|| error!("Parsing source unit: absolute path not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Parsing source unit: absolute path invalid: {node}"))?
            .to_string();
        let path = match &self.base_path {
            None => source_file_abs,
            Some(base) => {
                let path = Path::new(base)
                    .join(&source_file_abs)
                    .as_os_str()
                    .to_os_string()
                    .into_string();
                match path {
                    Ok(source_path) => source_path,
                    Err(_) => source_file_abs,
                }
            }
        };
        Ok(path)
    }

    /// Parse source unit element from a JSON AST node.
    fn parse_source_unit_element(&mut self, node: &Value) -> Result<SourceUnitElem> {
        match self.get_node_type(node)?.as_str() {
            "PragmaDirective" => self.parse_pragma_directive(node).map(SourceUnitElem::from),
            "ImportDirective" => self.parse_import_directive(node).map(SourceUnitElem::from),
            "UsingForDirective" => self.parse_using_directive(node).map(SourceUnitElem::from),
            "ErrorDefinition" => self.parse_error_def(node).map(SourceUnitElem::from),
            "StructDefinition" => self.parse_struct_definition(node).map(SourceUnitElem::from),
            "FunctionDefinition" => self
                .parse_function_definition(node)
                .map(SourceUnitElem::from),
            "UserDefinedValueTypeDefinition" => self
                .parse_user_defined_value_type_def(node)
                .map(SourceUnitElem::from),
            "EnumDefinition" => self.parse_enum_def(node).map(SourceUnitElem::from),
            "ContractDefinition" => self.parse_contract_def(node).map(SourceUnitElem::from),
            "VariableDeclaration" => self
                .parse_variable_declaration(node)
                .map(SourceUnitElem::from),
            _ => fail!("Failed to parse source element: {node}"),
        }
    }

    //-------------------------------------------------
    // Pragma directives.
    //-------------------------------------------------

    /// Parse pragma directive from a JSON AST node.
    fn parse_pragma_directive(&mut self, node: &Value) -> Result<PragmaDir> {
        let id = self.parse_id(node).ok();
        let pragma_lits = node
            .get("literals")
            .ok_or_else(|| error!("Pragma literals not found: {node}"))
            .and_then(|v| match v {
                Value::String(s) => Ok(vec![s.clone()]),
                Value::Array(arr) => Ok(arr
                    .iter()
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect::<Vec<String>>()),
                _ => fail!("Pragma literals invalid!"),
            })?;

        // match self.get_literals(node)? ;
        let kind = match pragma_lits.split_first() {
            Some((first, tail)) => match first.as_str() {
                "solidity" => {
                    let version = tail.join("");
                    PragmaKind::new_version(version)
                }
                "abicoder" => match tail.first() {
                    Some(s) => PragmaKind::new_abi_coder(s.to_string()),
                    None => fail!("Pragma abicoder not found!"),
                },
                "experimental" => match tail.first() {
                    Some(s) => PragmaKind::new_experimental(s.to_string()),
                    None => fail!("Pragma experimental not found!"),
                },
                _ => fail!("Pragma not supported: {}", first),
            },
            None => fail!("Pragma not found!"),
        };
        let loc = self.parse_source_location(node);
        Ok(PragmaDir::new(id, kind, loc))
    }

    //-------------------------------------------------
    // Import directives.
    //-------------------------------------------------

    /// Parse import directive from a JSON AST node.
    fn parse_import_directive(&self, node: &Value) -> Result<ImportDir> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        let file_path = node
            .get("flie")
            .ok_or_else(|| error!("Import directive: file path not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Import directive: file path invalid: {node}"))?
            .to_string();
        let abs_path = node
            .get("absolutePath")
            .ok_or_else(|| error!("Import directive: absolute path not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Import directive: absolute path invalid: {node}"))?
            .to_string();
        let symbol_aliases = node
            .get("symbolAliases")
            .ok_or_else(|| error!("Import directive: symbol aliases not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Import directive: symbol aliases invalid: {node}"))?
            .iter()
            .map(|v| self.parse_symbol_alias(v))
            .collect::<Result<Vec<ImportSymbol>>>()?;
        let unit_alias = node
            .get("unitAlias")
            .ok_or_else(|| error!("Import directive: unit alias not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Import directive: unit alias invalid: {node}"))?
            .to_string();
        let kind = match (&symbol_aliases[..], unit_alias.as_str()) {
            ([], "") => ImportSourceUnit::new(abs_path, file_path, None, loc).into(),
            (_, "") => ImportSymbols::new(abs_path, file_path, symbol_aliases, loc).into(),
            ([], _) => ImportSourceUnit::new(abs_path, file_path, Some(unit_alias), loc).into(),
            _ => fail!("TODO: parse both symbol and unit aliases: {node}"),
        };
        Ok(ImportDir::new(id, kind))
    }

    /// Parse `SymbolAlias` from a JSON AST node.
    fn parse_symbol_alias(&self, node: &Value) -> Result<ImportSymbol> {
        let symbol = node
            .get("foreign")
            .ok_or_else(|| error!("Import symbol alias: foreign key not found: {node}"))
            .and_then(|v| self.parse_name(v))?;
        let alias = node
            .get("local")
            .ok_or_else(|| error!("Import symbol alias: local key not found: {node}"))?
            .as_str()
            .map(|s| s.to_string());
        let loc = self.parse_source_location(node);
        Ok(ImportSymbol::new(symbol, alias, loc))
    }

    //-------------------------------------------------
    // Using directives.
    //-------------------------------------------------

    /// Parse pragma directive from a JSON AST node.
    fn parse_using_directive(&mut self, node: &Value) -> Result<UsingDir> {
        let id = self.parse_id(node).ok();
        let global = node
            .get("global")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let loc = self.parse_source_location(node);
        let kind: Result<_> = if let Some(funcs_node) = node.get("functionList") {
            let using_funcs = funcs_node
                .as_array()
                .ok_or_else(|| error!("Using directive: function list invalid: {}", funcs_node))?
                .iter()
                .map(|v| {
                    if let Some(Value::String(op)) = v.get("operator") {
                        let func_name = v
                            .get("definition")
                            .ok_or_else(|| error!("Using directive: definition not found: {v}"))
                            .and_then(|n| self.parse_name(n))?;
                        Ok(UsingFunc::new(&func_name, Some(op)))
                    } else {
                        let func_name = v
                            .get("function")
                            .ok_or_else(|| error!("Using directive: function not found: {v}"))
                            .and_then(|n| self.parse_name(n))?;
                        Ok(UsingFunc::new(&func_name, None))
                    }
                })
                .collect::<Result<Vec<UsingFunc>>>()?;
            Ok(UsingKind::UsingFunc(using_funcs))
        } else if let Some(lib_node) = node.get("libraryName") {
            let lib_name = self.parse_name(lib_node)?;
            let using_lib = UsingLib::new(&lib_name);
            Ok(UsingKind::UsingLib(using_lib))
        } else {
            fail!("Using directive invalid: {node}");
        };
        let typ = node
            .get("typeName")
            .and_then(|v| self.parse_data_type(v).ok());
        Ok(UsingDir::new(id, kind?, typ, global, loc))
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    /// Parse contract definition from a JSON AST node.
    fn parse_contract_def(&mut self, node: &Value) -> Result<ContractDef> {
        let id = self.parse_id(node).ok();
        let scope = self.parse_scope(node).ok();
        let name = Name::new(self.parse_name(node)?, None);
        let kind = node
            .get("contractKind")
            .ok_or_else(|| error!("Contract kind not found: {node}"))?
            .as_str()
            .map(ContractKind::from_string)
            .ok_or_else(|| error!("Contract kind invalid: {node}"))??;
        let is_abstract = match node.get("abstract") {
            Some(Value::Bool(v)) => v.to_owned(),
            Some(_) => fail!("Contract abstract flag invalid: {node}"),
            None => match node.get("fullyImplemented") {
                Some(Value::Bool(v)) => !v.to_owned(),
                Some(_) => fail!("Contract fully implemented flag invalid: {node}"),
                None => fail!("Contract abstract information not found: {node}"),
            },
        };
        let bases = node
            .get("baseContracts")
            .ok_or_else(|| error!("Base contracts not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Base contracts invalid: {node}"))?
            .iter()
            .map(|base_node| self.parse_base_contract(base_node))
            .collect::<Result<Vec<BaseContract>>>()?;
        let loc = self.parse_source_location(node);
        let elems: Vec<ContractElem> = node
            .get("nodes")
            .ok_or_else(|| error!("Contract elements not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Contract elements invalid: {node}"))?
            .iter()
            .map(|v| self.parse_contract_element(v))
            .collect::<Result<Vec<ContractElem>>>()?;
        Ok(ContractDef::new(id, scope, name, kind, is_abstract, bases, elems, loc))
    }

    /// Parse a base contract from a JSON AST node.
    fn parse_base_contract(&mut self, node: &Value) -> Result<BaseContract> {
        let contract_name: Name = node
            .get("baseName")
            .ok_or_else(|| error!("Base contract name not found: {node}"))
            .and_then(|v| self.parse_name(v))?
            .into();
        let arguments = node
            .get("arguments")
            .map(|args| {
                args.as_array()
                    .ok_or_else(|| error!("Base contract arguments invalid: {node}"))?
                    .iter()
                    .map(|v| self.parse_expr(v))
                    .collect::<Result<Vec<Expr>>>()
            })
            .unwrap_or(Ok(vec![]))?;
        let loc = self.parse_source_location(node);
        Ok(BaseContract::new(contract_name, arguments, loc))
    }

    /// Parse a contract element from a JSON AST node.
    fn parse_contract_element(&mut self, node: &Value) -> Result<ContractElem> {
        match self.get_node_type(node)?.as_str() {
            "StructDefinition" => self.parse_struct_definition(node).map(|def| def.into()),
            "EventDefinition" => self.parse_event_def(node).map(|def| def.into()),
            "ErrorDefinition" => self.parse_error_def(node).map(|def| def.into()),
            "EnumDefinition" => self.parse_enum_def(node).map(|def| def.into()),
            "VariableDeclaration" => self
                .parse_variable_declaration(node)
                .map(|decl| decl.into()),
            "FunctionDefinition" => self.parse_function_definition(node).map(|def| def.into()),
            "ModifierDefinition" => self.parse_modifier_definition(node).map(|def| def.into()),
            "UserDefinedValueTypeDefinition" => self
                .parse_user_defined_value_type_def(node)
                .map(|def| def.into()),
            "UsingForDirective" => self.parse_using_directive(node).map(|dir| dir.into()),
            _ => todo!("Parse contract element: {:?}", node),
        }
    }

    //-------------------------------------------------
    // Type name definition.
    //-------------------------------------------------

    /// Parse a type name definition from a JSON AST node.
    fn parse_user_defined_value_type_def(&mut self, node: &Value) -> Result<TypeDef> {
        let id = self.parse_id(node).ok();
        let scope = self.parse_scope(node).ok();
        let name: Name = self.parse_name(node)?.into();
        let typ = node
            .get("underlyingType")
            .ok_or_else(|| error!("User defined type: underlying type not found: {node}"))
            .map(|v| self.parse_data_type(v))??;
        let loc = self.parse_source_location(node);
        Ok(TypeDef::new(id, scope, name, typ, loc))
    }

    //-------------------------------------------------
    // Struct definition.
    //-------------------------------------------------

    /// Parse a struct definition from a JSON AST node.
    fn parse_struct_definition(&mut self, node: &Value) -> Result<StructDef> {
        let id = self.parse_id(node).ok();
        let scope = self.parse_scope(node).ok();
        let name: Name = self.parse_name(node)?.into();
        let fields = node
            .get("members")
            .ok_or_else(|| error!("Struct members not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Struct members invalid: {node}"))?
            .iter()
            .map(|member_node| self.parse_struct_field(member_node))
            .collect::<Result<Vec<StructField>>>()?;
        let loc = self.parse_source_location(node);
        Ok(StructDef::new(id, scope, name, fields, loc))
    }

    /// Parse a struct field from a JSON AST node.
    fn parse_struct_field(&mut self, node: &Value) -> Result<StructField> {
        let id = self.parse_id(node).ok();
        let name = self.parse_name(node)?;
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(StructField::new(id, name, typ, loc))
    }

    //-------------------------------------------------
    // Enum definition.
    //-------------------------------------------------

    /// Parse an enum definition from a JSON AST node.
    fn parse_enum_def(&self, node: &Value) -> Result<EnumDef> {
        let id = self.parse_id(node).ok();
        let scope = self.parse_scope(node).ok();
        let name: Name = self.parse_name(node)?.into();
        let loc = self.parse_source_location(node);
        let elems = node
            .get("members")
            .ok_or_else(|| error!("Enum members not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Enum members invalid: {node}"))?
            .iter()
            .map(|v| self.parse_name(v))
            .collect::<Result<Vec<String>>>()?;
        Ok(EnumDef::new(id, scope, name, elems, loc))
    }

    //-------------------------------------------------
    // Event definition.
    //-------------------------------------------------

    /// Parse an event definition from a JSON AST node.
    fn parse_event_def(&mut self, node: &Value) -> Result<EventDef> {
        let name: Name = self.parse_name(node)?.into();
        let params = node
            .get("parameters")
            .ok_or_else(|| error!("Event definition: parameters not found: {node}"))
            .and_then(|v| self.parse_parameters(v))?;
        let anonymous = node
            .get("anonymous")
            .ok_or_else(|| error!("Event definition: anonymous flag not found: {node}"))?
            .as_bool()
            .ok_or_else(|| error!("Event definition: anonymous flag invalid: {node}"))?;
        let loc = self.parse_source_location(node);
        Ok(EventDef::new(name, anonymous, params, loc))
    }

    //-------------------------------------------------
    // Error definition.
    //-------------------------------------------------

    /// Parse an error definition from a JSON AST node.
    fn parse_error_def(&mut self, node: &Value) -> Result<ErrorDef> {
        let name: Name = self.parse_name(node)?.into();
        let params = node
            .get("parameters")
            .map(|v| self.parse_parameters(v))
            .ok_or_else(|| error!("Error definition: parameters not found: {node}"))??;
        let loc = self.parse_source_location(node);
        Ok(ErrorDef::new(name, params, loc))
    }

    //-------------------------------------------------
    // Modifier definition.
    //-------------------------------------------------

    /// Parse a modifier definition from a JSON AST node.
    fn parse_modifier_definition(&mut self, node: &Value) -> Result<FuncDef> {
        let id = self.parse_id(node).ok();
        let scope = self.parse_scope(node).ok();
        let name: Name = self.parse_name(node)?.into();
        let is_virtual = node
            .get("virtual")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let loc = self.parse_source_location(node);
        let overriding = self.parse_overriding(node)?;
        let body = node
            .get("body")
            .and_then(|v| self.parse_block(v, false).ok());
        let params = node
            .get("parameters")
            .ok_or_else(|| error!("Modifier parameters not found: {node}"))
            .and_then(|v| self.parse_parameters(v))?;
        Ok(FuncDef::new(
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

    //-------------------------------------------------
    // Function definition.
    //-------------------------------------------------

    /// Parse a function definition from a JSON AST node.
    fn parse_function_definition(&mut self, node: &Value) -> Result<FuncDef> {
        let id = self.parse_id(node).ok();
        let scope = self.parse_scope(node).ok();
        let name = Name::new(self.parse_name(node)?, None);
        let params = node
            .get("parameters")
            .ok_or_else(|| error!("Function parameters not found: {node}"))
            .and_then(|v| self.parse_parameters(v))?;
        let returns = node
            .get("returnParameters")
            .ok_or_else(|| error!("Function return parameters not found: {node}"))
            .map(|v| self.parse_parameters(v))??;
        let kind = match node.get("kind") {
            Some(v) => v
                .as_str()
                .and_then(|s| FuncKind::new(s).ok())
                .ok_or_else(|| error!("Function kind invalid: {node}"))?,
            None => {
                if let Some(Value::Bool(true)) = node.get("isConstructor") {
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
        let is_virtual = node
            .get("virtual")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let fvis = self.parse_function_visibility(node)?;
        let fmut = self.parse_function_mutability(node)?;
        let body = match node.get("body") {
            Some(v) => Some(self.parse_block(v, false)?),
            None => None,
        };
        let modifiers = self.parse_function_modifier_invocations(node)?;
        let overriding = self.parse_overriding(node)?;
        let loc = self.parse_source_location(node);
        Ok(FuncDef::new(
            id, scope, name, kind, body, is_virtual, fvis, fmut, params, modifiers, overriding,
            returns, loc, None,
        ))
    }

    /// Parse function visibility information.
    ///
    /// The input JSON AST node should be a function definition node.
    fn parse_function_visibility(&self, node: &Value) -> Result<FuncVis> {
        node.get("visibility")
            .ok_or_else(|| error!("Function visibility not found: {node}"))?
            .as_str()
            .map(FuncVis::new)
            .ok_or_else(|| error!("Function visibility invalid: {node}"))
    }

    /// Parse function mutability information.
    ///
    /// The input JSON AST node should be a function definition node.
    fn parse_function_mutability(&self, node: &Value) -> Result<FuncMut> {
        node.get("stateMutability")
            .ok_or_else(|| error!("Function mutability not found: {node}"))?
            .as_str()
            .map(FuncMut::new)
            .ok_or_else(|| error!("Function mutability invalid: {node}"))?
    }

    //-------------------------------------------------
    // Parameter list
    //-------------------------------------------------

    /// Parse parameters of a function, event, or error definition.
    fn parse_parameters(&mut self, node: &Value) -> Result<Vec<VarDecl>> {
        let params = node
            .get("parameters")
            .ok_or_else(|| error!("Parameters not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Parameters invalid: {node}"))?
            .iter()
            .map(|param_node| self.parse_variable_declaration(param_node))
            .collect::<Result<Vec<VarDecl>>>()?;
        Ok(params)
    }

    //-------------------------------------------------
    // Overriding specifiers.
    //-------------------------------------------------

    /// Parse an override specifier from a JSON AST node.
    fn parse_overriding(&self, node: &Value) -> Result<Overriding> {
        match node.get("overrides") {
            Some(v) => {
                let contract_names = v
                    .get("overrides")
                    .ok_or_else(|| error!("Overrides node not found: {v}"))?
                    .as_array()
                    .ok_or_else(|| error!("Overrides node invalid: {v}"))?
                    .iter()
                    .map(|n| self.parse_name(n).map(|s| s.into()))
                    .collect::<Result<Vec<Name>>>()?;
                Ok(Overriding::Some(contract_names))
            }
            None => Ok(Overriding::None),
        }
    }

    //-------------------------------------------------
    // Function modifiers.
    //-------------------------------------------------

    /// Parse modifier invocations of a function definition.
    fn parse_function_modifier_invocations(&mut self, node: &Value) -> Result<Vec<CallExpr>> {
        let modifiers = node
            .get("modifiers")
            .ok_or_else(|| error!("Function modifiers not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Function modifiers invalid: {node}"))?
            .iter()
            .map(|v| self.parse_modifier_invocation(v))
            .collect::<Result<Vec<CallExpr>>>()?;
        Ok(modifiers)
    }

    /// Parse modifier invocations of a function from a JSON AST node.
    fn parse_modifier_invocation(&mut self, node: &Value) -> Result<CallExpr> {
        let id = self.parse_id(node).ok();
        let name: Name = node
            .get("modifierName")
            .ok_or_else(|| error!("Modifier invocation name not found: {node}"))
            .and_then(|v| self.parse_name(v))?
            .into();
        let args = node
            .get("arguments")
            .map(|args| {
                args.as_array()
                    .ok_or_else(|| error!("Modifier invocation arguments invalid: {node}"))?
                    .iter()
                    .map(|v| self.parse_expr(v))
                    .collect::<Result<Vec<Expr>>>()
            })
            .unwrap_or(Ok(vec![]))?;
        let kind = node
            .get("kind")
            .ok_or_else(|| error!("Modifier invocation kind not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Modifier invocation kind invalid: {node}"))
            .and_then(CallKind::new)?;
        let arg_typs: Vec<Type> = args.iter().map(|arg| arg.typ()).collect();
        let typ: Type = FuncType::new(arg_typs, vec![], FuncVis::None, FuncMut::None).into();
        let loc = self.parse_source_location(node);
        let callee: Expr = Identifier::new(None, name, typ.clone(), loc).into();
        Ok(CallExpr::new_call_unnamed_args(id, callee, vec![], args, kind, typ, loc))
    }

    //-------------------------------------------------
    // Blocks
    //-------------------------------------------------

    /// Parse a block.
    fn parse_block(&mut self, node: &Value, is_unchecked: bool) -> Result<Block> {
        let id = self.parse_id(node).ok();
        let statements = node
            .get("statements")
            .ok_or_else(|| error!("Block statements not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Block statements invalid: {node}"))?
            .iter()
            .map(|v| self.parse_stmt(v))
            .collect::<Result<Vec<Stmt>>>()?;
        let loc = self.parse_source_location(node);
        Ok(Block::new(id, statements, is_unchecked, loc))
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    /// Parse a statement.
    fn parse_stmt(&mut self, node: &Value) -> Result<Stmt> {
        match self.get_node_type(node)?.as_str() {
            "Block" => self.parse_block(node, false).map(|blk| blk.into()),
            "UncheckedBlock" => self.parse_block(node, true).map(|blk| blk.into()),
            "InlineAssembly" => self.parse_inline_asm_stmt(node),
            "Break" => self.parse_break_stmt(node),
            "Continue" => self.parse_continue_stmt(node),
            "DoWhileStatement" => self.parse_do_while_stmt(node),
            "ExpressionStatement" => self.parse_expr_stmt(node),
            "EmitStatement" => self.parse_emit_stmt(node),
            "ForStatement" => self.parse_for_stmt(node),
            "IfStatement" => self.parse_if_stmt(node),
            "Return" => self.parse_return_stmt(node),
            "PlaceholderStatement" => self.parse_place_holder_stmt(node),
            "RevertStatement" => self.parse_revert_stmt(node),
            "Throw" => self.parse_throw_stmt(node),
            "TryStatement" => self.parse_try_stmt(node),
            "VariableDeclarationStatement" => self.parse_var_decl_stmt(node),
            "WhileStatement" => self.parse_while_stmt(node),
            _ => todo!("Parse statement: {node}"),
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    /// Parse an inline assembly statement
    fn parse_inline_asm_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        let blk = match node.get("AST") {
            // Solidity 0.6 to 0.8
            Some(ast_node) => self.parse_yul_block(ast_node)?,
            // Solidity 0.4, 0.5
            None => match node.get("operations") {
                Some(Value::String(asm)) => yul_parser::parse_inline_assembly_block(asm)?,
                _ => yast::YulBlock::new(vec![]),
            },
        };
        Ok(AsmStmt::new(id, false, vec![], blk.body, loc).into())
    }

    //-------------------------------------------------
    // Break statement
    //-------------------------------------------------

    /// Parse a `break` statement.
    fn parse_break_stmt(&self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        Ok(BreakStmt::new(id, loc).into())
    }

    //-------------------------------------------------
    // Throw statement in Solidity 0.4
    //-------------------------------------------------

    /// Parse a `throw` statement.
    fn parse_throw_stmt(&self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        Ok(ThrowStmt::new(id, loc).into())
    }

    //-------------------------------------------------
    // Continue statement
    //-------------------------------------------------

    /// Parse a `continue` statement.
    fn parse_continue_stmt(&self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        Ok(ContinueStmt::new(id, loc).into())
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    /// Parse an expression statement.
    fn parse_expr_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let expr = node
            .get("expression")
            .ok_or_else(|| error!("Expression statement: expression not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let loc = self.parse_source_location(node);
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
        let id = self.parse_id(node).ok();
        let cond = node
            .get("condition")
            .ok_or_else(|| error!("If statement: condition not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let true_br = node
            .get("trueBody")
            .ok_or_else(|| error!("If statement: true body not found: {node}"))
            .map(|v| self.parse_stmt(v))??;
        let false_br = node
            .get("falseBody")
            .map(|v| self.parse_stmt(v))
            .transpose()?;
        let loc = self.parse_source_location(node);
        Ok(IfStmt::new(id, cond, true_br, false_br, loc).into())
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    /// Parse a `for` statement.
    fn parse_for_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let pre = node
            .get("initializationExpression")
            .ok_or_else(|| error!("For statement: initialization not found: {node}"))
            .and_then(|v| self.parse_stmt(v))
            .ok();
        let cond = node
            .get("condition")
            .ok_or_else(|| error!("For statement: condition not found: {node}"))
            .and_then(|v| self.parse_expr(v))
            .ok();
        let post = node
            .get("loopExpression")
            .ok_or_else(|| error!("For statement: loop expression not found: {node}"))
            .and_then(|v| self.parse_stmt(v))
            .ok();
        let body = node
            .get("body")
            .ok_or_else(|| error!("For statement: body not found: {node}"))
            .and_then(|v| self.parse_stmt(v))?;
        let loc = self.parse_source_location(node);
        Ok(ForStmt::new(id, pre, cond, post, body, loc).into())
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    /// Parse a `while` statement.
    fn parse_while_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let cond = node
            .get("condition")
            .ok_or_else(|| error!("While statement: condition not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let body = node
            .get("body")
            .ok_or_else(|| error!("While statement: body not found: {node}"))
            .and_then(|v| self.parse_stmt(v))?;
        let loc = self.parse_source_location(node);
        Ok(WhileStmt::new(id, cond, body, loc).into())
    }

    //-------------------------------------------------
    // Do-While statement
    //-------------------------------------------------

    /// Parse a `do_while` statement.
    fn parse_do_while_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let cond = node
            .get("condition")
            .ok_or_else(|| error!("Do while statement: condition not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let body = node
            .get("body")
            .ok_or_else(|| error!("Do while statement: body not found: {node}"))
            .and_then(|v| self.parse_stmt(v))?;
        let loc = self.parse_source_location(node);
        Ok(DoWhileStmt::new(id, cond, body, loc).into())
    }

    //-------------------------------------------------
    // Place holder statement
    //-------------------------------------------------

    /// Parse a `place-holder` statement.
    fn parse_place_holder_stmt(&self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        Ok(PlaceholderStmt::new(id, loc).into())
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    /// Parse a `return` statement.
    fn parse_return_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let expr = match node.get("expression") {
            Some(v) => Some(self.parse_expr(v)?),
            None => None,
        };
        let loc = self.parse_source_location(node);
        Ok(ReturnStmt::new(id, expr, loc).into())
    }

    //-------------------------------------------------
    // Try-catch statement
    //-------------------------------------------------

    /// Parse a `try` statement.
    fn parse_try_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        let expr = node
            .get("externalCall")
            .ok_or_else(|| error!("Try statement: external call not found: {node}"))
            .map(|v| self.parse_expr(v))??;
        let clause_nodes = node
            .get("clauses")
            .ok_or_else(|| error!("Try statement: clauses not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Try statement: clauses invalid: {node}"))?;
        match clause_nodes.split_first() {
            // `try` clause + `catch` clauses
            Some((try_node, catch_node)) => {
                let try_block = try_node
                    .get("block")
                    .ok_or_else(|| error!("Try statement: block not found: {}", try_node))
                    .and_then(|v| self.parse_block(v, false))?;
                let params = try_node
                    .get("parameters")
                    .map(|v| self.parse_parameters(v))
                    .ok_or_else(|| {
                        error!("Try statement: parameters not found: {}", try_node)
                    })??;
                let catch_clauses = catch_node
                    .iter()
                    .map(|cls| self.parse_catch_clause(cls))
                    .collect::<Result<Vec<CatchClause>>>()?;
                Ok(TryStmt::new(id, expr, params, try_block, catch_clauses, loc).into())
            }
            None => fail!("Implement parse_try_statement: {node}"),
        }
    }

    /// Parse a `catch` clause in a `try` statement.
    fn parse_catch_clause(&mut self, node: &Value) -> Result<CatchClause> {
        let id = self.parse_id(node).ok();
        let block = node
            .get("block")
            .ok_or_else(|| error!("Catch clause: block not found: {node}"))
            .and_then(|v| self.parse_block(v, false))?;
        let error = node
            .get("errorName")
            .ok_or_else(|| error!("Catch clause: error name not found: {node}"))?
            .as_str();
        let params = node
            .get("parameters")
            .map(|v| self.parse_parameters(v))
            .ok_or_else(|| error!("Catch clause: parameters not found: {node}"))??;
        let loc = self.parse_source_location(node);
        Ok(CatchClause::new(id, error, params, block, loc))
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    /// Parse a `revert` statement.
    fn parse_revert_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        let error_call = node
            .get("errorCall")
            .ok_or_else(|| error!("Revert statement: error call not found: {node}"))
            .and_then(|v| self.parse_function_call(v))?;
        let error = error_call.callee.deref().clone();
        let args = error_call.args;
        Ok(RevertStmt::new(id, Some(error), args, loc).into())
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    /// Parse an `emit` statement.
    fn parse_emit_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let loc = self.parse_source_location(node);
        let event_call = node
            .get("eventCall")
            .ok_or_else(|| error!("Emit statement: event call not found: {node}"))
            .and_then(|v| self.parse_function_call(v))?;
        let event = event_call.callee.deref().clone();
        let args = event_call.args;
        Ok(EmitStmt::new(id, event, args, loc).into())
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    /// Parse a variable declaration statement.
    fn parse_var_decl_stmt(&mut self, node: &Value) -> Result<Stmt> {
        let id = self.parse_id(node).ok();
        let vdecl_nodes = match node.get("declarations") {
            Some(Value::Array(nodes)) => nodes.clone(),
            Some(v) => vec![v.clone()],
            None => fail!("Variable declaration: declarations not found: {node}"),
        };
        let mut vars: Vec<Option<VarDecl>> = vec![];
        for vdecl_node in vdecl_nodes.iter() {
            match vdecl_node {
                Value::Null => vars.push(None),
                _ => match self.parse_variable_declaration(vdecl_node) {
                    Ok(vdecl) => vars.push(Some(vdecl)),
                    Err(err) => fail!(err),
                },
            }
        }
        let value = node
            .get("initialValue")
            .ok_or_else(|| error!("Variable declaration: initial value not found: {node}"))
            .and_then(|v| self.parse_expr(v))
            .ok();
        let loc = self.parse_source_location(node);
        Ok(VarDeclStmt::new(id, vars, value, loc).into())
    }

    //-------------------------------------------------
    // Expressions
    //-------------------------------------------------

    /// Parse an expression.
    fn parse_expr(&mut self, node: &Value) -> Result<Expr> {
        match self.get_node_type(node)?.as_str() {
            "Literal" => self.parse_literal(node).map(|lit| lit.into()),
            "Identifier" => self.parse_identifier(node).map(|id| id.into()),
            "UnaryOperation" => self.parse_unary_expr(node).map(Expr::from),
            "BinaryOperation" => self.parse_binary_expr(node).map(Expr::from),
            "Assignment" => self.parse_assign_expr(node).map(Expr::from),
            "FunctionCall" => self.parse_function_call(node).map(Expr::from),
            "FunctionCallOptions" => self.parse_func_call_opts(node).map(Expr::from),
            "MemberAccess" => self.parse_member_expr(node).map(Expr::from),
            "IndexAccess" => self.parse_index_expr(node).map(Expr::from),
            "TupleExpression" => match node.get("isInlineArray") {
                Some(Value::Bool(true)) => self.parse_inline_array_expr(node).map(Expr::from),
                _ => self.parse_tuple_expr(node).map(Expr::from),
            },
            "ElementaryTypeNameExpression" => self.parse_type_name_expr(node).map(Expr::from),
            "NewExpression" => self.parse_new_expr(node).map(Expr::from),
            "Conditional" => self.parse_conditional_expr(node).map(Expr::from),
            "IndexRangeAccess" => self.parse_slice_expr(node).map(Expr::from),
            _ => todo!("Parse expression: {node}"),
        }
    }

    //-------------------------------------------------
    // Unary expressions
    //-------------------------------------------------

    /// Parse unary expression.
    fn parse_unary_expr(&mut self, node: &Value) -> Result<UnaryExpr> {
        let id = self.parse_id(node).ok();
        let body = node
            .get("subExpression")
            .ok_or_else(|| error!("Unary sub expression not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let is_prefix_op = node
            .get("prefix")
            .ok_or_else(|| error!("Unary  not found: {node}"))?
            .as_bool()
            .unwrap_or(false);
        let op = node
            .get("operator")
            .ok_or_else(|| error!("Unary operator not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Unary operator invalid: {node}"))
            .and_then(|op| UnaryOp::new(op, is_prefix_op))?;
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(UnaryExpr::new(id, op, body, typ, loc))
    }

    //-------------------------------------------------
    // Binary expressions
    //-------------------------------------------------

    /// Parse binary expression.
    fn parse_binary_expr(&mut self, node: &Value) -> Result<BinaryExpr> {
        let id = self.parse_id(node).ok();
        let mut lhs = node
            .get("leftExpression")
            .ok_or_else(|| error!("Binary left expression not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let mut rhs = node
            .get("rightExpression")
            .ok_or_else(|| error!("Binary right expression not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        if let Some(common_type_node) = node.get("commonType") {
            let common_type = common_type_node
                .get("typeString")
                .ok_or_else(|| error!("Binary common type typeString not found: {node}"))?
                .as_str()
                .ok_or_else(|| error!("Binary common type typeString invalid: {node}"))
                .map(type_parser::parse_data_type)??;
            lhs.update_data_type(common_type.clone());
            rhs.update_data_type(common_type);
        }
        let op = node
            .get("operator")
            .ok_or_else(|| error!("Binary operator not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Binary operator invalid: {node}"))
            .and_then(BinOp::new)?;
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(BinaryExpr::new(id, op, lhs, rhs, typ, loc))
    }

    //-------------------------------------------------
    // Assignment expressions
    //-------------------------------------------------

    /// Parse assignment expression.
    fn parse_assign_expr(&mut self, node: &Value) -> Result<AssignExpr> {
        let id = self.parse_id(node).ok();
        let lhs = node
            .get("leftHandSide")
            .ok_or_else(|| error!("Assignment left hand side not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let rhs = node
            .get("rightHandSide")
            .ok_or_else(|| error!("Assignment right hand side not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let op = node
            .get("operator")
            .ok_or_else(|| error!("Assignment operator not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Assignment operator invalid: {node}"))
            .map(AssignOp::new)??;
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);

        Ok(AssignExpr::new(id, op, lhs, rhs, typ, loc))
    }

    //-------------------------------------------------
    // Call expressions
    //-------------------------------------------------

    /// Parse a call expression.
    fn parse_function_call(&mut self, node: &Value) -> Result<CallExpr> {
        let id = self.parse_id(node).ok();
        let (callee, call_opts) = node
            .get("expression")
            .ok_or_else(|| error!("Function call callee not found: {node}"))
            .and_then(|v| self.parse_expr(v))
            .map(|e| match e {
                Expr::CallOpts(exp) => (exp.callee.deref().clone(), exp.call_opts),
                exp => (exp, vec![]),
            })?;
        let (arg_values, arg_names, arg_locs) = self.parse_function_call_arguments(node)?;
        let kind = node
            .get("kind")
            .ok_or_else(|| error!("Function call kind not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Function call kind invalid: {node}"))
            .and_then(CallKind::new)?;
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        if arg_names.is_empty() && arg_locs.is_empty() {
            return Ok(CallExpr::new_call_unnamed_args(
                id, callee, call_opts, arg_values, kind, typ, loc,
            ));
        }

        // Return named arguments
        assert!(
            arg_values.len() == arg_names.len(),
            "Invalid named arguments of function call: {node}"
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

    /// Parse arguments of a function call, which include argument values,
    /// names, and name locations.
    fn parse_function_call_arguments(
        &mut self,
        node: &Value,
    ) -> Result<(Vec<Expr>, Vec<String>, Vec<Option<Loc>>)> {
        let arg_values = node
            .get("arguments")
            .ok_or_else(|| error!("Function call arguments not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Function call arguments invalid: {node}"))?
            .iter()
            .map(|v| self.parse_expr(v))
            .collect::<Result<Vec<Expr>>>()?;
        let arg_names = node
            .get("names")
            .ok_or_else(|| error!("Function call argument names not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Function call argument names invalid: {node}"))?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| error!("Function call argument name invalid: {v}"))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>>>()?;
        let arg_name_locs = match node.get("nameLocations") {
            Some(v) => v
                .as_array()
                .ok_or_else(|| error!("Function call argument name locations invalid: {node}"))?
                .iter()
                .map(|v| self.parse_source_location(v))
                .collect::<Vec<Option<Loc>>>(),
            None => vec![], // Older Solidity doesn't generate JSON key `nameLocations`
        };
        Ok((arg_values, arg_names, arg_name_locs))
    }

    //-------------------------------------------------
    // Function call options expressions
    //-------------------------------------------------

    /// Parse a `FunctionCallOptions` expression.
    fn parse_func_call_opts(&mut self, node: &Value) -> Result<CallOptsExpr> {
        let id = self.parse_id(node).ok();
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        let callee = node
            .get("expression")
            .ok_or_else(|| error!("Function call options callee not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let call_opt_names = node
            .get("names")
            .ok_or_else(|| error!("Function call options names not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Function call options names invalid: {node}"))?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| error!("Function call options name invalid: {v}"))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>>>()?;
        let call_opt_values = node
            .get("options")
            .ok_or_else(|| error!("Function call options not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Function call options invalid: {node}"))?
            .iter()
            .map(|v| self.parse_expr(v))
            .collect::<Result<Vec<Expr>>>()?;
        let call_opts = call_opt_names
            .iter()
            .zip(call_opt_values.iter())
            .map(|(name, value)| CallOpt::new(name.to_string(), value.clone(), value.loc()))
            .collect();
        Ok(CallOptsExpr::new(id, callee, call_opts, typ, loc))
    }

    //-------------------------------------------------
    // Member-access expression
    //-------------------------------------------------

    /// Parse a member access expression.
    fn parse_member_expr(&mut self, node: &Value) -> Result<MemberExpr> {
        let id = self.parse_id(node).ok();
        let base = node
            .get("expression")
            .ok_or_else(|| error!("Member expression base not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let member = node
            .get("memberName")
            .ok_or_else(|| error!("Member expression member name not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Member expression member name invalid: {node}"))?
            .into();
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(MemberExpr::new(id, base, member, typ, loc))
    }

    //-------------------------------------------------
    // Index-access expression.
    //-------------------------------------------------

    /// Parse an index access expression.
    fn parse_index_expr(&mut self, node: &Value) -> Result<IndexExpr> {
        let id = self.parse_id(node).ok();
        let base = node
            .get("baseExpression")
            .ok_or_else(|| error!("Index expression base not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let index = node
            .get("indexExpression")
            .ok_or_else(|| error!("Index expression not found: {node}"))
            .and_then(|v| self.parse_expr(v))
            .ok();
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(IndexExpr::new(id, base, index, typ, loc))
    }

    //-------------------------------------------------
    // The slice expression
    //-------------------------------------------------

    /// Parse a slice expression
    fn parse_slice_expr(&mut self, node: &Value) -> Result<SliceExpr> {
        let id = self.parse_id(node).ok();
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        let base = node
            .get("baseExpression")
            .ok_or_else(|| error!("Slice expression base not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let start_idx = node
            .get("startExpression")
            .ok_or_else(|| error!("Slice start expression not found: {node}"))
            .map(|v| self.parse_expr(v).ok())?;
        let end_idx = node
            .get("endExpression")
            .ok_or_else(|| error!("Slice end expression not found: {node}"))
            .map(|v| self.parse_expr(v).ok())?;
        Ok(SliceExpr::new(id, base, start_idx, end_idx, typ, loc))
    }

    //-------------------------------------------------
    // Tuple expression.
    //-------------------------------------------------

    /// Parse a tuple expression.
    fn parse_tuple_expr(&mut self, node: &Value) -> Result<TupleExpr> {
        let id = self.parse_id(node).ok();
        let elems = node
            .get("components")
            .ok_or_else(|| error!("Tuple expression components not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Tuple expression components invalid: {node}"))?
            .iter()
            .map(|v| self.parse_expr(v).ok())
            .collect::<Vec<Option<Expr>>>();
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(TupleExpr::new(id, elems, typ, loc))
    }

    //-------------------------------------------------
    // Inline array expression.
    //-------------------------------------------------

    /// Parse an inline array expression.
    fn parse_inline_array_expr(&mut self, node: &Value) -> Result<InlineArrayExpr> {
        let id = self.parse_id(node).ok();
        let elems = node
            .get("components")
            .ok_or_else(|| error!("Inline array expression components not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Inline array expression components invalid: {node}"))?
            .iter()
            .map(|v| self.parse_expr(v))
            .collect::<Result<Vec<Expr>>>()?;
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(InlineArrayExpr::new(id, elems, typ, loc))
    }

    //-------------------------------------------------
    // Elementary type name expression
    //-------------------------------------------------

    /// Parse an elementary type name expression.
    fn parse_type_name_expr(&mut self, node: &Value) -> Result<TypeNameExpr> {
        let id = self.parse_id(node).ok();
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(TypeNameExpr::new(id, typ, loc))
    }

    //-------------------------------------------------
    // The `new` expression
    //-------------------------------------------------

    /// Parse a `new` expression.
    fn parse_new_expr(&mut self, node: &Value) -> Result<NewExpr> {
        let id = self.parse_id(node).ok();
        let typ = node
            .get("typeName")
            .ok_or_else(|| error!("New expression type name not found: {node}"))
            .and_then(|v| self.parse_data_type(v))?;
        let loc = self.parse_source_location(node);
        Ok(NewExpr::new(id, typ, loc))
    }

    //-------------------------------------------------
    // The conditional expression
    //-------------------------------------------------

    /// Parse a conditional expression.
    fn parse_conditional_expr(&mut self, node: &Value) -> Result<ConditionalExpr> {
        let id = self.parse_id(node).ok();
        let cond = node
            .get("condition")
            .ok_or_else(|| error!("Conditional expression: condition not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let true_br = node
            .get("trueExpression")
            .ok_or_else(|| error!("Conditional true expression not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let false_br = node
            .get("falseExpression")
            .ok_or_else(|| error!("Conditional false expression not found: {node}"))
            .and_then(|v| self.parse_expr(v))?;
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(ConditionalExpr::new(id, cond, true_br, false_br, typ, loc))
    }

    //-------------------------------------------------
    // Parsing variable declaration.
    //-------------------------------------------------

    /// Parse a variable declaration from a JSON AST node.
    fn parse_variable_declaration(&mut self, node: &Value) -> Result<VarDecl> {
        let id = self.parse_id(node).ok();
        let scope = self.parse_scope(node).ok();
        let name = Name::new(self.parse_name(node)?, None);
        let value = node.get("value").and_then(|v| self.parse_expr(v).ok());
        let mutability = match node.get("mutability") {
            Some(v) => v
                .as_str()
                .ok_or_else(|| error!("Variable mutability invalid: {node}"))
                .and_then(VarMut::new)?,
            None => VarMut::None,
        };
        let is_state_var = node
            .get("stateVariable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let visibility = self.parse_variable_visibility(node)?;
        let overriding = self.parse_overriding(node)?;
        let data_loc = node
            .get("storageLocation")
            .ok_or_else(|| error!("Variable declaration: storage location not found: {node}"))?
            .as_str()
            .and_then(|s| DataLoc::new(s).ok());
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(VarDecl::new(
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

    /// Parse variable visibility information.
    ///
    /// The input JSON AST node should be a variable declaration node.
    fn parse_variable_visibility(&self, node: &Value) -> Result<VarVis> {
        node.get("visibility")
            .ok_or_else(|| error!("Variable visibility not found: {node}"))?
            .as_str()
            .map(VarVis::new)
            .ok_or_else(|| error!("Variable visibility invalid: {node}"))
    }

    //-------------------------------------------------
    // Parsing identifier.
    //-------------------------------------------------

    /// Parse an identifier from a JSON AST node.
    fn parse_identifier(&mut self, node: &Value) -> Result<Identifier> {
        let id = self.parse_id(node).ok();
        let name = Name::new(self.parse_name(node)?, None);
        let typ = self.parse_data_type(node)?;
        let loc = self.parse_source_location(node);
        Ok(Identifier::new(id, name, typ, loc))
    }

    //-------------------------------------------------
    // Parsing literal expressions
    //-------------------------------------------------

    /// Parse literal expression.
    fn parse_literal(&mut self, node: &Value) -> Result<Lit> {
        let loc = self.parse_source_location(node);
        let typ = self.parse_data_type(node)?;
        let value_node = node
            .get("value")
            .ok_or_else(|| error!("Literal value not found: {node}"));

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
                let unit = node
                    .get("subdenomination")
                    .and_then(|v| v.as_str())
                    .map(NumUnit::new);
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

            Type::String(_) => node
                .get("kind")
                .ok_or_else(|| error!("Literal string kind not found: {node}"))?
                .as_str()
                .ok_or_else(|| error!("Literal string kind invalid: {node}"))
                .map(|s| match s {
                    "hexString" => {
                        let hex_value = node
                            .get("hexValue")
                            .ok_or_else(|| error!("Literal hex value not found: {node}"))?
                            .as_str()
                            .ok_or_else(|| error!("Literal hex value invalid: {node}"))?;
                        Ok(HexLit::new(hex_value, typ, loc).into())
                    }

                    "unicodeString" => match value_node? {
                        Value::String(value) => {
                            Ok(UnicodeLit::new(value.to_string(), typ, loc).into())
                        }
                        _ => fail!("Failed to parse unicode string: {node}"),
                    },

                    // REVIEW: why not parsing to normal string first?
                    "string" => match value_node {
                        Ok(value) => {
                            let value = self.parse_string_lit(value)?;
                            if value.is_ascii() {
                                Ok(StringLit::new(value, typ, loc).into())
                            } else {
                                Ok(UnicodeLit::new(value, typ, loc).into())
                            }
                        }
                        _ => fail!("Failed to parse string literal: {node}"),
                    },

                    _ => fail!("Literal kind not found: {:?}", node),
                })?,
            _ => fail!("Need to parse literal type: {}", typ),
        }
    }

    /// Parse a bool literal.
    fn parse_bool_lit(&self, node: &Value) -> Result<bool> {
        match node.as_str() {
            Some(s) => s.parse::<bool>().map_err(|err| error!("{}", err)),
            None => fail!("Failed to parse bool literal: {node}"),
        }
    }

    /// Parse an integer literal to big integer.
    fn parse_int_lit(&self, node: &Value) -> Result<BigInt> {
        match node.as_str() {
            Some(s) => s.parse::<BigInt>().map_err(|err| error!("{}", err)),
            None => fail!("Failed to parse integer literal: {node}"),
        }
    }

    /// Parse a rational literal to a fixed-point number.
    fn parse_rational_lit(&self, node: &Value) -> Result<Decimal> {
        match node.as_str() {
            Some(s) => match Decimal::from_str(s) {
                Ok(n) => Ok(n),
                Err(_) => fail!("Failed to parse decimal: {}", s),
            },
            None => fail!("Failed to parse rational literal: {node}"),
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
            None => fail!("Failed to parse string literal: {node}"),
        }
    }

    //-------------------------------------------------
    // Data types
    //-------------------------------------------------

    /// Get data type of a JSON AST node.
    fn parse_data_type(&mut self, node: &Value) -> Result<Type> {
        let data_loc = match node.get("storageLocation") {
            Some(v) => v
                .as_str()
                .ok_or_else(|| error!("Data location invalid: {node}"))
                .and_then(DataLoc::new)?,
            None => DataLoc::None,
        };
        // First, parse data type from the `typeName` information.
        if let Some(type_name_node) = node.get("typeName")
            && let Ok(mut output_typ) = self.parse_type_name(type_name_node)
        {
            // Update data location if it is specified and not default
            if data_loc != DataLoc::None {
                output_typ.set_data_loc(data_loc);
            }
            return Ok(output_typ);
        }
        let mut output_typ = node
            .get("typeDescriptions")
            .ok_or_else(|| error!("Type descriptions not found: {node}"))?
            .get("typeString")
            .ok_or_else(|| error!("Type string not found in type descriptions: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Type string is not a string: {node}"))
            .map(type_parser::parse_data_type)??;
        // Update data location if it is specified and not default
        if data_loc != DataLoc::None {
            output_typ.set_data_loc(data_loc);
        }
        Ok(output_typ)
    }

    //-------------------------------------------------
    // Type descriptions
    //-------------------------------------------------

    /// Parse type from type descriptions.
    fn parse_type_descriptions(&self, node: &Value) -> Result<Type> {
        let mut typ = node
            .get("typeDescriptions")
            .ok_or_else(|| error!("Type descriptions not found: {node}"))?
            .get("typeString")
            .ok_or_else(|| error!("Type string not found in type descriptions: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Type string is not a string: {node}"))
            .and_then(type_parser::parse_data_type)?;
        let data_loc = node
            .get("storageLocation")
            .ok_or_else(|| error!("Type description: storage location not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Type description storage location invalid: {node}"))
            .and_then(DataLoc::new)?;
        typ.set_data_loc(data_loc);
        Ok(typ)
    }

    //-------------------------------------------------
    // Type name
    //-------------------------------------------------

    /// Get the type of a JSON AST node from the `typeName` field.
    fn parse_type_name(&mut self, node: &Value) -> Result<Type> {
        let output_typ = match self.get_node_type(node)?.as_str() {
            "ArrayTypeName" => self.parse_array_type_name(node),
            "FunctionTypeName" => self.parse_function_type_name(node),
            _ => Ok(self.parse_type_descriptions(node)?),
        };
        match output_typ {
            Ok(mut typ) => {
                // Update type name and its scope when possible
                if let Some(v) = node.get("pathNode") {
                    let type_path_components: Vec<Name> =
                        self.parse_name(v)?.split('.').map(Name::from).collect();
                    let (scope, type_name) = match &type_path_components[..] {
                        [scope, type_name] => (Some(scope.clone()), type_name.clone()),
                        _ => fail!("Type path invalid: {}!", v),
                    };
                    typ.update_name(type_name);
                    typ.update_scope(scope);
                }
                Ok(typ)
            }
            Err(err) => fail!(err),
        }
    }

    /// Parse a function type name
    fn parse_function_type_name(&mut self, node: &Value) -> Result<Type> {
        let fvis = self.parse_function_visibility(node)?;
        let fmut = self.parse_function_mutability(node)?;
        let params = node
            .get("parameterTypes")
            .ok_or_else(|| error!("Function type: parameter types not found: {node}"))?
            .get("parameters")
            .ok_or_else(|| error!("Function type: parameter types not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Function type: parameter types invalid: {node}"))?
            .iter()
            .map(|n| self.parse_type_name(n))
            .collect::<Result<Vec<Type>>>()?;
        let returns = node
            .get("returnParameterTypes")
            .ok_or_else(|| error!("Function type: return parameter types not found: {node}"))?
            .get("parameters")
            .ok_or_else(|| error!("Function type: return parameter types not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Function type: return parameter types invalid: {node}"))?
            .iter()
            .map(|n| self.parse_type_name(n))
            .collect::<Result<Vec<Type>>>()?;
        Ok(FuncType::new(params, returns, fvis, fmut).into())
    }

    /// Parse an array type from the `typeName` field.
    fn parse_array_type_name(&mut self, node: &Value) -> Result<Type> {
        let base = node
            .get("baseType")
            .ok_or_else(|| error!("Array type: base type not found: {node}"))
            .and_then(|v| self.parse_data_type(v))?;
        match self.parse_type_descriptions(node)? {
            Type::Array(typ) => {
                Ok(ArrayType::new(base, typ.length, typ.data_loc, typ.is_ptr).into())
            }
            _ => fail!("Fail to parse array type"),
        }
    }

    //-------------------------------------------------
    // Yul function definition
    //-------------------------------------------------

    /// Parse a Yul function definition from a JSON AST node.
    fn parse_yul_func_def(&mut self, node: &Value) -> Result<yast::YulFuncDef> {
        let name = self.parse_name(node)?;
        let body = node
            .get("body")
            .ok_or_else(|| error!("Yul function body not found: {node}"))
            .and_then(|v| self.parse_yul_block(v))?;
        let params = node
            .get("parameters")
            .ok_or_else(|| error!("Yul function parameters not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Yul function parameters invalid: {node}"))?
            .iter()
            .map(|v| self.parse_yul_ident(v))
            .collect::<Result<Vec<_>>>()?;
        let ret_vars = node
            .get("returnVariables")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .map(|v| self.parse_yul_ident(v))
            .collect::<Result<Vec<_>>>()?;
        Ok(yast::YulFuncDef::new(&name, params, ret_vars, body))
    }

    //-------------------------------------------------
    // Yul block
    //-------------------------------------------------

    /// Parse a Yul block.
    fn parse_yul_block(&mut self, node: &Value) -> Result<yast::YulBlock> {
        let stmts = node
            .get("statements")
            .ok_or_else(|| error!("Yul block statements not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Yul block statements invalid: {node}"))?
            .iter()
            .map(|v| self.parse_yul_stmt(v))
            .collect::<Result<Vec<_>>>()?;
        Ok(yast::YulBlock::new(stmts))
    }

    //-------------------------------------------------
    // Yul variable declaration
    //-------------------------------------------------

    /// Parse a Yul variable declaration statement.
    fn parse_yul_var_decl_stmt(&mut self, node: &Value) -> Result<yast::YulVarDecl> {
        let vars = node
            .get("variables")
            .ok_or_else(|| error!("Yul variable declarations not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Yul variable declarations invalid: {node}"))?
            .iter()
            .map(|v| self.parse_yul_ident(v))
            .collect::<Result<Vec<_>>>()?;
        let value = node.get("value").and_then(|v| self.parse_yul_expr(v).ok());
        Ok(yast::YulVarDecl::new(vars, value))
    }

    //-------------------------------------------------
    // Yul statement
    //-------------------------------------------------

    /// Parse a Yul statement.
    fn parse_yul_stmt(&mut self, node: &Value) -> Result<yast::YulStmt> {
        match self.get_node_type(node)?.as_str() {
            "YulVariableDeclaration" => self.parse_yul_var_decl_stmt(node).map(|stmt| stmt.into()),
            "YulExpressionStatement" => self.parse_yul_expr_stmt(node).map(|stmt| stmt.into()),
            "YulFunctionDefinition" => self.parse_yul_func_def(node).map(|stmt| stmt.into()),
            "YulAssignment" => self.parse_yul_assign_stmt(node).map(|stmt| stmt.into()),
            "YulIf" => self.parse_yul_if_stmt(node).map(|stmt| stmt.into()),
            "YulForLoop" => self.parse_yul_for_loop_stmt(node).map(|stmt| stmt.into()),
            "YulSwitch" => self.parse_yul_switch_stmt(node).map(|stmt| stmt.into()),
            "YulBlock" => self.parse_yul_block(node).map(|stmt| stmt.into()),
            "YulLeave" => Ok(yast::YulStmt::Leave),
            "YulContinue" => Ok(yast::YulStmt::Continue),
            "YulBreak" => Ok(yast::YulStmt::Break),
            _ => todo!("parse yul statement: {node}"),
        }
    }

    //-------------------------------------------------
    // Yul assignment statement
    //-------------------------------------------------

    /// Parse a Yul assignment statement
    fn parse_yul_assign_stmt(&mut self, node: &Value) -> Result<yast::YulAssignStmt> {
        let vars = node
            .get("variableNames")
            .ok_or_else(|| error!("Yul assignment variable names note found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Yul assignment variable names invalid: {node}"))?
            .iter()
            .map(|v| self.parse_yul_ident(v))
            .collect::<Result<Vec<_>>>()?;
        let value = node
            .get("value")
            .ok_or_else(|| error!("Yul assignment value not found: {node}"))
            .and_then(|v| self.parse_yul_expr(v))?;
        Ok(yast::YulAssignStmt::new(vars, value))
    }

    //-------------------------------------------------
    // Yul expression statement
    //-------------------------------------------------

    /// Parse a Yul expression statement.
    fn parse_yul_expr_stmt(&mut self, node: &Value) -> Result<yast::YulExpr> {
        node.get("expression")
            .ok_or_else(|| error!("Yul expression statement expression not found: {node}"))
            .and_then(|v| self.parse_yul_expr(v))
    }

    //-------------------------------------------------
    // Yul if statement
    //-------------------------------------------------

    /// Parse a Yul if statement.
    fn parse_yul_if_stmt(&mut self, node: &Value) -> Result<yast::YulIfStmt> {
        let body = node
            .get("body")
            .ok_or_else(|| error!("Yul if statement body not found: {node}"))
            .and_then(|v| self.parse_yul_block(v))?;
        let cond = node
            .get("condition")
            .ok_or_else(|| error!("Yul if statement condition not found: {node}"))
            .and_then(|v| self.parse_yul_expr(v))?;
        Ok(yast::YulIfStmt::new(cond, body))
    }

    //-------------------------------------------------
    // Yul for loop statement
    //-------------------------------------------------

    /// Parse a Yul for loop.
    fn parse_yul_for_loop_stmt(&mut self, node: &Value) -> Result<yast::YulForStmt> {
        let pre = node
            .get("pre")
            .ok_or_else(|| error!("Yul loop statement: pre not found: {node}"))
            .and_then(|v| self.parse_yul_block(v))?;
        let body = node
            .get("body")
            .ok_or_else(|| error!("Yul loop statement: body not found: {node}"))
            .and_then(|v| self.parse_yul_block(v))?;
        let post = node
            .get("post")
            .ok_or_else(|| error!("Yul loop statement: post not found: {node}"))
            .and_then(|v| self.parse_yul_block(v))?;
        let cond = node
            .get("condition")
            .ok_or_else(|| error!("Yul loop statement: condition not found: {node}"))
            .and_then(|v| self.parse_yul_expr(v))?;
        Ok(yast::YulForStmt::new(pre, cond, post, body))
    }

    //-------------------------------------------------
    // Yul switch statement
    //-------------------------------------------------

    /// Parse a Yul switch statement
    fn parse_yul_switch_stmt(&mut self, node: &Value) -> Result<yast::YulSwitchStmt> {
        let expr = node
            .get("expression")
            .ok_or_else(|| error!("Yul switch expression not found: {node}"))
            .and_then(|n| self.parse_yul_expr(n))?;
        let mut cases = vec![];
        let mut defaults = vec![];
        match node.get("cases") {
            Some(Value::Array(vs)) => {
                for v in vs {
                    if v.get("value")
                        .is_some_and(|n| n.is_string() && n.as_str() == Some("default"))
                    {
                        let default = v
                            .get("body")
                            .ok_or_else(|| error!("Yul switch default body not found: {v}"))
                            .and_then(|v| self.parse_yul_block(v))?;
                        defaults.push(yast::YulSwitchDefault::new(default));
                    } else {
                        let body = v
                            .get("body")
                            .ok_or_else(|| error!("Yul switch case body not found: {node}"))
                            .and_then(|n| self.parse_yul_block(n))?;
                        let value = v
                            .get("value")
                            .ok_or_else(|| error!("Yul switch case value not found: {node}"))
                            .and_then(|n| self.parse_yul_lit(n))?;
                        cases.push(yast::YulSwitchValue::new(value, body));
                    }
                }
            }
            Some(_) => fail!("Yul switch statement cases invalid: {node}"),
            None => {}
        }
        let default = defaults.first();
        if defaults.len() < 2 {
            Ok(yast::YulSwitchStmt::new(expr, cases, default.cloned()))
        } else {
            fail!("Yul switch statement has multiple default case: {node}");
        }
    }

    //-------------------------------------------------
    // Yul expression
    //-------------------------------------------------

    /// Parse a Yul expression.
    fn parse_yul_expr(&mut self, node: &Value) -> Result<yast::YulExpr> {
        match self.get_node_type(node)?.as_str() {
            "YulLiteral" => self.parse_yul_lit(node).map(|exp| exp.into()),
            "YulIdentifier" => self.parse_yul_ident_or_member_expr(node),
            "YulFunctionCall" => self.parse_yul_function_call(node).map(|exp| exp.into()),
            _ => fail!("Parse Yul expression: {node}"),
        }
    }

    //-------------------------------------------------
    // Yul function call
    //-------------------------------------------------

    /// Parse a Yul function call expression.
    fn parse_yul_function_call(&mut self, node: &Value) -> Result<yast::YulCallExpr> {
        let callee = node
            .get("functionName")
            .ok_or_else(|| error!("Function call callee not found: {node}"))
            .and_then(|v| self.parse_name(v))?;
        let typ = node
            .get("type")
            .ok_or_else(|| error!("Function call type not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Function call type invalid: {node}"))
            .and_then(|s| self.parse_yul_type(s))?;
        let loc = self.parse_source_location(node);
        let fn_name = yast::YulIdentifier::new(Name::new(callee, None), typ, loc);
        let arg_values = node
            .get("arguments")
            .ok_or_else(|| error!("Function call arguments not found: {node}"))?
            .as_array()
            .ok_or_else(|| error!("Function call arguments invalid: {node}"))?
            .iter()
            .map(|v| self.parse_yul_expr(v))
            .collect::<Result<Vec<yast::YulExpr>>>()?;
        Ok(yast::YulCallExpr::new(fn_name, arg_values))
    }

    //-------------------------------------------------
    // Yul identifier
    //-------------------------------------------------

    fn parse_yul_ident(&self, node: &Value) -> Result<yast::YulIdentifier> {
        let name = self.parse_name(node)?;
        let loc = self.parse_source_location(node);
        let typ = node
            .get("type")
            .ok_or_else(|| error!("Yul identifier type not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Yul identifier type invalid: {node}"))
            .and_then(|s| self.parse_yul_type(s))?;
        Ok(yast::YulIdentifier::new(Name::new(name.to_string(), None), typ, loc))
    }

    fn parse_yul_ident_or_member_expr(&self, node: &Value) -> Result<yast::YulExpr> {
        let name = self.parse_name(node)?;
        let loc = self.parse_source_location(node);
        let typ = node
            .get("type")
            .ok_or_else(|| error!("Yul identifier type not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Yul identifier type invalid: {node}"))
            .and_then(|s| self.parse_yul_type(s))?;
        let components = name.split('.').collect::<Vec<&str>>();
        match components[..] {
            [name] => {
                Ok(yast::YulIdentifier::new(Name::new(name.to_string(), None), typ, loc).into())
            }
            [name1, name2] => {
                let base = Name::new(name1.to_string(), None);
                let member = Name::new(name2.to_string(), None);
                Ok(yast::YulMemberExpr::new(base, member, loc).into())
            }
            _ => fail!("Failed to parse Yul identifier: {node}"),
        }
    }

    //-------------------------------------------------
    // Yul literals
    //-------------------------------------------------

    fn parse_yul_lit(&self, node: &Value) -> Result<yast::YulLit> {
        node.get("kind")
            .ok_or_else(|| error!("Yul literal kind not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Yul literal kind invalid: {node}"))
            .and_then(|s| match s {
                "number" => match node.get("value") {
                    Some(Value::String(s)) if s.starts_with("0x") => {
                        Ok(yast::YulNumLit::Hex(s.to_string()).into())
                    }
                    Some(v) => {
                        let number = self.parse_int_lit(v)?;
                        Ok(yast::YulNumLit::Dec(number).into())
                    }
                    _ => fail!("Failed to parse number literal"),
                },
                _ => match self.parse_yul_hex_lit(node) {
                    Ok(lit) => Ok(lit),
                    _ => self.parse_yul_string_lit(node).map(|lit| lit.into()),
                },
            })
    }

    fn parse_yul_string_lit(&self, node: &Value) -> Result<yast::YulStringLit> {
        node.get("value")
            .ok_or_else(|| error!("Yul string literal value not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Yul string literal value invalid: {node}"))
            .and_then(|s| Ok(yast::YulStringLit::new(s)))
    }

    fn parse_yul_hex_lit(&self, node: &Value) -> Result<yast::YulLit> {
        let hex_value = node
            .get("hexValue")
            .ok_or_else(|| error!("Yul literal hex value not found: {node}"))?
            .as_str()
            .ok_or_else(|| error!("Yul literal hex value invalid: {node}"))?;
        Ok(yast::YulHexLit::new(hex_value).into())
    }

    //-------------------------------------------------
    // Yul type
    //-------------------------------------------------

    fn parse_yul_type(&self, data_type: &str) -> Result<yast::YulType> {
        match data_type {
            "bool" => Ok(yast::YulType::Bool),
            "string" => Ok(yast::YulType::String),
            "uint" | "int" => {
                let regex = match Regex::new(r"(\d+)") {
                    Ok(re) => re,
                    Err(_) => fail!("Invalid regexp!"),
                };
                let bitwidth = match regex.captures(data_type) {
                    Some(capture) => match capture.get(1) {
                        Some(m) => {
                            let value = m.as_str();
                            match value.parse::<usize>() {
                                Ok(bw) => bw,
                                Err(_) => fail!("Invalid bitwidth: {value}"),
                            }
                        }
                        None => 256,
                    },
                    None => 256,
                };
                let signed = data_type.starts_with("int");
                Ok(yast::YulType::Int(yast::YulIntType::new(bitwidth, signed)))
            }
            _ => Ok(yast::YulType::Unkn),
        }
    }
}
