//! Parser for Vyper's JSON AST.
//!
//! Converts the JSON output of `vyper -f ast` into our internal Vyper AST
//! types.

use crate::vyper::ast::defs::*;
use crate::vyper::ast::exprs::*;
use crate::vyper::ast::loc::Loc;
use crate::vyper::ast::source_unit::*;
use crate::vyper::ast::stmts::*;
use crate::vyper::ast::types::{IntType, Type, UIntType};
use common::{error::Result, fail};
use serde_json::Value;

/// The Vyper JSON AST parser.
pub struct AstParser;

impl AstParser {
    /// Parse a JSON string (the full output of `vyper -f ast`) into a
    /// SourceUnit.
    pub fn parse(json_str: &str, file_path: &str) -> Result<SourceUnit> {
        let root: Value = serde_json::from_str(json_str)
            .map_err(|e| common::error::create_error(format!("Failed to parse JSON: {e}")))?;

        // The vyper compiler outputs either:
        //   { "ast": { "ast_type": "Module", ... } }
        // or directly:
        //   { "ast_type": "Module", ... }
        let module_node = if let Some(ast) = root.get("ast") {
            ast
        } else if root.get("ast_type").is_some() {
            &root
        } else {
            fail!("Invalid Vyper JSON AST: missing 'ast' or 'ast_type' field");
        };

        Self::parse_source_unit(module_node, file_path)
    }

    fn parse_source_unit(node: &Value, file_path: &str) -> Result<SourceUnit> {
        let ast_type = Self::get_ast_type(node)?;
        if ast_type != "Module" {
            fail!("Expected Module node, got: {ast_type}");
        }

        let body_arr = node.get("body").and_then(|v| v.as_array());
        let mut body = Vec::new();

        if let Some(elems) = body_arr {
            for elem in elems {
                if let Some(su_elem) = Self::parse_source_unit_elem(elem)? {
                    body.push(su_elem);
                }
            }
        }

        Ok(SourceUnit { path: file_path.to_string(), body, loc: Self::parse_loc(node) })
    }

    fn parse_source_unit_elem(node: &Value) -> Result<Option<SourceUnitElem>> {
        let ast_type = Self::get_ast_type(node)?;
        match ast_type.as_str() {
            "FunctionDef" => Ok(Some(SourceUnitElem::Func(Self::parse_func_def(node)?))),
            "AnnAssign" | "VariableDecl" => {
                Ok(Some(SourceUnitElem::StateVar(Self::parse_state_var(node)?)))
            }
            "EventDef" => Ok(Some(SourceUnitElem::Event(Self::parse_event_def(node)?))),
            "StructDef" => Ok(Some(SourceUnitElem::Struct(Self::parse_struct_def(node)?))),
            "InterfaceDef" => {
                Ok(Some(SourceUnitElem::Interface(Self::parse_interface_def(node)?)))
            }
            "EnumDef" => Ok(Some(SourceUnitElem::EnumDef(Self::parse_enum_def(node)?))),
            "FlagDef" => Ok(Some(SourceUnitElem::Flag(Self::parse_flag_def(node)?))),
            "Import" | "ImportFrom" => {
                Ok(Some(SourceUnitElem::Import(Self::parse_import_stmt(node)?)))
            }
            // Skip doc strings or expression-only top-level nodes
            "DocStr" | "Expr" => Ok(None),
            other => {
                log::warn!("Unknown top-level AST node type: {other}");
                Ok(None)
            }
        }
    }

    // ─── Definition parsing ──────────────────────────────────────

    fn parse_func_def(node: &Value) -> Result<FuncDef> {
        let name = Self::get_str(node, "name")?;

        // Parse decorators
        let decorators = Self::parse_decorators(node)?;

        // Parse parameters from "args"
        let params = Self::parse_func_params(node)?;

        // Parse return type
        let return_type = Self::parse_return_type(node)?;

        // Parse doc string
        let doc_string = node.get("doc_string").and_then(|v| {
            if v.is_null() {
                None
            } else {
                v.get("value")
                    .and_then(|s| s.as_str().map(|s| s.to_string()))
            }
        });

        // Parse body
        let body = Self::parse_stmt_list(node.get("body"))?;

        Ok(FuncDef {
            name,
            params,
            return_type,
            decorators,
            doc_string,
            body,
            loc: Self::parse_loc(node),
        })
    }

    fn parse_func_params(node: &Value) -> Result<Vec<Param>> {
        let args_node = match node.get("args") {
            Some(args) => args,
            None => return Ok(vec![]),
        };

        let args_array = match args_node.get("args").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Ok(vec![]),
        };

        let mut params = Vec::new();
        for arg in args_array {
            let name = Self::get_str(arg, "arg")?;
            let annotation = arg.get("annotation");
            let typ = match annotation {
                Some(ann) if !ann.is_null() => Self::parse_type(ann)?,
                _ => Type::UInt(UIntType { bits: 256 }),
            };
            let default = match arg.get("default") {
                Some(d) if !d.is_null() => Some(Self::parse_expr(d)?),
                _ => None,
            };
            params.push(Param { name, typ, default, loc: Self::parse_loc(arg) });
        }

