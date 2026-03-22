//! Lower the Vyper internal AST into SIR.

use crate::vyper::ast;
use crate::vyper::ast::Loc;
use common::error::Result;
use scirs::sir::dialect::evm::*;
use scirs::sir::*;

/// Convert Vyper AST source location to SIR span.
fn loc_to_span(loc: Option<&Loc>) -> Option<Span> {
    loc.map(|l| Span::new(l.lineno, l.end_lineno))
}


/// Run all normalization passes on a Vyper source unit.
pub fn run_passes(source_unit: &ast::SourceUnit) -> ast::SourceUnit {
    let su = super::rename_vars::rename_vars(source_unit);
    let su = super::rename_defs::rename_defs(&su);
    super::flatten_expr::flatten_expr(&su)
}

/// Lower a Vyper source unit into a SIR Module.
pub fn lower_source_unit(source_unit: &ast::SourceUnit) -> Result<Module> {
    let mut lowerer = Lowerer::new();
    lowerer.lower_source_unit(source_unit)
}

/// IR generator state.
pub struct Lowerer {
    tmp_var_index: usize,
}

impl Lowerer {
    pub fn new() -> Self {
        Lowerer { tmp_var_index: 0 }
    }

    fn fresh_var_name(&mut self) -> String {
        self.tmp_var_index += 1;
        format!("tmp__{}", self.tmp_var_index)
    }

    // ─── Source unit lowering ──────────────────────────────────

    fn lower_source_unit(&mut self, su: &ast::SourceUnit) -> Result<Module> {
        let contract_name = std::path::Path::new(&su.path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "vyper_contract".to_string());

        let mut members: Vec<MemberDecl> = Vec::new();

        for elem in &su.body {
            match elem {
                ast::SourceUnitElem::VersionAnnotation(_) => { /* skip */ }
                ast::SourceUnitElem::Import(_) => { /* skip for now */ }
                ast::SourceUnitElem::StateVar(sv) => {
                    members.push(self.lower_state_var(sv)?);
                }
                ast::SourceUnitElem::Event(ev) => {
                    members.push(self.lower_event_def(ev)?);
                }
                ast::SourceUnitElem::Struct(s) => {
                    members.push(self.lower_struct_def(s)?);
                }
                ast::SourceUnitElem::EnumDef(e) => {
                    members.push(self.lower_enum_def(e));
                }
                ast::SourceUnitElem::Flag(f) => {
                    members.push(self.lower_flag_def(f));
                }
                ast::SourceUnitElem::Interface(iface) => {
                    members.push(self.lower_interface_def(iface)?);
                }
                ast::SourceUnitElem::TypeAlias(_) => { /* skip */ }
                ast::SourceUnitElem::Func(f) => {
                    let fd = self.lower_func_def(f)?;
                    members.push(MemberDecl::Function(fd));
                }
            }
        }

        let mut attrs = vec![
            Attr::sir(attrs::sir_attrs::SOURCE_LANG, AttrValue::String("vyper".to_string())),
            Attr::sir(attrs::sir_attrs::CHAIN_TARGET, AttrValue::String("evm".to_string())),
        ];

        let contract = ContractDecl {
            name: contract_name,
            parents: vec![],
            attrs: vec![],
            members,
            span: loc_to_span(su.loc.as_ref()),
        };

        let mut module = Module::new(&su.path, vec![Decl::Contract(contract)]);
        module.attrs.append(&mut attrs);
        Ok(module)
    }

    // ─── Definition lowering ──────────────────────────────────

    fn lower_state_var(&mut self, sv: &ast::StateVarDecl) -> Result<MemberDecl> {
        let ty = self.lower_type(&sv.typ);
        let mut attrs = Vec::new();

        if sv.constant {
            attrs.push(Attr::evm(attrs::evm_attrs::IS_CONSTANT, AttrValue::Bool(true)));
        }
        if sv.immutable {
            attrs.push(Attr::evm(attrs::evm_attrs::IS_IMMUTABLE, AttrValue::Bool(true)));
        }

        // Check if type is public (wrapper)
        let (actual_ty, is_public) = match &sv.typ {
            ast::types::Type::Public(inner) => (self.lower_type(inner), true),
            _ => (ty, false),
        };

        if is_public {
            attrs.push(Attr::sir(
                attrs::sir_attrs::VISIBILITY,
                AttrValue::String("public".to_string()),
            ));
        }

        Ok(MemberDecl::Storage(StorageDecl {
            name: sv.name.clone(),
            ty: actual_ty,
            init: None,
            attrs,
            span: loc_to_span(sv.loc.as_ref()),
        }))
    }

    fn lower_event_def(&mut self, ev: &ast::EventDef) -> Result<MemberDecl> {
        let params: Vec<(String, Type)> = ev
            .fields
            .iter()
            .map(|f| (f.name.clone(), self.lower_type(&f.typ)))
            .collect();
        let indexed: Vec<bool> = ev.fields.iter().map(|f| f.indexed).collect();

        Ok(MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::EventDef {
            name: ev.name.clone(),
            params,
            indexed,
            anonymous: false,
        })))
    }

    fn lower_struct_def(&mut self, s: &ast::StructDef) -> Result<MemberDecl> {
        let fields: Vec<(String, Type)> = s
            .fields
            .iter()
            .map(|f| (f.name.clone(), self.lower_type(&f.typ)))
            .collect();

        Ok(MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::StructDef {
            name: s.name.clone(),
            fields,
        })))
    }

    fn lower_enum_def(&mut self, e: &ast::EnumDef) -> MemberDecl {
        MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::EnumDef {
            name: e.name.clone(),
            variants: e.variants.clone(),
        }))
    }

    fn lower_flag_def(&mut self, f: &ast::FlagDef) -> MemberDecl {
        MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::EnumDef {
            name: f.name.clone(),
            variants: f.variants.clone(),
        }))
    }

    fn lower_interface_def(&mut self, _iface: &ast::InterfaceDef) -> Result<MemberDecl> {
        // Interfaces become type references; we don't lower them as full contracts
        // for now, just skip or add a placeholder.
        // TODO: more detailed interface lowering
        Ok(MemberDecl::TypeAlias(scirs::sir::TypeAlias {
            name: _iface.name.clone(),
            ty: Type::Dialect(DialectType::Evm(EvmType::Address)),
        }))
    }

    fn lower_func_def(&mut self, f: &ast::FuncDef) -> Result<FunctionDecl> {
        let params: Vec<scirs::sir::Param> = f
            .params
            .iter()
            .map(|p| scirs::sir::Param::new(p.name.clone(), self.lower_type(&p.typ)))
            .collect();

        let returns: Vec<Type> = match &f.return_type {
            Some(ty) => vec![self.lower_type(ty)],
            None => vec![],
        };

        let body = self.lower_stmts(&f.body)?;

        let mut attrs = Vec::new();
        for dec in &f.decorators {
            self.lower_decorator(dec, &mut attrs);
        }

        Ok(FunctionDecl {
            name: f.name.clone(),
            type_params: vec![],
            params,
            returns,
            attrs,
            spec: None,
            body: Some(body),
            span: loc_to_span(f.loc.as_ref()),
        })
    }

    fn lower_decorator(&self, dec: &ast::FuncDecorator, attrs: &mut Vec<Attr>) {
        match dec {
            ast::FuncDecorator::Deploy => {
                attrs.push(Attr::evm(attrs::evm_attrs::IS_CONSTRUCTOR, AttrValue::Bool(true)));
            }
            ast::FuncDecorator::External => {
                attrs.push(Attr::sir(
                    attrs::sir_attrs::VISIBILITY,
                    AttrValue::String("public".to_string()),
                ));
            }
            ast::FuncDecorator::Internal => {
                attrs.push(Attr::sir(
                    attrs::sir_attrs::VISIBILITY,
                    AttrValue::String("internal".to_string()),
                ));
            }
            ast::FuncDecorator::View => {
                attrs.push(Attr::sir(
                    attrs::sir_attrs::MUTABILITY,
                    AttrValue::String("view".to_string()),
                ));
            }
            ast::FuncDecorator::Pure => {
                attrs.push(Attr::sir(
                    attrs::sir_attrs::MUTABILITY,
                    AttrValue::String("pure".to_string()),
                ));
            }
            ast::FuncDecorator::Payable => {
                attrs.push(Attr::evm(attrs::evm_attrs::PAYABLE, AttrValue::Bool(true)));
            }
            ast::FuncDecorator::NonReentrant(key) => {
                let key_val = key.clone().unwrap_or_else(|| "default".to_string());
                attrs.push(Attr::evm(attrs::evm_attrs::NONREENTRANT, AttrValue::String(key_val)));
            }
            ast::FuncDecorator::Custom(name) => {
                attrs.push(Attr::evm("custom_decorator", AttrValue::String(name.clone())));
            }
        }
    }

    // ─── Statement lowering ───────────────────────────────────

    fn lower_stmts(&mut self, stmts: &[ast::stmts::Stmt]) -> Result<Vec<Stmt>> {
        let mut result = Vec::new();
        for stmt in stmts {
            if let Some(s) = self.lower_stmt(stmt)? {
                result.push(s);
            }
        }
        Ok(result)
    }

    fn lower_stmt(&mut self, stmt: &ast::stmts::Stmt) -> Result<Option<Stmt>> {
        match stmt {
            ast::stmts::Stmt::Assign(s) => {
                let lhs = self.lower_expr(&s.target)?;
                let rhs = self.lower_expr(&s.value)?;
                Ok(Some(Stmt::Assign(AssignStmt { lhs, rhs, span: loc_to_span(s.loc.as_ref()) })))
            }
            ast::stmts::Stmt::AugAssign(s) => {
                let lhs = self.lower_expr(&s.target)?;
                let rhs = self.lower_expr(&s.value)?;
                let op = self.lower_binop(&s.op);
                Ok(Some(Stmt::AugAssign(AugAssignStmt {
                    op,
                    lhs,
                    rhs,
                    span: loc_to_span(s.loc.as_ref()),
                })))
            }
            ast::stmts::Stmt::AnnAssign(s) => {
                let name = match &s.target {
                    ast::Expr::Ident(id) => id.name.clone(),
                    _ => self.fresh_var_name(),
                };
                let ty = self.lower_type(&s.annotation);
                let init = match &s.value {
                    Some(v) => Some(self.lower_expr(v)?),
                    None => None,
                };
                Ok(Some(Stmt::LocalVar(LocalVarStmt {
                    vars: vec![Some(LocalVarDecl { name, ty })],
                    init,
                    span: loc_to_span(s.loc.as_ref()),
                })))
            }
            ast::stmts::Stmt::If(s) => {
                let cond = self.lower_expr(&s.cond)?;
                let then_body = self.lower_stmts(&s.then_body)?;
                let else_body = if s.else_body.is_empty() {
                    None
                } else {
                    Some(self.lower_stmts(&s.else_body)?)
                };
                Ok(Some(Stmt::If(IfStmt {
                    cond,
                    then_body,
                    else_body,
                    span: loc_to_span(s.loc.as_ref()),
                })))
            }
            ast::stmts::Stmt::For(s) => {
                let (init, cond, update) = self.lower_for_loop(s)?;
                let body = self.lower_stmts(&s.body)?;
                Ok(Some(Stmt::For(ForStmt {
                    init: init.map(Box::new),
                    cond,
                    update: update.map(Box::new),
                    body,
                    invariant: None,
                    span: loc_to_span(s.loc.as_ref()),
                })))
            }
            ast::stmts::Stmt::Return(s) => {
                let value = match &s.value {
                    Some(v) => Some(self.lower_expr(v)?),
                    None => None,
                };
                Ok(Some(Stmt::Return(ReturnStmt { value, span: loc_to_span(s.loc.as_ref()) })))
            }
            ast::stmts::Stmt::Assert(s) => {
                let cond = self.lower_expr(&s.test)?;
                let message = match &s.msg {
                    Some(m) => Some(self.lower_expr(m)?),
                    None => None,
                };
                Ok(Some(Stmt::Assert(AssertStmt {
                    cond,
                    message,
                    span: loc_to_span(s.loc.as_ref()),
                })))
            }
            ast::stmts::Stmt::Raise(s) => {
                let args = match &s.exc {
                    Some(e) => vec![self.lower_expr(e)?],
                    None => vec![],
                };
                Ok(Some(Stmt::Revert(RevertStmt {
                    error: None,
                    args,
                    span: loc_to_span(s.loc.as_ref()),
                })))
            }
            ast::stmts::Stmt::Log(s) => {
                let (event_name, args) = self.lower_log_event(&s.event)?;
                Ok(Some(Stmt::Dialect(DialectStmt::Evm(EvmStmt::EmitEvent {
                    event: event_name,
                    args,
                    span: loc_to_span(s.loc.as_ref()),
                }))))
            }
            ast::stmts::Stmt::Pass(_) => Ok(None),
            ast::stmts::Stmt::Break(_) => Ok(Some(Stmt::Break)),
            ast::stmts::Stmt::Continue(_) => Ok(Some(Stmt::Continue)),
            ast::stmts::Stmt::Expr(s) => {
                let expr = self.lower_expr(&s.value)?;
                Ok(Some(Stmt::Expr(ExprStmt { expr, span: loc_to_span(s.loc.as_ref()) })))
            }
        }
    }

    fn lower_for_loop(
        &mut self,
        s: &ast::stmts::ForStmt,
    ) -> Result<(Option<Stmt>, Option<Expr>, Option<Stmt>)> {
        let target_name = match &s.target {
            ast::Expr::Ident(id) => id.name.clone(),
            _ => self.fresh_var_name(),
        };

        match &s.iter {
            ast::stmts::ForIter::Range(range) => {
                let start_val = match &range.start {
                    Some(e) => self.lower_expr(e)?,
                    None => Expr::Lit(Lit::Num(NumLit {
                        value: Num::Int(IntNum { value: 0.into(), typ: Type::I256 }),
                        span: None,
                    })),
                };
                let stop_val = self.lower_expr(&range.stop)?;

                // init: let target = start
                let init = Stmt::LocalVar(LocalVarStmt {
                    vars: vec![Some(LocalVarDecl {
                        name: target_name.clone(),
                        ty: Type::I256,
                    })],
                    init: Some(start_val),
                    span: None,
                });

                // cond: target < stop
                let cond = Expr::BinOp(BinOpExpr {
                    op: scirs::sir::BinOp::Lt,
                    lhs: Box::new(Expr::Var(VarExpr {
                        name: target_name.clone(),
                        ty: Type::I256,
                        span: None,
                    })),
                    rhs: Box::new(stop_val),
                    overflow: OverflowSemantics::Checked,
                    span: None,
                });

                // update: target = target + 1
                let update = Stmt::AugAssign(AugAssignStmt {
                    op: scirs::sir::BinOp::Add,
                    lhs: Expr::Var(VarExpr {
                        name: target_name.clone(),
                        ty: Type::I256,
                        span: None,
                    }),
                    rhs: Expr::Lit(Lit::Num(NumLit {
                        value: Num::Int(IntNum { value: 1.into(), typ: Type::I256 }),
                        span: None,
                    })),
                    span: None,
                });

                Ok((Some(init), Some(cond), Some(update)))
            }
            ast::stmts::ForIter::Iterable(arr) => {
                // for x in arr_var — model as index-based loop
                let arr_expr = self.lower_expr(arr)?;
                let idx_name = self.fresh_var_name();

                // init: let idx = 0
                let init = Stmt::LocalVar(LocalVarStmt {
                    vars: vec![Some(LocalVarDecl {
                        name: idx_name.clone(),
                        ty: Type::I256,
                    })],
                    init: Some(Expr::Lit(Lit::Num(NumLit {
                        value: Num::Int(IntNum { value: 0.into(), typ: Type::I256 }),
                        span: None,
                    }))),
                    span: None,
                });

                // cond: idx < len(arr)
                let cond = Expr::BinOp(BinOpExpr {
                    op: scirs::sir::BinOp::Lt,
                    lhs: Box::new(Expr::Var(VarExpr {
                        name: idx_name.clone(),
                        ty: Type::I256,
                        span: None,
                    })),
                    rhs: Box::new(Expr::Dialect(DialectExpr::Evm(EvmExpr::Len(Box::new(
                        arr_expr,
                    ))))),
                    overflow: OverflowSemantics::Checked,
                    span: None,
                });

                // update: idx = idx + 1
                let update = Stmt::AugAssign(AugAssignStmt {
                    op: scirs::sir::BinOp::Add,
                    lhs: Expr::Var(VarExpr { name: idx_name, ty: Type::I256, span: None }),
                    rhs: Expr::Lit(Lit::Num(NumLit {
                        value: Num::Int(IntNum { value: 1.into(), typ: Type::I256 }),
                        span: None,
                    })),
                    span: None,
                });

                Ok((Some(init), Some(cond), Some(update)))
            }
        }
    }

    fn lower_log_event(&mut self, event_expr: &ast::Expr) -> Result<(String, Vec<Expr>)> {
        match event_expr {
            ast::Expr::Call(call) => {
                let event_name = match call.func.as_ref() {
                    ast::Expr::Ident(id) => id.name.clone(),
                    ast::Expr::Attribute(attr) => attr.attr.clone(),
                    _ => "UnknownEvent".to_string(),
                };
                let args: Vec<Expr> = call
                    .args
                    .iter()
                    .map(|a| self.lower_expr(a))
                    .collect::<Result<Vec<_>>>()?;
                Ok((event_name, args))
            }
            ast::Expr::Ident(id) => Ok((id.name.clone(), vec![])),
            _ => Ok(("UnknownEvent".to_string(), vec![])),
        }
    }

    // ─── Expression lowering ──────────────────────────────────

    fn lower_expr(&mut self, expr: &ast::Expr) -> Result<Expr> {
        match expr {
            ast::Expr::Ident(id) => Ok(Expr::Var(VarExpr {
                name: id.name.clone(),
                ty: Type::None,
                span: loc_to_span(id.loc.as_ref()),
            })),

            ast::Expr::Lit(lit) => Ok(Expr::Lit(self.lower_lit(lit)?)),

            ast::Expr::Attribute(attr) => self.lower_attribute(attr),

            ast::Expr::Subscript(sub) => {
                let base = self.lower_expr(&sub.value)?;
                let index = self.lower_expr(&sub.index)?;
                Ok(Expr::IndexAccess(IndexAccessExpr {
                    base: Box::new(base),
                    index: Some(Box::new(index)),
                    ty: Type::None,
                    span: loc_to_span(sub.loc.as_ref()),
                }))
            }

            ast::Expr::Call(call) => {
                let callee = self.lower_expr(&call.func)?;
                let args: Vec<Expr> = call
                    .args
                    .iter()
                    .map(|a| self.lower_expr(a))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Expr::FunctionCall(CallExpr {
                    callee: Box::new(callee),
                    args,
                    ty: Type::None,
                    span: loc_to_span(call.loc.as_ref()),
                }))
            }

            ast::Expr::BinOp(e) => {
                let lhs = self.lower_expr(&e.left)?;
                let rhs = self.lower_expr(&e.right)?;
                let op = self.lower_binop(&e.op);
                Ok(Expr::BinOp(BinOpExpr {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    overflow: OverflowSemantics::Checked,
                    span: loc_to_span(e.loc.as_ref()),
                }))
            }

            ast::Expr::BoolOp(e) => {
                let op = match e.op {
                    ast::BoolOp::And => scirs::sir::BinOp::And,
                    ast::BoolOp::Or => scirs::sir::BinOp::Or,
                };
                if e.values.len() >= 2 {
                    let mut result = self.lower_expr(&e.values[0])?;
                    for val in &e.values[1..] {
                        let rhs = self.lower_expr(val)?;
                        result = Expr::BinOp(BinOpExpr {
                            op,
                            lhs: Box::new(result),
                            rhs: Box::new(rhs),
                            overflow: OverflowSemantics::Checked,
                            span: loc_to_span(e.loc.as_ref()),
                        });
                    }
                    Ok(result)
                } else if e.values.len() == 1 {
                    self.lower_expr(&e.values[0])
                } else {
                    Ok(Expr::Lit(Lit::Bool(BoolLit { value: true, span: None })))
                }
            }

            ast::Expr::Compare(e) => {
                if e.ops.len() == 1 && e.comparators.len() == 1 {
                    let lhs = self.lower_expr(&e.left)?;
                    let rhs = self.lower_expr(&e.comparators[0])?;
                    let op = self.lower_cmpop(&e.ops[0]);
                    Ok(Expr::BinOp(BinOpExpr {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        overflow: OverflowSemantics::Checked,
                        span: loc_to_span(e.loc.as_ref()),
                    }))
                } else {
                    // Chain: a < b < c => (a < b) and (b < c)
                    let mut parts = Vec::new();
                    let mut prev = self.lower_expr(&e.left)?;
                    for (op, comp) in e.ops.iter().zip(e.comparators.iter()) {
                        let rhs = self.lower_expr(comp)?;
                        let sir_op = self.lower_cmpop(op);
                        parts.push(Expr::BinOp(BinOpExpr {
                            op: sir_op,
                            lhs: Box::new(prev.clone()),
                            rhs: Box::new(rhs.clone()),
                            overflow: OverflowSemantics::Checked,
                            span: loc_to_span(e.loc.as_ref()),
                        }));
                        prev = rhs;
                    }
                    let mut result = parts.remove(0);
                    for part in parts {
                        result = Expr::BinOp(BinOpExpr {
                            op: scirs::sir::BinOp::And,
                            lhs: Box::new(result),
                            rhs: Box::new(part),
                            overflow: OverflowSemantics::Checked,
                            span: loc_to_span(e.loc.as_ref()),
                        });
                    }
                    Ok(result)
                }
            }

            ast::Expr::UnaryOp(e) => {
                let operand = self.lower_expr(&e.operand)?;
                let op = match e.op {
                    ast::UnaryOp::Not => scirs::sir::UnOp::Not,
                    ast::UnaryOp::Neg => scirs::sir::UnOp::Neg,
                    ast::UnaryOp::Invert => scirs::sir::UnOp::BitNot,
                };
                Ok(Expr::UnOp(UnOpExpr {
                    op,
                    operand: Box::new(operand),
                    span: loc_to_span(e.loc.as_ref()),
                }))
            }

            ast::Expr::Convert { expr, to, loc: _ } => {
                let inner = self.lower_expr(expr)?;
                let target_type = self.lower_type(to);
                Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::Convert {
                    expr: Box::new(inner),
                    to: target_type,
                })))
            }

            ast::Expr::Empty(ty, _loc) => {
                let sir_ty = self.lower_type(ty);
                Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::Empty(sir_ty))))
            }

            ast::Expr::Len(inner, _loc) => {
                let inner_expr = self.lower_expr(inner)?;
                Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::Len(Box::new(inner_expr)))))
            }

            ast::Expr::Concat(parts, _loc) => {
                let exprs: Vec<Expr> = parts
                    .iter()
                    .map(|p| self.lower_expr(p))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::Concat(exprs))))
            }

            ast::Expr::Slice { expr, start, length, .. } => {
                let e = self.lower_expr(expr)?;
                let s = self.lower_expr(start)?;
                let l = self.lower_expr(length)?;
                Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::Slice {
                    expr: Box::new(e),
                    start: Box::new(s),
                    length: Box::new(l),
                })))
            }

            ast::Expr::RawCall { target, data, value, gas, .. } => {
                let t = self.lower_expr(target)?;
                let d = self.lower_expr(data)?;
                let v = value
                    .as_ref()
                    .map(|v| self.lower_expr(v))
                    .transpose()?
                    .map(Box::new);
                let g = gas
                    .as_ref()
                    .map(|g| self.lower_expr(g))
                    .transpose()?
                    .map(Box::new);
                Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::RawCall {
                    target: Box::new(t),
                    data: Box::new(d),
                    value: v,
                    gas: g,
                })))
            }

            ast::Expr::Send { target, value, .. } => {
                let t = self.lower_expr(target)?;
                let v = self.lower_expr(value)?;
                Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::Send {
                    target: Box::new(t),
                    value: Box::new(v),
                })))
            }

            ast::Expr::Keccak256(inner, _loc) => {
                let inner_expr = self.lower_expr(inner)?;
                Ok(Expr::FunctionCall(CallExpr {
                    callee: Box::new(Expr::Var(VarExpr {
                        name: "keccak256".to_string(),
                        ty: Type::FixedBytes(32),
                        span: None,
                    })),
                    args: vec![inner_expr],
                    ty: Type::FixedBytes(32),
                    span: None,
                }))
            }

            ast::Expr::Sha256(inner, _loc) => {
                let inner_expr = self.lower_expr(inner)?;
                Ok(Expr::FunctionCall(CallExpr {
                    callee: Box::new(Expr::Var(VarExpr {
                        name: "sha256".to_string(),
                        ty: Type::FixedBytes(32),
                        span: None,
                    })),
                    args: vec![inner_expr],
                    ty: Type::FixedBytes(32),
                    span: None,
                }))
            }

            ast::Expr::Tuple(elems, loc) => {
                let exprs: Vec<Option<Expr>> = elems
                    .iter()
                    .map(|e| self.lower_expr(e).map(Some))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Expr::Tuple(TupleExpr {
                    elems: exprs,
                    ty: Type::None,
                    span: loc
                        .as_ref()
                        .and_then(|l| Some(Span::new(l.lineno, l.end_lineno))),
                }))
            }

            ast::Expr::IfExp(e) => {
                let cond = self.lower_expr(&e.test)?;
                let then_expr = self.lower_expr(&e.body)?;
                let else_expr = self.lower_expr(&e.orelse)?;
                Ok(Expr::Ternary(TernaryExpr {
                    cond: Box::new(cond),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                    span: loc_to_span(e.loc.as_ref()),
                }))
            }
        }
    }

    fn lower_attribute(&mut self, attr: &ast::AttributeExpr) -> Result<Expr> {
        // Detect special built-in attributes
        match attr.value.as_ref() {
            ast::Expr::Ident(id) => match id.name.as_str() {
                "self" => {
                    // self.x → storage access (lower to Var)
                    if attr.attr == "balance" {
                        return Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::SelfBalance)));
                    }
                    Ok(Expr::Var(VarExpr {
                        name: attr.attr.clone(),
                        ty: Type::None,
                        span: loc_to_span(attr.loc.as_ref()),
                    }))
                }
                "msg" => match attr.attr.as_str() {
                    "sender" => Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::MsgSender))),
                    "value" => Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::MsgValue))),
                    _ => Ok(Expr::FieldAccess(FieldAccessExpr {
                        base: Box::new(Expr::Var(VarExpr {
                            name: "msg".to_string(),
                            ty: Type::None,
                            span: None,
                        })),
                        field: attr.attr.clone(),
                        ty: Type::None,
                        span: loc_to_span(attr.loc.as_ref()),
                    })),
                },
                "block" => match attr.attr.as_str() {
                    "timestamp" => Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::Timestamp))),
                    "number" => Ok(Expr::Dialect(DialectExpr::Evm(EvmExpr::BlockNumber))),
                    _ => Ok(Expr::FieldAccess(FieldAccessExpr {
                        base: Box::new(Expr::Var(VarExpr {
                            name: "block".to_string(),
                            ty: Type::None,
                            span: None,
                        })),
                        field: attr.attr.clone(),
                        ty: Type::None,
                        span: loc_to_span(attr.loc.as_ref()),
                    })),
                },
                _ => {
                    let base = self.lower_expr(&attr.value)?;
                    Ok(Expr::FieldAccess(FieldAccessExpr {
                        base: Box::new(base),
                        field: attr.attr.clone(),
                        ty: Type::None,
                        span: loc_to_span(attr.loc.as_ref()),
                    }))
                }
            },
            _ => {
                let base = self.lower_expr(&attr.value)?;
                Ok(Expr::FieldAccess(FieldAccessExpr {
                    base: Box::new(base),
                    field: attr.attr.clone(),
                    ty: Type::None,
                    span: loc_to_span(attr.loc.as_ref()),
                }))
            }
        }
    }

    fn lower_lit(&self, lit: &ast::Lit) -> Result<Lit> {
        match &lit.kind {
            ast::LitKind::Bool(b) => {
                Ok(Lit::Bool(BoolLit { value: *b, span: loc_to_span(lit.loc.as_ref()) }))
            }
            ast::LitKind::Int(i) => Ok(Lit::Num(NumLit {
                value: Num::Int(IntNum { value: (*i).into(), typ: Type::I256 }),
                span: loc_to_span(lit.loc.as_ref()),
            })),
            ast::LitKind::Str(s) => Ok(Lit::String(StringLit {
                value: s.clone(),
                span: loc_to_span(lit.loc.as_ref()),
            })),
            ast::LitKind::Hex(h) => {
                Ok(Lit::Hex(HexLit { value: h.clone(), span: loc_to_span(lit.loc.as_ref()) }))
            }
            ast::LitKind::Bytes(b) => Ok(Lit::Hex(HexLit {
                value: b.iter().map(|byte| format!("{byte:02x}")).collect(),
                span: loc_to_span(lit.loc.as_ref()),
            })),
        }
    }

    // ─── Type lowering ────────────────────────────────────────

    fn lower_type(&self, ty: &ast::types::Type) -> Type {
        match ty {
            ast::types::Type::Bool => Type::Bool,
            ast::types::Type::Address => Type::Dialect(DialectType::Evm(EvmType::Address)),
            ast::types::Type::Bytes32 => Type::FixedBytes(32),
            ast::types::Type::UInt(u) => Type::from_int(Some(u.bits), false),
            ast::types::Type::Int(i) => Type::from_int(Some(i.bits), true),
            ast::types::Type::Decimal => Type::Si256, // approximation
            ast::types::Type::BoundedString(n) => {
                Type::Dialect(DialectType::Evm(EvmType::BoundedString(*n)))
            }
            ast::types::Type::BoundedBytes(n) => {
                Type::Dialect(DialectType::Evm(EvmType::BoundedBytes(*n)))
            }
            ast::types::Type::DynArray { elem, max_len } => {
                Type::Dialect(DialectType::Evm(EvmType::DynArray {
                    elem: Box::new(self.lower_type(elem)),
                    max_len: *max_len,
                }))
            }
            ast::types::Type::FixedArray { elem, len } => {
                Type::FixedArray(Box::new(self.lower_type(elem)), *len)
            }
            ast::types::Type::HashMap(k, v) => {
                Type::Map(Box::new(self.lower_type(k)), Box::new(self.lower_type(v)))
            }
            ast::types::Type::Struct(name) => Type::TypeRef(name.clone()),
            ast::types::Type::Enum(name) => Type::TypeRef(name.clone()),
            ast::types::Type::Flag(name) => Type::TypeRef(name.clone()),
            ast::types::Type::Interface(name) => Type::TypeRef(name.clone()),
            ast::types::Type::Public(inner) => self.lower_type(inner),
            ast::types::Type::Tuple(elems) => {
                Type::Tuple(elems.iter().map(|e| self.lower_type(e)).collect())
            }
        }
    }

    // ─── Operator lowering ────────────────────────────────────

    fn lower_binop(&self, op: &ast::BinOp) -> scirs::sir::BinOp {
        match op {
            ast::BinOp::Add => scirs::sir::BinOp::Add,
            ast::BinOp::Sub => scirs::sir::BinOp::Sub,
            ast::BinOp::Mul => scirs::sir::BinOp::Mul,
            ast::BinOp::Div | ast::BinOp::FloorDiv => scirs::sir::BinOp::Div,
            ast::BinOp::Mod => scirs::sir::BinOp::Mod,
            ast::BinOp::Pow => scirs::sir::BinOp::Pow,
            ast::BinOp::BitAnd => scirs::sir::BinOp::BitAnd,
            ast::BinOp::BitOr => scirs::sir::BinOp::BitOr,
            ast::BinOp::BitXor => scirs::sir::BinOp::BitXor,
            ast::BinOp::Shl => scirs::sir::BinOp::Shl,
            ast::BinOp::Shr => scirs::sir::BinOp::Shr,
        }
    }

    fn lower_cmpop(&self, op: &ast::CmpOp) -> scirs::sir::BinOp {
        match op {
            ast::CmpOp::Eq => scirs::sir::BinOp::Eq,
            ast::CmpOp::NotEq => scirs::sir::BinOp::Ne,
            ast::CmpOp::Lt => scirs::sir::BinOp::Lt,
            ast::CmpOp::LtE => scirs::sir::BinOp::Le,
            ast::CmpOp::Gt => scirs::sir::BinOp::Gt,
            ast::CmpOp::GtE => scirs::sir::BinOp::Ge,
            ast::CmpOp::In => scirs::sir::BinOp::Eq, // approximate
            ast::CmpOp::NotIn => scirs::sir::BinOp::Ne, // approximate
        }
    }
}