        Ok(params)
    }

    fn parse_return_type(node: &Value) -> Result<Option<Type>> {
        // Check "returns" field first (common in Vyper AST)
        if let Some(returns) = node.get("returns") {
            if !returns.is_null() {
                return Ok(Some(Self::parse_type(returns)?));
            }
        }
        // Also check the "return_type" field from args
        if let Some(args) = node.get("args") {
            if let Some(returns) = args.get("returns") {
                if !returns.is_null() {
                    return Ok(Some(Self::parse_type(returns)?));
                }
            }
        }
        Ok(None)
    }

    fn parse_decorators(node: &Value) -> Result<Vec<FuncDecorator>> {
        let dec_list = match node.get("decorator_list").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Ok(vec![]),
        };

        let mut decorators = Vec::new();
        for dec in dec_list {
            decorators.push(Self::parse_decorator(dec)?);
        }
        Ok(decorators)
    }

    fn parse_decorator(node: &Value) -> Result<FuncDecorator> {
        let ast_type = Self::get_ast_type(node)?;
        match ast_type.as_str() {
            "Name" => {
                let id = Self::get_str(node, "id")?;
                match id.as_str() {
                    "deploy" => Ok(FuncDecorator::Deploy),
                    "external" => Ok(FuncDecorator::External),
                    "internal" => Ok(FuncDecorator::Internal),
                    "view" => Ok(FuncDecorator::View),
                    "pure" => Ok(FuncDecorator::Pure),
                    "payable" => Ok(FuncDecorator::Payable),
                    "nonreentrant" => Ok(FuncDecorator::NonReentrant(None)),
                    other => Ok(FuncDecorator::Custom(other.to_string())),
                }
            }
            "Call" => {
                // @nonreentrant("key")
                let func = node.get("func");
                let func_name = func
                    .and_then(|f| f.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if func_name == "nonreentrant" {
                    let key = node
                        .get("args")
                        .and_then(|a| a.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.get("value"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    Ok(FuncDecorator::NonReentrant(key))
                } else {
                    Ok(FuncDecorator::Custom(func_name.to_string()))
                }
            }
            _ => Ok(FuncDecorator::Custom(ast_type)),
        }
    }

    fn parse_state_var(node: &Value) -> Result<StateVarDecl> {
        // AnnAssign has target (Name) and annotation (type)
        let target = node.get("target");
        let name = target
            .and_then(|t| t.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let annotation = node.get("annotation");
        let typ = match annotation {
            Some(ann) if !ann.is_null() => Self::parse_type(ann)?,
            _ => Type::UInt(UIntType { bits: 256 }),
        };

        // Check for constant/immutable
        let constant = node
            .get("is_constant")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let immutable = node
            .get("is_immutable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(StateVarDecl {
            name,
            typ,
            constant,
            immutable,
            doc_string: None,
            loc: Self::parse_loc(node),
        })
    }

    fn parse_event_def(node: &Value) -> Result<EventDef> {
        let name = Self::get_str(node, "name")?;
        let body = node.get("body").and_then(|v| v.as_array());

        let mut fields = Vec::new();
        if let Some(body_arr) = body {
            for field_node in body_arr {
                let ast_type = Self::get_ast_type(field_node)?;
                if ast_type == "AnnAssign" {
                    let field_name = field_node
                        .get("target")
                        .and_then(|t| t.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let annotation = field_node.get("annotation");
                    let (typ, indexed) = Self::parse_event_field_type(annotation)?;

                    fields.push(EventField { name: field_name, typ, indexed });
                }
            }
        }

        Ok(EventDef { name, fields, loc: Self::parse_loc(node) })
    }

    fn parse_event_field_type(annotation: Option<&Value>) -> Result<(Type, bool)> {
        match annotation {
            Some(ann) if !ann.is_null() => {
                let ast_type = Self::get_ast_type(ann)?;
                if ast_type == "Call" {
                    // indexed(type)
                    let func = ann.get("func");
                    let func_name = func
                        .and_then(|f| f.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if func_name == "indexed" {
                        let inner_type = ann
                            .get("args")
                            .and_then(|a| a.as_array())
                            .and_then(|arr| arr.first());
                        match inner_type {
                            Some(t) => Ok((Self::parse_type(t)?, true)),
                            None => Ok((Type::UInt(UIntType { bits: 256 }), true)),
                        }
                    } else {
                        Ok((Self::parse_type(ann)?, false))
                    }
                } else {
                    Ok((Self::parse_type(ann)?, false))
                }
            }
            _ => Ok((Type::UInt(UIntType { bits: 256 }), false)),
        }
    }

    fn parse_struct_def(node: &Value) -> Result<StructDef> {
        let name = Self::get_str(node, "name")?;
        let body = node.get("body").and_then(|v| v.as_array());

        let mut fields = Vec::new();
        if let Some(body_arr) = body {
            for field_node in body_arr {
                let ast_type = Self::get_ast_type(field_node)?;
                if ast_type == "AnnAssign" {
                    let field_name = field_node
                        .get("target")
                        .and_then(|t| t.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let annotation = field_node.get("annotation");
                    let typ = match annotation {
                        Some(ann) if !ann.is_null() => Self::parse_type(ann)?,
                        _ => Type::UInt(UIntType { bits: 256 }),
                    };
                    fields.push(StructField { name: field_name, typ });
                }
            }
        }

        Ok(StructDef { name, fields, loc: Self::parse_loc(node) })
    }

    fn parse_interface_def(node: &Value) -> Result<InterfaceDef> {
        let name = Self::get_str(node, "name")?;
        let body = node.get("body").and_then(|v| v.as_array());

        let mut funcs = Vec::new();
        if let Some(body_arr) = body {
            for func_node in body_arr {
                let ast_type = Self::get_ast_type(func_node)?;
                if ast_type == "FunctionDef" {
                    let func_name = Self::get_str(func_node, "name")?;
                    let params = Self::parse_func_params(func_node)?;
                    let return_type = Self::parse_return_type(func_node)?;

                    // Get mutability from decorators
                    let decorators = Self::parse_decorators(func_node)?;
                    let mutability = decorators.iter().find_map(|d| match d {
                        FuncDecorator::View => Some("view".to_string()),
                        FuncDecorator::Pure => Some("pure".to_string()),
                        FuncDecorator::External => Some("nonpayable".to_string()),
                        FuncDecorator::Payable => Some("payable".to_string()),
                        _ => None,
                    });

                    funcs.push(InterfaceFunc { name: func_name, params, return_type, mutability });
                }
            }
        }

        Ok(InterfaceDef { name, funcs, loc: Self::parse_loc(node) })
    }

    fn parse_enum_def(node: &Value) -> Result<EnumDef> {
        let name = Self::get_str(node, "name")?;
        let body = node.get("body").and_then(|v| v.as_array());
        let variants = Self::parse_enum_variants(body);

        Ok(EnumDef { name, variants, loc: Self::parse_loc(node) })
    }

    fn parse_flag_def(node: &Value) -> Result<FlagDef> {
        let name = Self::get_str(node, "name")?;
        let body = node.get("body").and_then(|v| v.as_array());
        let variants = Self::parse_enum_variants(body);

        Ok(FlagDef { name, variants, loc: Self::parse_loc(node) })
    }

    fn parse_enum_variants(body: Option<&Vec<Value>>) -> Vec<String> {
        let mut variants = Vec::new();
        if let Some(body_arr) = body {
            for variant_node in body_arr {
                if let Some(id) = variant_node.get("id").and_then(|v| v.as_str()) {
                    variants.push(id.to_string());
                } else if let Some(v) = variant_node.get("value").and_then(|v| v.as_str()) {
                    variants.push(v.to_string());
                } else if let Some(name) = variant_node.get("name").and_then(|v| v.as_str()) {
                    variants.push(name.to_string());
                }
            }
        }
        variants
    }

    fn parse_import_stmt(node: &Value) -> Result<ImportStmt> {
        let module = node
            .get("module")
            .or_else(|| node.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let alias = node
            .get("alias")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(ImportStmt { module, alias, loc: Self::parse_loc(node) })
    }

    // ─── Statement parsing ───────────────────────────────────────

    fn parse_stmt_list(body: Option<&Value>) -> Result<Vec<Stmt>> {
        match body.and_then(|v| v.as_array()) {
            Some(arr) => {
                let mut stmts = Vec::new();
                for item in arr {
                    stmts.push(Self::parse_stmt(item)?);
                }
                Ok(stmts)
            }
            None => Ok(vec![]),
        }
    }

    fn parse_stmt(node: &Value) -> Result<Stmt> {
        let ast_type = Self::get_ast_type(node)?;
        match ast_type.as_str() {
            "Assign" => Self::parse_assign_stmt(node),
            "AugAssign" => Self::parse_aug_assign_stmt(node),
            "AnnAssign" => Self::parse_ann_assign_stmt(node),
            "If" => Self::parse_if_stmt(node),
            "For" => Self::parse_for_stmt(node),
            "Return" => Self::parse_return_stmt(node),
            "Assert" => Self::parse_assert_stmt(node),
            "Raise" => Ok(Stmt::Raise(RaiseStmt {
                exc: node
                    .get("exc")
                    .filter(|v| !v.is_null())
                    .map(|v| Self::parse_expr(v))
                    .transpose()?,
                loc: Self::parse_loc(node),
            })),
            "Log" => Self::parse_log_stmt(node),
            "Pass" => Ok(Stmt::Pass(Self::parse_loc(node))),
            "Break" => Ok(Stmt::Break(Self::parse_loc(node))),
            "Continue" => Ok(Stmt::Continue(Self::parse_loc(node))),
            "Expr" => {
                let value = node.get("value");
                match value {
                    Some(v) if !v.is_null() => Ok(Stmt::Expr(ExprStmt {
                        value: Self::parse_expr(v)?,
                        loc: Self::parse_loc(node),
                    })),
                    _ => Ok(Stmt::Pass(Self::parse_loc(node))),
                }
            }
            other => {
                log::warn!("Unknown statement type: {other}");
                Ok(Stmt::Pass(Self::parse_loc(node)))
            }
        }
    }

    fn parse_assign_stmt(node: &Value) -> Result<Stmt> {
        let target = node.get("target").or_else(|| {
            node.get("targets")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
        });
        let value = node.get("value");

        let target_expr = match target {
            Some(t) => Self::parse_expr(t)?,
            None => fail!("Assign missing target"),
        };
        let value_expr = match value {
            Some(v) if !v.is_null() => Self::parse_expr(v)?,
            _ => fail!("Assign missing value"),
        };

        Ok(Stmt::Assign(AssignStmt {
            target: target_expr,
            value: value_expr,
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_aug_assign_stmt(node: &Value) -> Result<Stmt> {
        let target = node.get("target");
        let value = node.get("value");
        let op = Self::parse_binop(node.get("op"))?;

        let target_expr = match target {
            Some(t) => Self::parse_expr(t)?,
            None => fail!("AugAssign missing target"),
        };
        let value_expr = match value {
            Some(v) if !v.is_null() => Self::parse_expr(v)?,
            _ => fail!("AugAssign missing value"),
        };

        Ok(Stmt::AugAssign(AugAssignStmt {
            target: target_expr,
            op,
            value: value_expr,
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_ann_assign_stmt(node: &Value) -> Result<Stmt> {
        let target = node.get("target");
        let annotation = node.get("annotation");
        let value = node.get("value");

        let target_expr = match target {
            Some(t) => Self::parse_expr(t)?,
            None => fail!("AnnAssign missing target"),
        };
        let typ = match annotation {
            Some(ann) if !ann.is_null() => Self::parse_type(ann)?,
            _ => Type::UInt(UIntType { bits: 256 }),
        };
        let value_expr = match value {
            Some(v) if !v.is_null() => Some(Self::parse_expr(v)?),
            _ => None,
        };

        Ok(Stmt::AnnAssign(AnnAssignStmt {
            target: target_expr,
            annotation: typ,
            value: value_expr,
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_if_stmt(node: &Value) -> Result<Stmt> {
        let test = node.get("test");
        let cond = match test {
            Some(t) => Self::parse_expr(t)?,
            None => fail!("If missing test"),
        };

        let then_body = Self::parse_stmt_list(node.get("body"))?;
        let else_body = Self::parse_stmt_list(node.get("orelse"))?;

        Ok(Stmt::If(IfStmt { cond, then_body, else_body, loc: Self::parse_loc(node) }))
    }

    fn parse_for_stmt(node: &Value) -> Result<Stmt> {
        let target = node.get("target");
        let target_expr = match target {
            Some(t) => Self::parse_expr(t)?,
            None => fail!("For missing target"),
        };

        let iter_node = node.get("iter");
        let iter = Self::parse_for_iter(iter_node)?;

        let body = Self::parse_stmt_list(node.get("body"))?;

        Ok(Stmt::For(ForStmt { target: target_expr, iter, body, loc: Self::parse_loc(node) }))
    }

    fn parse_for_iter(node: Option<&Value>) -> Result<ForIter> {
        match node {
            Some(n) => {
                let ast_type = Self::get_ast_type(n)?;
                if ast_type == "Call" {
                    // Check if it's range()
                    let func = n.get("func");
                    let func_name = func
                        .and_then(|f| f.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if func_name == "range" {
                        let args = n.get("args").and_then(|v| v.as_array());
                        match args {
                            Some(arr) if arr.len() >= 2 => Ok(ForIter::Range(RangeIter {
                                start: Some(Box::new(Self::parse_expr(&arr[0])?)),
                                stop: Box::new(Self::parse_expr(&arr[1])?),
                            })),
                            Some(arr) if arr.len() == 1 => Ok(ForIter::Range(RangeIter {
                                start: None,
                                stop: Box::new(Self::parse_expr(&arr[0])?),
                            })),
                            _ => Ok(ForIter::Range(RangeIter {
                                start: None,
                                stop: Box::new(Expr::Lit(Lit {
                                    kind: LitKind::Int(0),
                                    loc: None,
                                })),
                            })),
                        }
                    } else {
                        Ok(ForIter::Iterable(Box::new(Self::parse_expr(n)?)))
                    }
                } else {
                    Ok(ForIter::Iterable(Box::new(Self::parse_expr(n)?)))
                }
            }
            None => fail!("For missing iter"),
        }
    }

    fn parse_return_stmt(node: &Value) -> Result<Stmt> {
        let value = node.get("value");
        let expr = match value {
            Some(v) if !v.is_null() => Some(Self::parse_expr(v)?),
            _ => None,
        };

        Ok(Stmt::Return(ReturnStmt { value: expr, loc: Self::parse_loc(node) }))
    }

    fn parse_assert_stmt(node: &Value) -> Result<Stmt> {
        let test = node.get("test");
        let cond = match test {
            Some(t) => Self::parse_expr(t)?,
            None => fail!("Assert missing test"),
        };

        let msg = node
            .get("msg")
            .filter(|v| !v.is_null())
            .map(|v| Self::parse_expr(v))
            .transpose()?;

        Ok(Stmt::Assert(AssertStmt { test: cond, msg, loc: Self::parse_loc(node) }))
    }

    fn parse_log_stmt(node: &Value) -> Result<Stmt> {
        let value = node.get("value");
        let event_expr = match value {
            Some(v) if !v.is_null() => Self::parse_expr(v)?,
            _ => {
                // Some Vyper versions put the call directly in the Log node
                let body_node = node.get("body").or_else(|| node.get("call"));
                match body_node {
                    Some(b) if !b.is_null() => Self::parse_expr(b)?,
                    _ => {
                        // Construct from event name + args fields
                        let event_name = node
                            .get("event")
                            .and_then(|v| v.as_str())
                            .unwrap_or("UnknownEvent");
                        Expr::Call(CallExpr {
                            func: Box::new(Expr::Ident(Identifier {
                                name: event_name.to_string(),
                                loc: None,
                            })),
                            args: vec![],
                            keywords: vec![],
                            loc: Self::parse_loc(node),
                        })
                    }
                }
            }
        };

        Ok(Stmt::Log(LogStmt { event: event_expr, loc: Self::parse_loc(node) }))
    }

    // ─── Expression parsing ──────────────────────────────────────

    pub fn parse_expr(node: &Value) -> Result<Expr> {
        let ast_type = Self::get_ast_type(node)?;
        match ast_type.as_str() {
            "Name" | "NameConstant" => Self::parse_name_expr(node),
            "Attribute" => Self::parse_attribute_expr(node),
            "Subscript" => Self::parse_subscript_expr(node),
            "Call" => Self::parse_call_expr(node),
            "BinOp" => Self::parse_binop_expr(node),
            "BoolOp" => Self::parse_boolop_expr(node),
            "Compare" => Self::parse_compare_expr(node),
            "UnaryOp" => Self::parse_unaryop_expr(node),
            "Constant" | "Num" | "Str" | "Bytes" => Self::parse_constant_expr(node),
            "Tuple" => Self::parse_tuple_expr(node),
            "IfExp" => Self::parse_ifexp_expr(node),
            "Int" => Self::parse_constant_expr(node),
            "List" => Self::parse_list_expr(node),
            other => {
                log::warn!("Unknown expression type: {other}");
                Ok(Expr::Ident(Identifier {
                    name: format!("__unknown_{other}__"),
                    loc: Self::parse_loc(node),
                }))
            }
        }
    }

    fn parse_name_expr(node: &Value) -> Result<Expr> {
        let id = node
            .get("id")
            .and_then(|v| v.as_str())
            .or_else(|| node.get("value").and_then(|v| v.as_str()));

        match id {
            Some("True") => {
                Ok(Expr::Lit(Lit { kind: LitKind::Bool(true), loc: Self::parse_loc(node) }))
            }
            Some("False") => {
                Ok(Expr::Lit(Lit { kind: LitKind::Bool(false), loc: Self::parse_loc(node) }))
            }
            Some(name) => {
                Ok(Expr::Ident(Identifier { name: name.to_string(), loc: Self::parse_loc(node) }))
            }
            None => {
                // Check for boolean value field
                if let Some(val) = node.get("value") {
                    if let Some(b) = val.as_bool() {
                        return Ok(Expr::Lit(Lit {
                            kind: LitKind::Bool(b),
                            loc: Self::parse_loc(node),
                        }));
                    }
                }
                Ok(Expr::Ident(Identifier {
                    name: "__unnamed__".to_string(),
                    loc: Self::parse_loc(node),
                }))
            }
        }
    }

    fn parse_attribute_expr(node: &Value) -> Result<Expr> {
        let value = node.get("value");
        let attr = Self::get_str(node, "attr")?;

        let value_expr = match value {
            Some(v) if !v.is_null() => Self::parse_expr(v)?,
            _ => Expr::Ident(Identifier { name: "self".to_string(), loc: None }),
        };

        Ok(Expr::Attribute(AttributeExpr {
            value: Box::new(value_expr),
            attr,
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_subscript_expr(node: &Value) -> Result<Expr> {
        let value = node.get("value");
        let slice = node.get("slice");

        let value_expr = match value {
            Some(v) => Self::parse_expr(v)?,
            None => fail!("Subscript missing value"),
        };
        let index_expr = match slice {
            Some(s) if !s.is_null() => Self::parse_expr(s)?,
            _ => fail!("Subscript missing slice"),
        };

        Ok(Expr::Subscript(SubscriptExpr {
            value: Box::new(value_expr),
            index: Box::new(index_expr),
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_call_expr(node: &Value) -> Result<Expr> {
        let func = node.get("func");
        let args = node.get("args").and_then(|v| v.as_array());
        let keywords = node.get("keywords").and_then(|v| v.as_array());

        let func_expr = match func {
            Some(f) => Self::parse_expr(f)?,
            None => fail!("Call missing func"),
        };

        let mut arg_exprs = Vec::new();
        if let Some(arg_arr) = args {
            for arg in arg_arr {
                arg_exprs.push(Self::parse_expr(arg)?);
            }
        }

        let mut kw_list = Vec::new();
        if let Some(kw_arr) = keywords {
            for kw in kw_arr {
                let name = Self::get_str(kw, "arg").unwrap_or_default();
                let value = kw.get("value");
                match value {
                    Some(v) if !v.is_null() => {
                        kw_list.push(Keyword { name, value: Self::parse_expr(v)? });
                    }
                    _ => {}
                }
            }
        }

        // Detect builtins
        let loc = Self::parse_loc(node);
        match &func_expr {
            Expr::Ident(id) => match id.name.as_str() {
                "convert" => {
                    if arg_exprs.len() >= 2 {
                        let to_type = Self::expr_to_type(&arg_exprs[1])?;
                        return Ok(Expr::Convert {
                            expr: Box::new(arg_exprs.remove(0)),
                            to: to_type,
                            loc,
                        });
                    }
                }
                "empty" => {
                    if arg_exprs.len() == 1 {
                        let ty = Self::expr_to_type(&arg_exprs[0])?;
                        return Ok(Expr::Empty(ty, loc));
                    }
                }
                "len" => {
                    if arg_exprs.len() == 1 {
                        return Ok(Expr::Len(Box::new(arg_exprs.remove(0)), loc));
                    }
                }
                "concat" => {
                    return Ok(Expr::Concat(arg_exprs, loc));
                }
                "slice" => {
                    if arg_exprs.len() >= 3 {
                        let length = arg_exprs.remove(2);
                        let start = arg_exprs.remove(1);
                        let expr = arg_exprs.remove(0);
                        return Ok(Expr::Slice {
                            expr: Box::new(expr),
                            start: Box::new(start),
                            length: Box::new(length),
                            loc,
                        });
                    }
                }
                "raw_call" => {
                    if arg_exprs.len() >= 2 {
                        let data = arg_exprs.remove(1);
                        let target = arg_exprs.remove(0);
                        let value = kw_list
                            .iter()
                            .find(|k| k.name == "value")
                            .map(|k| Box::new(k.value.clone()));
                        let gas = kw_list
                            .iter()
                            .find(|k| k.name == "gas" || k.name == "max_outsize")
                            .map(|k| Box::new(k.value.clone()));
                        return Ok(Expr::RawCall {
                            target: Box::new(target),
                            data: Box::new(data),
                            value,
                            gas,
                            loc,
                        });
                    }
                }
                "send" => {
                    if arg_exprs.len() >= 2 {
                        let value = arg_exprs.remove(1);
                        let target = arg_exprs.remove(0);
                        return Ok(Expr::Send {
                            target: Box::new(target),
                            value: Box::new(value),
                            loc,
                        });
                    }
                }
                "keccak256" => {
                    if arg_exprs.len() == 1 {
                        return Ok(Expr::Keccak256(Box::new(arg_exprs.remove(0)), loc));
                    }
                }
                "sha256" => {
                    if arg_exprs.len() == 1 {
                        return Ok(Expr::Sha256(Box::new(arg_exprs.remove(0)), loc));
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Ok(Expr::Call(CallExpr {
            func: Box::new(func_expr),
            args: arg_exprs,
            keywords: kw_list,
            loc,
        }))
    }

    fn parse_binop_expr(node: &Value) -> Result<Expr> {
        let left = node.get("left");
        let right = node.get("right");
        let op = Self::parse_binop(node.get("op"))?;

        let left_expr = match left {
            Some(l) => Self::parse_expr(l)?,
            None => fail!("BinOp missing left"),
        };
        let right_expr = match right {
            Some(r) => Self::parse_expr(r)?,
            None => fail!("BinOp missing right"),
        };

        Ok(Expr::BinOp(BinOpExpr {
            left: Box::new(left_expr),
            op,
            right: Box::new(right_expr),
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_boolop_expr(node: &Value) -> Result<Expr> {
        let op = node.get("op");
        let values = node.get("values").and_then(|v| v.as_array());

        let bool_op = match op {
            Some(o) => {
                let ast_type = Self::get_ast_type(o)?;
                match ast_type.as_str() {
                    "And" => BoolOp::And,
                    "Or" => BoolOp::Or,
                    other => {
                        log::warn!("Unknown BoolOp: {other}");
                        BoolOp::And
                    }
                }
            }
            None => {
                // Check if op is a string
                let op_str = node.get("op").and_then(|v| v.as_str()).unwrap_or("And");
                match op_str {
                    "Or" => BoolOp::Or,
                    _ => BoolOp::And,
                }
            }
        };

        let mut value_exprs = Vec::new();
        if let Some(arr) = values {
            for v in arr {
                value_exprs.push(Self::parse_expr(v)?);
            }
        }

        Ok(Expr::BoolOp(BoolOpExpr {
            op: bool_op,
            values: value_exprs,
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_compare_expr(node: &Value) -> Result<Expr> {
        let left = node.get("left");
        let ops = node.get("ops").and_then(|v| v.as_array());
        let comparators = node.get("comparators").and_then(|v| v.as_array());

        let left_expr = match left {
            Some(l) => Self::parse_expr(l)?,
            None => fail!("Compare missing left"),
        };

        let mut cmp_ops = Vec::new();
        if let Some(arr) = ops {
            for o in arr {
                cmp_ops.push(Self::parse_cmpop(o)?);
            }
        }

        let mut cmp_exprs = Vec::new();
        if let Some(arr) = comparators {
            for c in arr {
                cmp_exprs.push(Self::parse_expr(c)?);
            }
        }

        Ok(Expr::Compare(CompareExpr {
            left: Box::new(left_expr),
            ops: cmp_ops,
            comparators: cmp_exprs,
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_unaryop_expr(node: &Value) -> Result<Expr> {
        let op = node.get("op");
        let operand = node.get("operand");

        let unary_op = match op {
            Some(o) => {
                let ast_type = Self::get_ast_type(o)?;
                match ast_type.as_str() {
                    "Not" => UnaryOp::Not,
                    "USub" | "Neg" => UnaryOp::Neg,
                    "Invert" => UnaryOp::Invert,
                    other => {
                        log::warn!("Unknown UnaryOp: {other}");
                        UnaryOp::Not
                    }
                }
            }
            None => {
                let op_str = node.get("op").and_then(|v| v.as_str()).unwrap_or("Not");
                match op_str {
                    "USub" | "Neg" => UnaryOp::Neg,
                    "Invert" => UnaryOp::Invert,
                    _ => UnaryOp::Not,
                }
            }
        };

        let operand_expr = match operand {
            Some(o) => Self::parse_expr(o)?,
            None => fail!("UnaryOp missing operand"),
        };

        Ok(Expr::UnaryOp(UnaryOpExpr {
            op: unary_op,
            operand: Box::new(operand_expr),
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_constant_expr(node: &Value) -> Result<Expr> {
        let loc = Self::parse_loc(node);

        // Try "value" field first
        if let Some(val) = node.get("value") {
            if val.is_boolean() {
                return Ok(Expr::Lit(Lit { kind: LitKind::Bool(val.as_bool().unwrap()), loc }));
            }
            if val.is_i64() {
                return Ok(Expr::Lit(Lit {
                    kind: LitKind::Int(val.as_i64().unwrap() as i128),
                    loc,
                }));
            }
            if val.is_u64() {
                return Ok(Expr::Lit(Lit {
                    kind: LitKind::Int(val.as_u64().unwrap() as i128),
                    loc,
                }));
            }
            if val.is_f64() {
                // Approximate: treat as integer for now
                return Ok(Expr::Lit(Lit {
                    kind: LitKind::Int(val.as_f64().unwrap() as i128),
                    loc,
                }));
            }
            if let Some(s) = val.as_str() {
                if s.starts_with("0x") || s.starts_with("0X") {
                    return Ok(Expr::Lit(Lit { kind: LitKind::Hex(s[2..].to_string()), loc }));
                }
                return Ok(Expr::Lit(Lit { kind: LitKind::Str(s.to_string()), loc }));
            }
            if val.is_null() {
                return Ok(Expr::Lit(Lit { kind: LitKind::Int(0), loc }));
            }
        }

        // Fall through: try the node itself
        if let Some(n) = node.get("n") {
            if let Some(i) = n.as_i64() {
                return Ok(Expr::Lit(Lit { kind: LitKind::Int(i as i128), loc }));
            }
        }

        Ok(Expr::Lit(Lit { kind: LitKind::Int(0), loc }))
    }

    fn parse_tuple_expr(node: &Value) -> Result<Expr> {
        let elts = node.get("elts").or_else(|| node.get("elements"));
        let mut exprs = Vec::new();
        if let Some(arr) = elts.and_then(|v| v.as_array()) {
            for e in arr {
                exprs.push(Self::parse_expr(e)?);
            }
        }
        Ok(Expr::Tuple(exprs, Self::parse_loc(node)))
    }

    fn parse_ifexp_expr(node: &Value) -> Result<Expr> {
        let test = node.get("test");
        let body = node.get("body");
        let orelse = node.get("orelse");

        let test_expr = match test {
            Some(t) => Self::parse_expr(t)?,
            None => fail!("IfExp missing test"),
        };
        let body_expr = match body {
            Some(b) => Self::parse_expr(b)?,
            None => fail!("IfExp missing body"),
        };
        let orelse_expr = match orelse {
            Some(o) => Self::parse_expr(o)?,
            None => fail!("IfExp missing orelse"),
        };

        Ok(Expr::IfExp(IfExpExpr {
            test: Box::new(test_expr),
            body: Box::new(body_expr),
            orelse: Box::new(orelse_expr),
            loc: Self::parse_loc(node),
        }))
    }

    fn parse_list_expr(node: &Value) -> Result<Expr> {
        let elts = node.get("elts").or_else(|| node.get("elements"));
        let mut exprs = Vec::new();
        if let Some(arr) = elts.and_then(|v| v.as_array()) {
            for e in arr {
                exprs.push(Self::parse_expr(e)?);
            }
        }
        Ok(Expr::Tuple(exprs, Self::parse_loc(node)))
    }

    // ─── Type parsing ────────────────────────────────────────────

    pub fn parse_type(node: &Value) -> Result<Type> {
        let ast_type = Self::get_ast_type(node)?;
        match ast_type.as_str() {
            "Name" => Self::parse_type_name(node),
            "Subscript" => Self::parse_type_subscript(node),
            "Attribute" => {
                let attr = Self::get_str(node, "attr")?;
                Ok(Type::Interface(attr))
            }
            "Call" => {
                // public(type) or indexed(type)
                let func = node.get("func");
                let func_name = func
                    .and_then(|f| f.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                match func_name {
                    "public" => {
                        let inner = node
                            .get("args")
                            .and_then(|a| a.as_array())
                            .and_then(|arr| arr.first());
                        match inner {
                            Some(t) => Ok(Type::Public(Box::new(Self::parse_type(t)?))),
                            None => Ok(Type::Public(Box::new(Type::UInt(UIntType { bits: 256 })))),
                        }
                    }
                    "indexed" => {
                        let inner = node
                            .get("args")
                            .and_then(|a| a.as_array())
                            .and_then(|arr| arr.first());
                        match inner {
                            Some(t) => Self::parse_type(t),
                            None => Ok(Type::UInt(UIntType { bits: 256 })),
                        }
                    }
                    _ => {
                        // Treat as interface call (e.g., IERC20(_token))
                        Ok(Type::Interface(func_name.to_string()))
                    }
                }
            }
            other => {
                log::warn!("Unknown type node: {other}");
                Ok(Type::UInt(UIntType { bits: 256 }))
            }
        }
    }

    fn parse_type_name(node: &Value) -> Result<Type> {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("uint256");
        Self::parse_type_from_name(id)
    }

    fn parse_type_from_name(name: &str) -> Result<Type> {
        match name {
            "bool" => Ok(Type::Bool),
            "address" => Ok(Type::Address),
            "bytes32" => Ok(Type::Bytes32),
            "decimal" => Ok(Type::Decimal),
            s if s.starts_with("uint") => {
                let bits: u16 = s[4..].parse().unwrap_or(256);
                Ok(Type::UInt(UIntType { bits }))
            }
            s if s.starts_with("int") => {
                let bits: u16 = s[3..].parse().unwrap_or(256);
                Ok(Type::Int(IntType { bits }))
            }
            s if s.starts_with("bytes") => {
                let n: u64 = s[5..].parse().unwrap_or(32);
                if n <= 32 {
                    Ok(Type::Bytes32) // fixed bytes
                } else {
                    Ok(Type::BoundedBytes(n))
                }
            }
            other => {
                // Could be a struct, enum, or interface name
                Ok(Type::Struct(other.to_string()))
            }
        }
    }

    fn parse_type_subscript(node: &Value) -> Result<Type> {
        let value = node.get("value");
        let slice = node.get("slice");

        let base_name = value
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match base_name {
            "HashMap" => {
                // HashMap[K, V]
                let slice_node = slice;
                let (key_type, val_type) = Self::parse_map_type_args(slice_node)?;
                Ok(Type::HashMap(Box::new(key_type), Box::new(val_type)))
            }
            "DynArray" => {
                // DynArray[T, N]
                let (elem_type, max_len) = Self::parse_dynarray_args(slice)?;
                Ok(Type::DynArray { elem: Box::new(elem_type), max_len })
            }
            "String" => {
                // String[N]
                let n = Self::parse_size_arg(slice)?;
                Ok(Type::BoundedString(n))
            }
            "Bytes" => {
                // Bytes[N]
                let n = Self::parse_size_arg(slice)?;
                Ok(Type::BoundedBytes(n))
            }
            _ => {
                // T[N] — fixed array, or a type subscript
                let base_type = match value {
                    Some(v) => Self::parse_type(v)?,
                    None => Type::UInt(UIntType { bits: 256 }),
                };
                let len = Self::parse_size_arg(slice)?;
                Ok(Type::FixedArray { elem: Box::new(base_type), len })
            }
        }
    }

    fn parse_map_type_args(slice: Option<&Value>) -> Result<(Type, Type)> {
        match slice {
            Some(s) => {
                let ast_type = Self::get_ast_type(s).unwrap_or_default();
                if ast_type == "Tuple" || ast_type == "Index" {
                    let elts = s.get("elts").or_else(|| s.get("value"));
                    if let Some(arr) = elts.and_then(|v| v.as_array()) {
                        if arr.len() >= 2 {
                            let k = Self::parse_type(&arr[0])?;
                            let v = Self::parse_type(&arr[1])?;
                            return Ok((k, v));
                        }
                    }
                }
                // Try as a single Index node with value being a Tuple
                if let Some(value) = s.get("value") {
                    if let Some(arr) = value.as_array() {
                        if arr.len() >= 2 {
                            let k = Self::parse_type(&arr[0])?;
                            let v = Self::parse_type(&arr[1])?;
                            return Ok((k, v));
                        }
                    }
                }
                // Fallback: slice has two sub-nodes
                if let Some(arr) = s.as_array() {
                    if arr.len() >= 2 {
                        let k = Self::parse_type(&arr[0])?;
                        let v = Self::parse_type(&arr[1])?;
                        return Ok((k, v));
                    }
                }
                Ok((Type::Address, Type::UInt(UIntType { bits: 256 })))
            }
            None => Ok((Type::Address, Type::UInt(UIntType { bits: 256 }))),
        }
    }

    fn parse_dynarray_args(slice: Option<&Value>) -> Result<(Type, u64)> {
        match slice {
            Some(s) => {
                let ast_type = Self::get_ast_type(s).unwrap_or_default();
                if ast_type == "Tuple" || ast_type == "Index" {
                    let elts = s.get("elts").or_else(|| s.get("value"));
                    if let Some(arr) = elts.and_then(|v| v.as_array()) {
                        if arr.len() >= 2 {
                            let elem = Self::parse_type(&arr[0])?;
                            let max_len = Self::extract_int_value(&arr[1]).unwrap_or(256);
                            return Ok((elem, max_len));
                        }
                    }
                }
                Ok((Type::UInt(UIntType { bits: 256 }), 256))
            }
            None => Ok((Type::UInt(UIntType { bits: 256 }), 256)),
        }
    }

    fn parse_size_arg(slice: Option<&Value>) -> Result<u64> {
        match slice {
            Some(s) => {
                if let Some(n) = Self::extract_int_value(s) {
                    return Ok(n);
                }
                // Try parsing as a constant
                if let Some(val) = s.get("value") {
                    if let Some(n) = val.as_u64() {
                        return Ok(n);
                    }
                    if let Some(n) = val.as_i64() {
                        return Ok(n as u64);
                    }
                }
                if let Some(n) = s.get("n").and_then(|v| v.as_u64()) {
                    return Ok(n);
                }
                Ok(256)
            }
            None => Ok(256),
        }
    }

    fn extract_int_value(node: &Value) -> Option<u64> {
        if let Some(v) = node.as_u64() {
            return Some(v);
        }
        if let Some(v) = node.as_i64() {
            return Some(v as u64);
        }
        if let Some(v) = node.get("value") {
            if let Some(n) = v.as_u64() {
                return Some(n);
            }
            if let Some(n) = v.as_i64() {
                return Some(n as u64);
            }
        }
        if let Some(v) = node.get("n") {
            if let Some(n) = v.as_u64() {
                return Some(n);
            }
        }
        None
    }

    /// Try to convert an expression (used as a type argument) to a Type.
    fn expr_to_type(expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Ident(id) => Self::parse_type_from_name(&id.name),
            Expr::Subscript(sub) => {
                // Reconstruct as a type
                if let Expr::Ident(id) = sub.value.as_ref() {
                    match id.name.as_str() {
                        "HashMap" => {
                            let key = Self::expr_to_type(&sub.index)?;
                            Ok(Type::HashMap(
                                Box::new(key),
                                Box::new(Type::UInt(UIntType { bits: 256 })),
                            ))
                        }
                        _ => Ok(Type::Struct(id.name.clone())),
                    }
                } else {
                    Ok(Type::UInt(UIntType { bits: 256 }))
                }
            }
            _ => Ok(Type::UInt(UIntType { bits: 256 })),
        }
    }

    // ─── Operator parsing ────────────────────────────────────────

    fn parse_binop(node: Option<&Value>) -> Result<BinOp> {
        match node {
            Some(n) => {
                let ast_type = Self::get_ast_type(n).unwrap_or_default();
                match ast_type.as_str() {
                    "Add" => Ok(BinOp::Add),
                    "Sub" => Ok(BinOp::Sub),
                    "Mul" | "Mult" => Ok(BinOp::Mul),
                    "Div" => Ok(BinOp::Div),
                    "FloorDiv" => Ok(BinOp::FloorDiv),
                    "Mod" => Ok(BinOp::Mod),
                    "Pow" => Ok(BinOp::Pow),
                    "BitAnd" => Ok(BinOp::BitAnd),
                    "BitOr" => Ok(BinOp::BitOr),
                    "BitXor" => Ok(BinOp::BitXor),
                    "LShift" | "Shl" => Ok(BinOp::Shl),
                    "RShift" | "Shr" => Ok(BinOp::Shr),
                    _ => {
                        // Try string value
                        if let Some(s) = n.as_str() {
                            return Self::parse_binop_str(s);
                        }
                        log::warn!("Unknown BinOp: {ast_type}");
                        Ok(BinOp::Add)
                    }
                }
            }
            None => Ok(BinOp::Add),
        }
    }

    fn parse_binop_str(s: &str) -> Result<BinOp> {
        match s {
            "Add" => Ok(BinOp::Add),
            "Sub" => Ok(BinOp::Sub),
            "Mul" | "Mult" => Ok(BinOp::Mul),
            "Div" => Ok(BinOp::Div),
            "FloorDiv" => Ok(BinOp::FloorDiv),
            "Mod" => Ok(BinOp::Mod),
            "Pow" => Ok(BinOp::Pow),
            "BitAnd" => Ok(BinOp::BitAnd),
            "BitOr" => Ok(BinOp::BitOr),
            "BitXor" => Ok(BinOp::BitXor),
            "LShift" | "Shl" => Ok(BinOp::Shl),
            "RShift" | "Shr" => Ok(BinOp::Shr),
            _ => Ok(BinOp::Add),
        }
    }

    fn parse_cmpop(node: &Value) -> Result<CmpOp> {
        let ast_type = Self::get_ast_type(node).unwrap_or_default();
        match ast_type.as_str() {
            "Eq" => Ok(CmpOp::Eq),
            "NotEq" => Ok(CmpOp::NotEq),
            "Lt" => Ok(CmpOp::Lt),
            "LtE" => Ok(CmpOp::LtE),
            "Gt" => Ok(CmpOp::Gt),
            "GtE" => Ok(CmpOp::GtE),
            "In" => Ok(CmpOp::In),
            "NotIn" => Ok(CmpOp::NotIn),
            _ => {
                if let Some(s) = node.as_str() {
                    match s {
                        "Eq" => return Ok(CmpOp::Eq),
                        "NotEq" => return Ok(CmpOp::NotEq),
                        "Lt" => return Ok(CmpOp::Lt),
                        "LtE" => return Ok(CmpOp::LtE),
                        "Gt" => return Ok(CmpOp::Gt),
                        "GtE" => return Ok(CmpOp::GtE),
                        "In" => return Ok(CmpOp::In),
                        "NotIn" => return Ok(CmpOp::NotIn),
                        _ => {}
                    }
                }
                log::warn!("Unknown CmpOp: {ast_type}");
                Ok(CmpOp::Eq)
            }
        }
    }

    // ─── Utility methods ─────────────────────────────────────────

    fn get_ast_type(node: &Value) -> Result<String> {
        node.get("ast_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| common::error::create_error("Missing 'ast_type' field"))
    }

    fn get_str(node: &Value, field: &str) -> Result<String> {
        node.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| common::error::create_error(format!("Missing '{field}' field")))
    }

    fn parse_loc(node: &Value) -> Option<Loc> {
        let lineno = node.get("lineno")?.as_u64()? as u32;
        let col_offset = node.get("col_offset").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let end_lineno = node
            .get("end_lineno")
            .and_then(|v| v.as_u64())
            .unwrap_or(lineno as u64) as u32;
        let end_col_offset = node
            .get("end_col_offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        Some(Loc { lineno, col_offset, end_lineno, end_col_offset })
    }
}

// ─── Unit tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_loc() {
        let node = json!({
            "lineno": 5,
            "col_offset": 4,
            "end_lineno": 7,
            "end_col_offset": 10
        });
        let loc = AstParser::parse_loc(&node).unwrap();
        assert_eq!(loc.lineno, 5);
        assert_eq!(loc.col_offset, 4);
        assert_eq!(loc.end_lineno, 7);
        assert_eq!(loc.end_col_offset, 10);
    }

    #[test]
    fn test_parse_type_bool() {
        let node = json!({"ast_type": "Name", "id": "bool"});
        let typ = AstParser::parse_type(&node).unwrap();
        assert_eq!(typ, Type::Bool);
    }

    #[test]
    fn test_parse_type_address() {
        let node = json!({"ast_type": "Name", "id": "address"});
        let typ = AstParser::parse_type(&node).unwrap();
        assert_eq!(typ, Type::Address);
    }

    #[test]
    fn test_parse_type_uint256() {
        let node = json!({"ast_type": "Name", "id": "uint256"});
        let typ = AstParser::parse_type(&node).unwrap();
        assert_eq!(typ, Type::UInt(UIntType { bits: 256 }));
    }

    #[test]
    fn test_parse_type_int128() {
        let node = json!({"ast_type": "Name", "id": "int128"});
        let typ = AstParser::parse_type(&node).unwrap();
        assert_eq!(typ, Type::Int(IntType { bits: 128 }));
    }

    #[test]
    fn test_parse_type_bounded_string() {
        let node = json!({
            "ast_type": "Subscript",
            "value": {"ast_type": "Name", "id": "String"},
            "slice": {"ast_type": "Constant", "value": 32}
        });
        let typ = AstParser::parse_type(&node).unwrap();
        assert_eq!(typ, Type::BoundedString(32));
    }

    #[test]
    fn test_parse_type_bounded_bytes() {
        let node = json!({
            "ast_type": "Subscript",
            "value": {"ast_type": "Name", "id": "Bytes"},
            "slice": {"ast_type": "Constant", "value": 100}
        });
        let typ = AstParser::parse_type(&node).unwrap();
        assert_eq!(typ, Type::BoundedBytes(100));
    }

    #[test]
    fn test_parse_name_expr() {
        let node = json!({"ast_type": "Name", "id": "x", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 1});
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Ident(id) => assert_eq!(id.name, "x"),
            _ => panic!("Expected Ident"),
        }
    }

    #[test]
    fn test_parse_constant_int() {
        let node = json!({"ast_type": "Constant", "value": 42, "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 2});
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Lit(lit) => assert_eq!(lit.kind, LitKind::Int(42)),
            _ => panic!("Expected Lit"),
        }
    }

    #[test]
    fn test_parse_constant_bool_true() {
        let node = json!({"ast_type": "Name", "id": "True"});
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Lit(lit) => assert_eq!(lit.kind, LitKind::Bool(true)),
            _ => panic!("Expected Lit(Bool(true))"),
        }
    }

    #[test]
    fn test_parse_constant_string() {
        let node = json!({"ast_type": "Constant", "value": "hello", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 7});
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Lit(lit) => assert_eq!(lit.kind, LitKind::Str("hello".to_string())),
            _ => panic!("Expected Lit(Str)"),
        }
    }

    #[test]
    fn test_parse_attribute_expr() {
        let node = json!({
            "ast_type": "Attribute",
            "value": {"ast_type": "Name", "id": "self"},
            "attr": "balance",
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 12
        });
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Attribute(e) => {
                assert_eq!(e.attr, "balance");
                match *e.value {
                    Expr::Ident(id) => assert_eq!(id.name, "self"),
                    _ => panic!("Expected Ident for value"),
                }
            }
            _ => panic!("Expected Attribute"),
        }
    }

    #[test]
    fn test_parse_binop_expr() {
        let node = json!({
            "ast_type": "BinOp",
            "left": {"ast_type": "Name", "id": "a"},
            "right": {"ast_type": "Constant", "value": 1},
            "op": {"ast_type": "Add"},
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 5
        });
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::BinOp(e) => {
                assert_eq!(e.op, BinOp::Add);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_parse_compare_expr() {
        let node = json!({
            "ast_type": "Compare",
            "left": {"ast_type": "Name", "id": "x"},
            "ops": [{"ast_type": "GtE"}],
            "comparators": [{"ast_type": "Constant", "value": 0}],
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 5
        });
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Compare(e) => {
                assert_eq!(e.ops.len(), 1);
                assert_eq!(e.ops[0], CmpOp::GtE);
            }
            _ => panic!("Expected Compare"),
        }
    }

    #[test]
    fn test_parse_call_convert() {
        let node = json!({
            "ast_type": "Call",
            "func": {"ast_type": "Name", "id": "convert"},
            "args": [
                {"ast_type": "Name", "id": "x"},
                {"ast_type": "Name", "id": "uint256"}
            ],
            "keywords": [],
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 20
        });
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Convert { to, .. } => {
                assert_eq!(to, Type::UInt(UIntType { bits: 256 }));
            }
            _ => panic!("Expected Convert"),
        }
    }

    #[test]
    fn test_parse_call_empty() {
        let node = json!({
            "ast_type": "Call",
            "func": {"ast_type": "Name", "id": "empty"},
            "args": [{"ast_type": "Name", "id": "address"}],
            "keywords": [],
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 15
        });
        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Empty(ty, _) => assert_eq!(ty, Type::Address),
            _ => panic!("Expected Empty"),
        }
    }

    #[test]
    fn test_parse_decorator_external() {
        let node = json!({"ast_type": "Name", "id": "external"});
        let dec = AstParser::parse_decorator(&node).unwrap();
        assert_eq!(dec, FuncDecorator::External);
    }

    #[test]
    fn test_parse_decorator_nonreentrant() {
        let node = json!({
            "ast_type": "Call",
            "func": {"ast_type": "Name", "id": "nonreentrant"},
            "args": [{"ast_type": "Constant", "value": "lock"}],
            "keywords": []
        });
        let dec = AstParser::parse_decorator(&node).unwrap();
        assert_eq!(dec, FuncDecorator::NonReentrant(Some("lock".to_string())));
    }

    #[test]
    fn test_parse_simple_module() {
        let json_str = r#"{
            "ast_type": "Module",
            "body": [
                {
                    "ast_type": "AnnAssign",
                    "target": {"ast_type": "Name", "id": "owner"},
                    "annotation": {"ast_type": "Name", "id": "address"},
                    "value": null,
                    "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 20
                },
                {
                    "ast_type": "FunctionDef",
                    "name": "__init__",
                    "args": {"args": [], "returns": null},
                    "decorator_list": [{"ast_type": "Name", "id": "deploy"}],
                    "body": [
                        {
                            "ast_type": "Assign",
                            "target": {
                                "ast_type": "Attribute",
                                "value": {"ast_type": "Name", "id": "self"},
                                "attr": "owner",
                                "lineno": 5, "col_offset": 4, "end_lineno": 5, "end_col_offset": 14
                            },
                            "value": {
                                "ast_type": "Attribute",
                                "value": {"ast_type": "Name", "id": "msg"},
                                "attr": "sender",
                                "lineno": 5, "col_offset": 17, "end_lineno": 5, "end_col_offset": 27
                            },
                            "lineno": 5, "col_offset": 4, "end_lineno": 5, "end_col_offset": 27
                        }
                    ],
                    "returns": null,
                    "doc_string": null,
                    "lineno": 3, "col_offset": 0, "end_lineno": 6, "end_col_offset": 0
                }
            ],
            "lineno": 1, "col_offset": 0, "end_lineno": 6, "end_col_offset": 0
        }"#;

        let su = AstParser::parse(json_str, "test.vy").unwrap();
        assert_eq!(su.path, "test.vy");
        assert_eq!(su.body.len(), 2);

        // First elem is state var
        match &su.body[0] {
            SourceUnitElem::StateVar(sv) => {
                assert_eq!(sv.name, "owner");
                assert_eq!(sv.typ, Type::Address);
            }
            _ => panic!("Expected StateVar"),
        }

        // Second elem is function
        match &su.body[1] {
            SourceUnitElem::Func(f) => {
                assert_eq!(f.name, "__init__");
                assert_eq!(f.decorators.len(), 1);
                assert_eq!(f.decorators[0], FuncDecorator::Deploy);
                assert_eq!(f.body.len(), 1);
            }
            _ => panic!("Expected Func"),
        }
    }

    #[test]
    fn test_parse_event_def() {
        let json_str = r#"{
            "ast_type": "Module",
            "body": [
                {
                    "ast_type": "EventDef",
                    "name": "Transfer",
                    "body": [
                        {
                            "ast_type": "AnnAssign",
                            "target": {"ast_type": "Name", "id": "sender"},
                            "annotation": {
                                "ast_type": "Call",
                                "func": {"ast_type": "Name", "id": "indexed"},
                                "args": [{"ast_type": "Name", "id": "address"}],
                                "keywords": []
                            },
                            "value": null,
                            "lineno": 2, "col_offset": 4, "end_lineno": 2, "end_col_offset": 30
                        },
                        {
                            "ast_type": "AnnAssign",
                            "target": {"ast_type": "Name", "id": "amount"},
                            "annotation": {"ast_type": "Name", "id": "uint256"},
                            "value": null,
                            "lineno": 3, "col_offset": 4, "end_lineno": 3, "end_col_offset": 20
                        }
                    ],
                    "lineno": 1, "col_offset": 0, "end_lineno": 4, "end_col_offset": 0
                }
            ],
            "lineno": 1, "col_offset": 0, "end_lineno": 4, "end_col_offset": 0
        }"#;

        let su = AstParser::parse(json_str, "test.vy").unwrap();
        assert_eq!(su.body.len(), 1);
        match &su.body[0] {
            SourceUnitElem::Event(e) => {
                assert_eq!(e.name, "Transfer");
                assert_eq!(e.fields.len(), 2);
                assert_eq!(e.fields[0].name, "sender");
                assert!(e.fields[0].indexed);
                assert_eq!(e.fields[0].typ, Type::Address);
                assert_eq!(e.fields[1].name, "amount");
                assert!(!e.fields[1].indexed);
                assert_eq!(e.fields[1].typ, Type::UInt(UIntType { bits: 256 }));
            }
            _ => panic!("Expected Event"),
        }
    }

    #[test]
    fn test_parse_assert_stmt() {
        let node = json!({
            "ast_type": "Assert",
            "test": {
                "ast_type": "Compare",
                "left": {"ast_type": "Name", "id": "amount"},
                "ops": [{"ast_type": "Gt"}],
                "comparators": [{"ast_type": "Constant", "value": 0}],
                "lineno": 1, "col_offset": 7, "end_lineno": 1, "end_col_offset": 17
            },
            "msg": {"ast_type": "Constant", "value": "Amount must be > 0"},
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 40
        });

        let stmt = AstParser::parse_stmt(&node).unwrap();
        match stmt {
            Stmt::Assert(s) => {
                assert!(s.msg.is_some());
                match &s.test {
                    Expr::Compare(c) => {
                        assert_eq!(c.ops[0], CmpOp::Gt);
                    }
                    _ => panic!("Expected Compare in assert test"),
                }
            }
            _ => panic!("Expected Assert"),
        }
    }

    #[test]
    fn test_parse_aug_assign() {
        let node = json!({
            "ast_type": "AugAssign",
            "target": {
                "ast_type": "Attribute",
                "value": {"ast_type": "Name", "id": "self"},
                "attr": "totalSupply",
                "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 16
            },
            "op": {"ast_type": "Add"},
            "value": {"ast_type": "Name", "id": "amount"},
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 27
        });

        let stmt = AstParser::parse_stmt(&node).unwrap();
        match stmt {
            Stmt::AugAssign(s) => {
                assert_eq!(s.op, BinOp::Add);
            }
            _ => panic!("Expected AugAssign"),
        }
    }

    #[test]
    fn test_parse_subscript_expr() {
        let node = json!({
            "ast_type": "Subscript",
            "value": {
                "ast_type": "Attribute",
                "value": {"ast_type": "Name", "id": "self"},
                "attr": "balanceOf",
                "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 14
            },
            "slice": {
                "ast_type": "Attribute",
                "value": {"ast_type": "Name", "id": "msg"},
                "attr": "sender",
                "lineno": 1, "col_offset": 15, "end_lineno": 1, "end_col_offset": 25
            },
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 26
        });

        let expr = AstParser::parse_expr(&node).unwrap();
        match expr {
            Expr::Subscript(s) => {
                match *s.value {
                    Expr::Attribute(ref a) => assert_eq!(a.attr, "balanceOf"),
                    _ => panic!("Expected Attribute"),
                }
                match *s.index {
                    Expr::Attribute(ref a) => assert_eq!(a.attr, "sender"),
                    _ => panic!("Expected Attribute"),
                }
            }
            _ => panic!("Expected Subscript"),
        }
    }
}