// ─── Unit tests ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vyper::ast::types::{IntType, UIntType};

    #[test]
    fn test_lower_type_bool() {
        let ir = Lowerer::new();
        assert_eq!(ir.lower_type(&ast::types::Type::Bool), Type::Bool);
    }

    #[test]
    fn test_lower_type_address() {
        let ir = Lowerer::new();
        assert_eq!(
            ir.lower_type(&ast::types::Type::Address),
            Type::Dialect(DialectType::Evm(EvmType::Address))
        );
    }

    #[test]
    fn test_lower_type_uint256() {
        let ir = Lowerer::new();
        assert_eq!(ir.lower_type(&ast::types::Type::UInt(UIntType { bits: 256 })), Type::I256);
    }

    #[test]
    fn test_lower_type_int128() {
        let ir = Lowerer::new();
        assert_eq!(ir.lower_type(&ast::types::Type::Int(IntType { bits: 128 })), Type::Si128);
    }

    #[test]
    fn test_lower_type_bounded_string() {
        let ir = Lowerer::new();
        assert_eq!(
            ir.lower_type(&ast::types::Type::BoundedString(32)),
            Type::Dialect(DialectType::Evm(EvmType::BoundedString(32)))
        );
    }

    #[test]
    fn test_lower_type_hashmap() {
        let ir = Lowerer::new();
        let ty = ast::types::Type::HashMap(
            Box::new(ast::types::Type::Address),
            Box::new(ast::types::Type::UInt(UIntType { bits: 256 })),
        );
        assert_eq!(
            ir.lower_type(&ty),
            Type::Map(
                Box::new(Type::Dialect(DialectType::Evm(EvmType::Address))),
                Box::new(Type::I256)
            )
        );
    }

    #[test]
    fn test_lower_type_dynarray() {
        let ir = Lowerer::new();
        let ty = ast::types::Type::DynArray {
            elem: Box::new(ast::types::Type::UInt(UIntType { bits: 256 })),
            max_len: 100,
        };
        assert_eq!(
            ir.lower_type(&ty),
            Type::Dialect(DialectType::Evm(EvmType::DynArray {
                elem: Box::new(Type::I256),
                max_len: 100
            }))
        );
    }

    #[test]
    fn test_lower_binop() {
        let ir = Lowerer::new();
        assert_eq!(ir.lower_binop(&ast::BinOp::Add), scirs::sir::BinOp::Add);
        assert_eq!(ir.lower_binop(&ast::BinOp::Sub), scirs::sir::BinOp::Sub);
        assert_eq!(ir.lower_binop(&ast::BinOp::Mul), scirs::sir::BinOp::Mul);
        assert_eq!(ir.lower_binop(&ast::BinOp::Mod), scirs::sir::BinOp::Mod);
    }

    #[test]
    fn test_lower_cmpop() {
        let ir = Lowerer::new();
        assert_eq!(ir.lower_cmpop(&ast::CmpOp::Eq), scirs::sir::BinOp::Eq);
        assert_eq!(ir.lower_cmpop(&ast::CmpOp::GtE), scirs::sir::BinOp::Ge);
        assert_eq!(ir.lower_cmpop(&ast::CmpOp::Lt), scirs::sir::BinOp::Lt);
    }

    #[test]
    fn test_lower_simple_source_unit() {
        let su = ast::SourceUnit {
            path: "test.vy".to_string(),
            body: vec![
                ast::SourceUnitElem::StateVar(ast::StateVarDecl {
                    name: "owner".to_string(),
                    typ: ast::types::Type::Address,
                    constant: false,
                    immutable: false,
                    doc_string: None,
                    loc: None,
                }),
                ast::SourceUnitElem::Func(ast::FuncDef {
                    name: "__init__".to_string(),
                    params: vec![],
                    return_type: None,
                    decorators: vec![ast::FuncDecorator::Deploy, ast::FuncDecorator::External],
                    doc_string: None,
                    body: vec![],
                    loc: None,
                }),
            ],
            loc: None,
        };

        let module = lower_source_unit(&su).unwrap();
        assert_eq!(module.id, "test.vy");
        assert_eq!(module.decls.len(), 1);

        match &module.decls[0] {
            Decl::Contract(c) => {
                assert_eq!(c.name, "test");
                assert_eq!(c.members.len(), 2);

                // First member is storage
                match &c.members[0] {
                    MemberDecl::Storage(s) => {
                        assert_eq!(s.name, "owner");
                    }
                    _ => panic!("Expected Storage"),
                }

                // Second member is function
                match &c.members[1] {
                    MemberDecl::Function(f) => {
                        assert_eq!(f.name, "__init__");
                        assert!(f.attrs.iter().any(|a| a.key == "is_constructor"));
                    }
                    _ => panic!("Expected Function"),
                }
            }
            _ => panic!("Expected Contract"),
        }
    }
}
