//! Lower the Solidity AST into SIR.

use crate::solidity::ast;
use crate::solidity::ast::Loc;
use common::{error::Result, fail};
use log::trace;
use scirs::sir::attrs::{Attr, AttrValue, sir_attrs};
use scirs::sir::dialect::evm::*;
use scirs::sir::*;

/// Convert AST source location to SIR span.
fn loc_to_span(loc: Option<Loc>) -> Option<common::loc::Loc> {
    loc.map(|l| common::loc::Loc::new(l.start_line as u32, l.end_line as u32))
}

// Modules moved to mod.rs

/// Supporting function to print output source unit of a normalization step.
fn print_output_source_units(source_units: &[ast::SourceUnit]) {
    trace!("Output source unit:");
    for source_unit in source_units {
        if log::max_level() >= log::Level::Trace {
            source_unit.print_highlighted_code();
            println!();
        }
    }
}

/// Run all normalization passes on source units.
pub fn run_passes(source_units: &[ast::SourceUnit]) -> Vec<ast::SourceUnit> {
    let source_units = super::unroll_tuples::unroll_unary_tuple(source_units);
    print_output_source_units(&source_units);

    let env = ast::NamingEnv::new();
    let (source_units, env) = super::rename_defs::rename_defs(&source_units, Some(&env));
    print_output_source_units(&source_units);

    let source_units = super::eliminate_imports::eliminate_import(&source_units);
    print_output_source_units(&source_units);

    let source_units = super::merge_pragmas::merge_pragmas(&source_units);
    print_output_source_units(&source_units);

    let (source_units, _) = super::rename_callees::rename_callees(&source_units, Some(&env));
    print_output_source_units(&source_units);

    let source_units = super::resolve_inheritance::resolve_inheritance(&source_units);
    print_output_source_units(&source_units);

    let source_units = super::strip_specifiers::strip_specifiers(&source_units);
    print_output_source_units(&source_units);

    super::unroll_tuples::unroll_unary_tuple(&source_units)
}

pub fn lower_source_unit(source_unit: &ast::SourceUnit) -> Result<Module> {
    let mut lowerer = Lowerer::new();
    lowerer.lower_source_unit(source_unit)
}

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

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn lower_source_unit(&mut self, su: &ast::SourceUnit) -> Result<Module> {
        let mut decls: Vec<Decl> = vec![];
        let mut global_members: Vec<MemberDecl> = vec![];
        let mut module_attrs: Vec<Attr> = vec![];

        for elem in &su.elems {
            match elem {
                ast::SourceUnitElem::Pragma(p) => {
                    // Capture `pragma solidity <version>` as a module attribute.
                    if let ast::PragmaKind::Version(ver) = &p.kind {
                        module_attrs.push(Attr::sir(
                            sir_attrs::PRAGMA_SOLIDITY,
                            AttrValue::String(ver.clone()),
                        ));
                    }
                }
                ast::SourceUnitElem::Import(_) => {
                    fail!("IR: `import` must be eliminated: {}", elem)
                }
                ast::SourceUnitElem::Using(u) => {
                    // Preserve using-for directives — will be eliminated at SIR → CIR level.
                    let target_type = match &u.target_type {
                        Some(t) => Some(self.lower_type(t)?),
                        None => None,
                    };
                    let library = match &u.kind {
                        ast::UsingKind::UsingLib(lib) => lib.lib_name.to_string(),
                        ast::UsingKind::UsingFunc(funcs) => funcs
                            .iter()
                            .map(|f| f.func_name.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    };
                    global_members.push(MemberDecl::UsingFor(UsingForDecl {
                        library,
                        target_type,
                        span: loc_to_span(u.loc),
                    }));
                }
                ast::SourceUnitElem::Error(e) => {
                    global_members.push(self.lower_error_def(e)?);
                }
                ast::SourceUnitElem::Var(v) => {
                    global_members.push(self.lower_state_var(v)?);
                }
                ast::SourceUnitElem::UserType(_) => {
                    fail!("User-defined type must be eliminated!")
                }
                ast::SourceUnitElem::Struct(s) => {
                    global_members.push(self.lower_struct_def(s)?);
                }
                ast::SourceUnitElem::Enum(e) => {
                    global_members.push(self.lower_enum_def(e));
                }
                ast::SourceUnitElem::Func(f) => {
                    let fd = self.lower_func_def(f)?;
                    global_members.push(MemberDecl::Function(fd));
                }
                ast::SourceUnitElem::Contract(c) => {
                    decls.push(Decl::Contract(self.lower_contract_def(c)?));
                }
            }
        }

        // Wrap free-standing items in a synthetic __global__ contract
        if !global_members.is_empty() {
            let global = ContractDecl {
                name: "__global__".to_string(),
                parents: vec![],
                attrs: vec![],
                members: global_members,
                span: None,
            };
            decls.insert(0, Decl::Contract(global));
        }

        Ok(Module { id: su.path.clone(), attrs: module_attrs, decls })
    }

    //-------------------------------------------------
    // Contract definition
    //-------------------------------------------------

    fn lower_contract_def(&mut self, c: &ast::ContractDef) -> Result<ContractDecl> {
        trace!("Lower contract: {}", c.name);

        // Populate parents from base_contracts — will be resolved at SIR → CIR level.
        let parents: Vec<String> = c
            .base_contracts
            .iter()
            .map(|b| b.name.to_string())
            .collect();

        let mut members = vec![];
        for elem in &c.body {
            members.extend(self.lower_contract_elem(elem)?);
        }

        Ok(ContractDecl {
            name: c.name.to_string(),
            parents,
            attrs: vec![],
            members,
            span: loc_to_span(c.loc),
        })
    }

    fn lower_contract_elem(&mut self, elem: &ast::ContractElem) -> Result<Vec<MemberDecl>> {
        match elem {
            ast::ContractElem::Using(u) => {
                // Preserve using-for directives — will be eliminated at SIR → CIR level.
                let target_type = match &u.target_type {
                    Some(t) => Some(self.lower_type(t)?),
                    None => None,
                };
                let library = match &u.kind {
                    ast::UsingKind::UsingLib(lib) => lib.lib_name.to_string(),
                    ast::UsingKind::UsingFunc(funcs) => funcs
                        .iter()
                        .map(|f| f.func_name.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                };
                Ok(vec![MemberDecl::UsingFor(UsingForDecl {
                    library,
                    target_type,
                    span: loc_to_span(u.loc),
                })])
            }
            ast::ContractElem::Event(e) => Ok(vec![self.lower_event_def(e)?]),
            ast::ContractElem::Error(e) => Ok(vec![self.lower_error_def(e)?]),
            ast::ContractElem::Struct(s) => Ok(vec![self.lower_struct_def(s)?]),
            ast::ContractElem::Enum(e) => Ok(vec![self.lower_enum_def(e)]),
            ast::ContractElem::Type(_) => {
                fail!("IR: user-defined type must be eliminated!")
            }
            ast::ContractElem::Var(v) => Ok(vec![self.lower_state_var(v)?]),
            ast::ContractElem::Func(f) => {
                if f.kind == ast::FuncKind::Modifier {
                    // Lower modifier definitions — will be inlined at SIR → CIR level.
                    Ok(vec![self.lower_modifier_def(f)?])
                } else {
                    Ok(vec![MemberDecl::Function(self.lower_func_def(f)?)])
                }
            }
        }
    }

    //-------------------------------------------------
    // Definitions
    //-------------------------------------------------

    fn lower_error_def(&mut self, e: &ast::ErrorDef) -> Result<MemberDecl> {
        let params = self.lower_param_list(&e.params)?;
        Ok(MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::ErrorDef(EvmErrorDef {
            name: e.name.to_string(),
            params: params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone()))
                .collect(),
            loc: Default::default(),
        }))))
    }

    fn lower_event_def(&mut self, e: &ast::EventDef) -> Result<MemberDecl> {
        let params = self.lower_param_list(&e.params)?;
        Ok(MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::EventDef(EvmEventDef {
            name: e.name.to_string(),
            params: params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone()))
                .collect(),
            indexed: vec![false; params.len()],
            anonymous: e.is_anonymous,
            loc: Default::default(),
        }))))
    }

    fn lower_struct_def(&mut self, s: &ast::StructDef) -> Result<MemberDecl> {
        let mut fields = vec![];
        for f in &s.fields {
            let ty = self.lower_type(&f.typ)?;
            fields.push((f.name.clone(), ty));
        }
        Ok(MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::StructDef(EvmStructDef {
            name: s.name.to_string(),
            fields,
            loc: Default::default(),
        }))))
    }

    fn lower_enum_def(&mut self, e: &ast::EnumDef) -> MemberDecl {
        MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::EnumDef(EvmEnumDef {
            name: e.name.to_string(),
            variants: e.elems.clone(),
            loc: Default::default(),
        })))
    }

    fn lower_state_var(&mut self, v: &ast::VarDecl) -> Result<MemberDecl> {
        let ty = self.lower_type(&v.typ)?;
        let init = match &v.value {
            Some(e) => {
                let (expr, _stmts) = self.lower_expr(e)?;
                Some(expr)
            }
            None => None,
        };
        Ok(MemberDecl::Storage(StorageDecl::new(v.name.to_string(), ty, init, loc_to_span(v.loc))))
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    fn lower_func_def(&mut self, f: &ast::FuncDef) -> Result<FunctionDecl> {
        trace!("Lower function: {}", f.name);
        let params = self.lower_param_list(&f.params)?;
        let returns = f
            .returns
            .iter()
            .map(|r| self.lower_type(&r.typ))
            .collect::<Result<Vec<_>>>()?;
        let body = match &f.body {
            Some(blk) => Some(self.lower_block(blk)?),
            None => None,
        };
        let modifier_invocs = self.lower_modifier_invocations(&f.modifier_invocs)?;
        let mut decl =
            FunctionDecl::new(f.name.to_string(), params, returns, body, loc_to_span(f.loc));
        decl.modifier_invocs = modifier_invocs;
        Ok(decl)
    }

    fn lower_modifier_invocations(
        &mut self,
        invocs: &[ast::CallExpr],
    ) -> Result<Vec<ModifierInvoc>> {
        let mut result = vec![];
        for invoc in invocs {
            // Skip base constructor calls — those are handled differently.
            if invoc.kind == ast::CallKind::BaseConstructorCall {
                continue;
            }
            let callee_name = invoc.callee.to_string();
            let args = match &invoc.args {
                ast::CallArgs::Unnamed(args) => {
                    let mut stmts = vec![];
                    let mut exprs = vec![];
                    for e in args {
                        let (ne, extra) = self.lower_expr(e)?;
                        stmts.extend(extra);
                        exprs.push(ne);
                    }
                    exprs
                }
                ast::CallArgs::Named(_) => {
                    fail!("Named args in modifier invocations are not supported")
                }
            };
            result.push(ModifierInvoc { name: callee_name, args, span: loc_to_span(invoc.loc) });
        }
        Ok(result)
    }

    fn lower_modifier_def(&mut self, f: &ast::FuncDef) -> Result<MemberDecl> {
        let params = self.lower_param_list(&f.params)?;
        let body = match &f.body {
            Some(blk) => self.lower_block_with_placeholder(blk)?,
            None => vec![],
        };
        Ok(MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::ModifierDef(
            EvmModifierDef {
                name: f.name.to_string(),
                params: params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect(),
                body,
                loc: Default::default(),
            },
        ))))
    }

    fn lower_block_with_placeholder(&mut self, blk: &ast::Block) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        for s in &blk.body {
            stmts.extend(self.lower_stmt_with_placeholder(s)?);
        }
        Ok(stmts)
    }

    fn lower_stmt_with_placeholder(&mut self, stmt: &ast::Stmt) -> Result<Vec<Stmt>> {
        match stmt {
            ast::Stmt::Placeholder(_) => Ok(vec![Stmt::Dialect(DialectStmt::Evm(
                EvmStmt::Placeholder(EvmPlaceholder { loc: Default::default() }),
            ))]),
            _ => self.lower_stmt(stmt),
        }
    }

    fn lower_param_list(&mut self, params: &[ast::VarDecl]) -> Result<Vec<Param>> {
        params
            .iter()
            .map(|p| {
                let ty = self.lower_type(&p.typ)?;
                Ok(Param::new(p.name.to_string(), ty))
            })
            .collect()
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    fn lower_block(&mut self, blk: &ast::Block) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        for s in &blk.body {
            stmts.extend(self.lower_stmt(s)?);
        }
        Ok(stmts)
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    fn lower_stmt(&mut self, stmt: &ast::Stmt) -> Result<Vec<Stmt>> {
        match stmt {
            ast::Stmt::Asm(s) => Ok(self.lower_asm_stmt(s)),
            ast::Stmt::Block(blk) => self.lower_block(blk),
            ast::Stmt::Break(_) => Ok(vec![Stmt::Break]),
            ast::Stmt::Placeholder(_) => {
                fail!("PlaceholderStmt must be eliminated at AST level!")
            }
            ast::Stmt::Continue(_) => Ok(vec![Stmt::Continue]),
            ast::Stmt::DoWhile(s) => self.lower_do_while_stmt(s),
            ast::Stmt::Emit(s) => self.lower_emit_stmt(s),
            ast::Stmt::Expr(s) => self.lower_expr_stmt(s),
            ast::Stmt::For(s) => self.lower_for_stmt(s),
            ast::Stmt::If(s) => self.lower_if_stmt(s),
            ast::Stmt::Return(s) => self.lower_return_stmt(s),
            ast::Stmt::Revert(s) => self.lower_revert_stmt(s),
            ast::Stmt::Throw(s) => Ok(vec![Stmt::Revert(RevertStmt {
                error: None,
                args: vec![],
                span: loc_to_span(s.loc),
            })]),
            ast::Stmt::Try(s) => self.lower_try_stmt(s),
            ast::Stmt::VarDecl(s) => self.lower_var_decl_stmt(s),
            ast::Stmt::While(s) => self.lower_while_stmt(s),
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    fn lower_asm_stmt(&mut self, s: &ast::AsmStmt) -> Vec<Stmt> {
        let asm_text = s
            .body
            .iter()
            .map(|s| format!("{s}"))
            .collect::<Vec<_>>()
            .join("\n");
        let expr = Expr::Dialect(DialectExpr::Evm(EvmExpr::InlineAsm(EvmInlineAsm {
            asm_text,
            loc: Default::default(),
        })));
        vec![Stmt::Expr(ExprStmt { expr, span: loc_to_span(s.loc) })]
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    fn lower_expr_stmt(&mut self, s: &ast::ExprStmt) -> Result<Vec<Stmt>> {
        match &s.expr {
            ast::Expr::Assign(a) => self.lower_assign_expr_as_stmt(a, loc_to_span(s.loc)),
            // ── assert(cond) → Stmt::Assert ──────────────────────
            ast::Expr::Call(call) if call.callee.to_string() == "assert" => {
                self.lower_assert(call, loc_to_span(s.loc))
            }
            // ── require(cond, msg?) → if !cond { revert(msg) } ──
            ast::Expr::Call(call) if call.callee.to_string() == "require" => {
                self.lower_require(call, loc_to_span(s.loc))
            }
            // ── selfdestruct(recipient) → EvmStmt::Selfdestruct ──
            ast::Expr::Call(call) if call.callee.to_string() == "selfdestruct" => {
                let (args, extra) = self.lower_call_args_exprs(&call.args)?;
                let mut stmts = extra;
                let mut pos = args.into_positional();
                let recipient = if pos.is_empty() {
                    Expr::Lit(Lit::Bool(BoolLit::new(false, loc_to_span(s.loc))))
                } else {
                    pos.remove(0)
                };
                stmts.push(Stmt::Dialect(DialectStmt::Evm(EvmStmt::Selfdestruct(
                    EvmSelfdestruct { recipient, loc: loc_to_span(s.loc).unwrap_or_default() },
                ))));
                Ok(stmts)
            }
            _ => {
                let mut stmts = vec![];
                let (expr, extra) = self.lower_expr(&s.expr)?;
                stmts.extend(extra);
                stmts.push(Stmt::Expr(ExprStmt { expr, span: loc_to_span(s.loc) }));
                Ok(stmts)
            }
        }
    }

    /// Lower `assert(cond)` → `Stmt::Assert { cond, message: None }`.
    fn lower_assert(
        &mut self,
        call: &ast::CallExpr,
        span: Option<common::loc::Loc>,
    ) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let (args, extra) = self.lower_call_args_exprs(&call.args)?;
        stmts.extend(extra);
        let mut pos = args.into_positional();
        let cond = if pos.is_empty() {
            Expr::Lit(Lit::Bool(BoolLit::new(true, span)))
        } else {
            pos.remove(0)
        };
        let message = if pos.is_empty() {
            None
        } else {
            Some(pos.remove(0))
        };
        stmts.push(Stmt::Assert(AssertStmt { cond, message, span }));
        Ok(stmts)
    }

    /// Lower `require(cond, msg?)` → `if !cond { revert(msg) }`.
    fn lower_require(
        &mut self,
        call: &ast::CallExpr,
        span: Option<common::loc::Loc>,
    ) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let (args, extra) = self.lower_call_args_exprs(&call.args)?;
        stmts.extend(extra);
        let mut pos = args.into_positional();
        let cond = if pos.is_empty() {
            Expr::Lit(Lit::Bool(BoolLit::new(true, span)))
        } else {
            pos.remove(0)
        };
        let revert_args = pos; // remaining args become revert message
        let negated_cond = Expr::UnOp(UnOpExpr { op: UnOp::Not, operand: Box::new(cond), span });
        let revert_stmt = Stmt::Revert(RevertStmt { error: None, args: revert_args, span });
        stmts.push(Stmt::If(IfStmt {
            cond: negated_cond,
            then_body: vec![revert_stmt],
            else_body: None,
            span,
        }));
        Ok(stmts)
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    fn lower_if_stmt(&mut self, s: &ast::IfStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let (cond, extra) = self.lower_expr(&s.condition)?;
        stmts.extend(extra);
        let then_body = self.lower_stmt(&s.true_branch)?;
        let else_body = match &s.false_branch {
            Some(fb) => Some(self.lower_stmt(fb)?),
            None => None,
        };
        stmts.push(Stmt::If(IfStmt { cond, then_body, else_body, span: loc_to_span(s.loc) }));
        Ok(stmts)
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    fn lower_for_stmt(&mut self, s: &ast::ForStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let init = match &s.pre_loop {
            Some(s) => {
                let v = self.lower_stmt(s)?;
                if v.len() == 1 {
                    Some(Box::new(v.into_iter().next().unwrap()))
                } else {
                    Some(Box::new(Stmt::Block(v)))
                }
            }
            None => None,
        };
        let cond = match &s.condition {
            Some(e) => {
                let (c, extra) = self.lower_expr(e)?;
                stmts.extend(extra);
                Some(c)
            }
            None => None,
        };
        let update = match &s.post_loop {
            Some(s) => {
                let v = self.lower_stmt(s)?;
                if v.len() == 1 {
                    Some(Box::new(v.into_iter().next().unwrap()))
                } else {
                    Some(Box::new(Stmt::Block(v)))
                }
            }
            None => None,
        };
        let body = self.lower_stmt(&s.body)?;
        stmts.push(Stmt::For(ForStmt {
            init,
            cond,
            update,
            body,
            invariant: None,
            span: loc_to_span(s.loc),
        }));
        Ok(stmts)
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    fn lower_while_stmt(&mut self, s: &ast::WhileStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let (cond, extra) = self.lower_expr(&s.condition)?;
        stmts.extend(extra);
        let body = self.lower_stmt(&s.body)?;
        stmts.push(Stmt::While(WhileStmt {
            cond,
            body,
            invariant: None,
            span: loc_to_span(s.loc),
        }));
        Ok(stmts)
    }

    //-------------------------------------------------
    // Do-while statement
    //-------------------------------------------------

    fn lower_do_while_stmt(&mut self, s: &ast::DoWhileStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let (cond, extra) = self.lower_expr(&s.condition)?;
        stmts.extend(extra);
        let body = self.lower_stmt(&s.body)?;
        stmts.push(Stmt::While(WhileStmt {
            cond,
            body,
            invariant: None,
            span: loc_to_span(s.loc),
        }));
        Ok(stmts)
    }

    //-------------------------------------------------
    // Try-catch statement
    //-------------------------------------------------

    fn lower_try_stmt(&mut self, s: &ast::TryStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let (expr, extra) = self.lower_expr(&s.guarded_expr)?;
        stmts.extend(extra);
        let returns = s
            .returns
            .iter()
            .map(|r| {
                let ty = self.lower_type(&r.typ)?;
                Ok((r.name.to_string(), ty))
            })
            .collect::<Result<Vec<_>>>()?;
        let body = self.lower_block(&s.body)?;
        let mut catches = vec![];
        for c in &s.catch_clauses {
            let params = c
                .params
                .iter()
                .map(|p| {
                    let ty = self.lower_type(&p.typ)?;
                    Ok((p.name.to_string(), ty))
                })
                .collect::<Result<Vec<_>>>()?;
            catches.push(EvmCatchClause {
                error: c.error.clone(),
                params,
                body: self.lower_block(&c.body)?,
                loc: loc_to_span(c.loc).unwrap_or_default(),
            });
        }
        stmts.push(Stmt::Dialect(DialectStmt::Evm(EvmStmt::TryCatch(EvmTryCatch {
            guarded_expr: expr,
            returns,
            body,
            catch_clauses: catches,
            loc: loc_to_span(s.loc).unwrap_or_default(),
        }))));
        Ok(stmts)
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    fn lower_emit_stmt(&mut self, s: &ast::EmitStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let event = match &s.event {
            ast::Expr::Ident(id) => id.name.to_string(),
            e => fail!("TODO: event expression: {} in emit stmt", e),
        };
        let (args, extra) = self.lower_call_args_exprs(&s.args)?;
        stmts.extend(extra);
        stmts.push(Stmt::Dialect(DialectStmt::Evm(EvmStmt::EmitEvent(EvmEmitEvent {
            event,
            args: args.into_positional(),
            loc: loc_to_span(s.loc).unwrap_or_default(),
        }))));
        Ok(stmts)
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    fn lower_revert_stmt(&mut self, s: &ast::RevertStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let error = match &s.error {
            Some(ast::Expr::Ident(id)) => Some(id.name.to_string()),
            Some(e) => fail!("TODO: error expression: {} in revert", e),
            None => None,
        };
        let (args, extra) = self.lower_call_args_exprs(&s.args)?;
        stmts.extend(extra);
        stmts.push(Stmt::Revert(RevertStmt {
            error,
            args: args.into_positional(),
            span: loc_to_span(s.loc),
        }));
        Ok(stmts)
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    fn lower_return_stmt(&mut self, s: &ast::ReturnStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let value = match &s.expr {
            Some(e) => {
                let (expr, extra) = self.lower_expr(e)?;
                stmts.extend(extra);
                Some(expr)
            }
            None => None,
        };
        stmts.push(Stmt::Return(ReturnStmt { value, span: loc_to_span(s.loc) }));
        Ok(stmts)
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    fn lower_var_decl_stmt(&mut self, s: &ast::VarDeclStmt) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let mut vars: Vec<Option<LocalVarDecl>> = vec![];
        for vopt in &s.var_decls {
            match vopt {
                Some(vd) => {
                    let ty = self.lower_type(&vd.typ)?;
                    vars.push(Some(LocalVarDecl { name: vd.name.to_string(), ty }));
                }
                None => vars.push(None),
            }
        }
        let init = match &s.value {
            Some(e) => {
                let (expr, extra) = self.lower_expr(e)?;
                stmts.extend(extra);
                Some(expr)
            }
            None => None,
        };
        stmts.push(Stmt::LocalVar(LocalVarStmt { vars, init, span: loc_to_span(s.loc) }));
        Ok(stmts)
    }

    //-------------------------------------------------
    // Assignment expression (→ statement)
    //-------------------------------------------------

    fn lower_assign_expr_as_stmt(
        &mut self,
        e: &ast::AssignExpr,
        span: Option<common::loc::Loc>,
    ) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];
        let (lhs, extra) = self.lower_expr(&e.left)?;
        stmts.extend(extra);
        let (rhs, extra) = self.lower_expr(&e.right)?;
        stmts.extend(extra);

        match e.operator {
            ast::AssignOp::Assign => {
                stmts.push(Stmt::Assign(AssignStmt { lhs, rhs, span }));
            }
            ref op => {
                let binop = match op {
                    ast::AssignOp::AssignAdd => BinOp::Add,
                    ast::AssignOp::AssignSub => BinOp::Sub,
                    ast::AssignOp::AssignMul => BinOp::Mul,
                    ast::AssignOp::AssignDiv => BinOp::Div,
                    ast::AssignOp::AssignMod => BinOp::Mod,
                    ast::AssignOp::AssignBitAnd => BinOp::BitAnd,
                    ast::AssignOp::AssignBitOr => BinOp::BitOr,
                    ast::AssignOp::AssignBitXor => BinOp::BitXor,
                    ast::AssignOp::AssignShl => BinOp::Shl,
                    ast::AssignOp::AssignShr => BinOp::Shr,
                    _ => fail!("Unknown assign operator: {}", op),
                };
                stmts.push(Stmt::AugAssign(AugAssignStmt { op: binop, lhs, rhs, span }));
            }
        }
        Ok(stmts)
    }

    //-------------------------------------------------
    // Expression
    //-------------------------------------------------

    fn lower_expr(&mut self, expr: &ast::Expr) -> Result<(Expr, Vec<Stmt>)> {
        match expr {
            ast::Expr::Lit(l) => Ok((Expr::Lit(self.lower_lit(l)?), vec![])),
            ast::Expr::Ident(id) => match id.name.base.as_str() {
                "this" => Ok((
                    Expr::Dialect(DialectExpr::Evm(EvmExpr::This(EvmThis {
                        loc: Default::default(),
                    }))),
                    vec![],
                )),
                "super" => Ok((
                    Expr::Dialect(DialectExpr::Evm(EvmExpr::Super(EvmSuper {
                        loc: Default::default(),
                    }))),
                    vec![],
                )),
                _ => {
                    let ty = self.lower_type(&id.typ)?;
                    Ok((
                        Expr::Var(VarExpr::new(id.name.to_string(), ty, loc_to_span(id.loc))),
                        vec![],
                    ))
                }
            },
            ast::Expr::Unary(e) => self.lower_unary_expr(e),
            ast::Expr::Binary(e) => self.lower_binary_expr(e),
            ast::Expr::Assign(e) => {
                // Assignment as expression: emit assign stmt, return lhs
                let stmts = self.lower_assign_expr_as_stmt(e, loc_to_span(e.loc))?;
                let (lhs, extra) = self.lower_expr(&e.left)?;
                let mut all_stmts = stmts;
                all_stmts.extend(extra);
                Ok((lhs, all_stmts))
            }
            ast::Expr::Call(e) => self.lower_call_expr(e),
            ast::Expr::CallOpts(e) => self.lower_call_opts_expr(e),
            ast::Expr::Tuple(e) => self.lower_tuple_expr(e),
            ast::Expr::Index(e) => self.lower_index_expr(e),
            ast::Expr::Slice(e) => self.lower_slice_expr(e),
            ast::Expr::Member(e) => self.lower_member_expr(e),
            ast::Expr::Conditional(e) => self.lower_conditional_expr(e),
            ast::Expr::InlineArray(e) => self.lower_inline_array_expr(e),
            ast::Expr::New(e) => self.lower_new_expr(e),
            ast::Expr::TypeName(e) => {
                let ty = self.lower_type(&e.typ)?;
                Ok((
                    Expr::Var(VarExpr::new(ty.to_string(), Type::None, loc_to_span(e.loc))),
                    vec![],
                ))
            }
        }
    }

    //-------------------------------------------------
    // Unary expression
    //-------------------------------------------------

    fn lower_unary_expr(&mut self, e: &ast::UnaryExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let (operand, extra) = self.lower_expr(&e.body)?;
        stmts.extend(extra);
        let span = loc_to_span(e.loc);

        match &e.op {
            ast::UnaryOp::Not => Ok((
                Expr::UnOp(UnOpExpr { op: UnOp::Not, operand: Box::new(operand), span }),
                stmts,
            )),
            ast::UnaryOp::Neg => Ok((
                Expr::UnOp(UnOpExpr { op: UnOp::Neg, operand: Box::new(operand), span }),
                stmts,
            )),
            ast::UnaryOp::BitNot => Ok((
                Expr::UnOp(UnOpExpr { op: UnOp::BitNot, operand: Box::new(operand), span }),
                stmts,
            )),
            ast::UnaryOp::Delete => Ok((
                Expr::UnOp(UnOpExpr { op: UnOp::Delete, operand: Box::new(operand), span }),
                stmts,
            )),
            ast::UnaryOp::PreIncr | ast::UnaryOp::PreDecr => {
                // ++x  →  x = x + 1; return x
                let binop = if matches!(e.op, ast::UnaryOp::PreIncr) {
                    BinOp::Add
                } else {
                    BinOp::Sub
                };
                let one = Expr::Lit(Lit::one(span));
                let rhs = Expr::BinOp(BinOpExpr {
                    op: binop,
                    lhs: Box::new(operand.clone()),
                    rhs: Box::new(one),
                    overflow: OverflowSemantics::Checked,
                    span,
                });
                stmts.push(Stmt::Assign(AssignStmt { lhs: operand.clone(), rhs, span }));
                Ok((operand, stmts))
            }
            ast::UnaryOp::PostIncr | ast::UnaryOp::PostDecr => {
                // x++  →  tmp = x; x = x + 1; return tmp
                let ty = operand.typ();
                let tmp_name = self.fresh_var_name();
                let tmp_var = Expr::Var(VarExpr::new(tmp_name.clone(), ty.clone(), span));
                stmts.push(Stmt::LocalVar(LocalVarStmt {
                    vars: vec![Some(LocalVarDecl { name: tmp_name, ty })],
                    init: Some(operand.clone()),
                    span,
                }));
                let binop = if matches!(e.op, ast::UnaryOp::PostIncr) {
                    BinOp::Add
                } else {
                    BinOp::Sub
                };
                let one = Expr::Lit(Lit::one(span));
                let rhs = Expr::BinOp(BinOpExpr {
                    op: binop,
                    lhs: Box::new(operand.clone()),
                    rhs: Box::new(one),
                    overflow: OverflowSemantics::Checked,
                    span,
                });
                stmts.push(Stmt::Assign(AssignStmt { lhs: operand, rhs, span }));
                Ok((tmp_var, stmts))
            }
        }
    }

    //-------------------------------------------------
    // Binary expression
    //-------------------------------------------------

    fn lower_binary_expr(&mut self, e: &ast::BinaryExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let (lhs, extra) = self.lower_expr(&e.left)?;
        stmts.extend(extra);
        let (rhs, extra) = self.lower_expr(&e.right)?;
        stmts.extend(extra);
        let op = self.lower_binary_op(&e.operator)?;
        let span = loc_to_span(e.loc);
        let expr = Expr::BinOp(BinOpExpr {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            overflow: OverflowSemantics::Checked,
            span,
        });
        Ok((expr, stmts))
    }

    fn lower_binary_op(&self, op: &ast::BinOp) -> Result<BinOp> {
        match op {
            ast::BinOp::Add => Ok(BinOp::Add),
            ast::BinOp::Sub => Ok(BinOp::Sub),
            ast::BinOp::Mul => Ok(BinOp::Mul),
            ast::BinOp::Div => Ok(BinOp::Div),
            ast::BinOp::Mod => Ok(BinOp::Mod),
            ast::BinOp::Power => Ok(BinOp::Pow),
            ast::BinOp::BitAnd => Ok(BinOp::BitAnd),
            ast::BinOp::BitOr => Ok(BinOp::BitOr),
            ast::BinOp::BitXor => Ok(BinOp::BitXor),
            ast::BinOp::Shl => Ok(BinOp::Shl),
            ast::BinOp::Shr => Ok(BinOp::Shr),
            ast::BinOp::Eq => Ok(BinOp::Eq),
            ast::BinOp::Ne => Ok(BinOp::Ne),
            ast::BinOp::Lt => Ok(BinOp::Lt),
            ast::BinOp::Le => Ok(BinOp::Le),
            ast::BinOp::Gt => Ok(BinOp::Gt),
            ast::BinOp::Ge => Ok(BinOp::Ge),
            ast::BinOp::And => Ok(BinOp::And),
            ast::BinOp::Or => Ok(BinOp::Or),
            _ => fail!("Unknown binary operator: {}", op),
        }
    }

    //-------------------------------------------------
    // Function call expression
    //-------------------------------------------------

    fn lower_call_expr(&mut self, e: &ast::CallExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let ty = self.lower_type(&e.typ)?;
        let span = loc_to_span(e.loc);

        // ── Fix 2: Type conversion: address(0), uint256(x) → TypeCast ──
        if e.kind == ast::CallKind::TypeConversionCall {
            let (args, extra) = self.lower_call_args_exprs(&e.args)?;
            stmts.extend(extra);
            let mut positional = args.into_positional();
            if positional.len() == 1 {
                let inner = positional.remove(0);
                return Ok((
                    Expr::TypeCast(TypeCastExpr { ty, expr: Box::new(inner), span }),
                    stmts,
                ));
            }
            // Multi-arg type conversion — fall through to generic call
        }

        // ── Fix 7: EVM builtin functions ─────────────────────────
        if let ast::Expr::Ident(id) = &*e.callee {
            let name = id.name.base.as_str();
            match name {
                "keccak256" | "sha256" | "ripemd160" | "ecrecover" | "addmod" | "mulmod"
                | "gasleft" | "blockhash" => {
                    let (args, extra) = self.lower_call_args_exprs(&e.args)?;
                    stmts.extend(extra);
                    let mut pos = args.into_positional();
                    let evm = match name {
                        "keccak256" => EvmExpr::Keccak256(EvmKeccak256 {
                            expr: Box::new(pos.remove(0)),
                            loc: Default::default(),
                        }),
                        "sha256" => EvmExpr::Sha256(EvmSha256 {
                            expr: Box::new(pos.remove(0)),
                            loc: Default::default(),
                        }),
                        "ripemd160" => EvmExpr::Ripemd160(EvmRipemd160 {
                            expr: Box::new(pos.remove(0)),
                            loc: Default::default(),
                        }),
                        "ecrecover" => {
                            let hash = Box::new(pos.remove(0));
                            let v = Box::new(pos.remove(0));
                            let r = Box::new(pos.remove(0));
                            let s = Box::new(pos.remove(0));
                            EvmExpr::Ecrecover(EvmEcrecover {
                                hash,
                                v,
                                r,
                                s,
                                loc: Default::default(),
                            })
                        }
                        "addmod" => {
                            let x = Box::new(pos.remove(0));
                            let y = Box::new(pos.remove(0));
                            let k = Box::new(pos.remove(0));
                            EvmExpr::Addmod(EvmAddmod { x, y, k, loc: Default::default() })
                        }
                        "mulmod" => {
                            let x = Box::new(pos.remove(0));
                            let y = Box::new(pos.remove(0));
                            let k = Box::new(pos.remove(0));
                            EvmExpr::Mulmod(EvmMulmod { x, y, k, loc: Default::default() })
                        }
                        "gasleft" => EvmExpr::Gasleft(EvmGasleft { loc: Default::default() }),
                        "blockhash" => EvmExpr::Blockhash(EvmBlockhash {
                            expr: Box::new(pos.remove(0)),
                            loc: Default::default(),
                        }),
                        _ => unreachable!(),
                    };
                    return Ok((Expr::Dialect(DialectExpr::Evm(evm)), stmts));
                }
                _ => {}
            }
        }

        // ── Fix 8: abi.* builtins ────────────────────────────────
        if let ast::Expr::Member(mem) = &*e.callee {
            if let ast::Expr::Ident(base) = &*mem.base {
                if base.name.base.as_str() == "abi" {
                    let method = mem.member.to_string();
                    let (args, extra) = self.lower_call_args_exprs(&e.args)?;
                    stmts.extend(extra);
                    let pos = args.into_positional();
                    let evm = match method.as_str() {
                        "encode" => Some(EvmExpr::AbiEncode(EvmAbiEncode {
                            args: pos,
                            loc: Default::default(),
                        })),
                        "encodePacked" => Some(EvmExpr::AbiEncodePacked(EvmAbiEncodePacked {
                            args: pos,
                            loc: Default::default(),
                        })),
                        "decode" => {
                            let mut p = pos;
                            let data = if p.is_empty() {
                                Expr::Lit(Lit::String(StringLit::new(String::new(), span)))
                            } else {
                                p.remove(0)
                            };
                            // The rest of the args represent the types to decode to
                            // In the AST, `abi.decode(data, (uint, address))` passes types
                            // We use the expression's return type from the AST
                            let types = match &ty {
                                Type::Tuple(ts) => ts.clone(),
                                Type::None => vec![],
                                t => vec![t.clone()],
                            };
                            Some(EvmExpr::AbiDecode(EvmAbiDecode {
                                data: Box::new(data),
                                types,
                                loc: Default::default(),
                            }))
                        }
                        "encodeWithSelector" => {
                            let mut p = pos;
                            let selector = Box::new(if p.is_empty() {
                                Expr::Lit(Lit::String(StringLit::new(String::new(), span)))
                            } else {
                                p.remove(0)
                            });
                            Some(EvmExpr::AbiEncodeWithSelector(EvmAbiEncodeWithSelector {
                                selector,
                                args: p,
                                loc: Default::default(),
                            }))
                        }
                        "encodeWithSignature" => {
                            let mut p = pos;
                            let signature = Box::new(if p.is_empty() {
                                Expr::Lit(Lit::String(StringLit::new(String::new(), span)))
                            } else {
                                p.remove(0)
                            });
                            Some(EvmExpr::AbiEncodeWithSignature(EvmAbiEncodeWithSignature {
                                signature,
                                args: p,
                                loc: Default::default(),
                            }))
                        }
                        "encodeCall" => {
                            let mut p = pos;
                            let func = Box::new(if p.is_empty() {
                                Expr::Lit(Lit::String(StringLit::new(String::new(), span)))
                            } else {
                                p.remove(0)
                            });
                            Some(EvmExpr::AbiEncodeCall(EvmAbiEncodeCall {
                                func,
                                args: p,
                                loc: Default::default(),
                            }))
                        }
                        _ => None,
                    };
                    if let Some(evm) = evm {
                        return Ok((Expr::Dialect(DialectExpr::Evm(evm)), stmts));
                    }
                }
            }

            // ── Fix 9: addr.transfer(amt) / addr.send(amt) ─────────
            let method = mem.member.to_string();
            if method == "transfer" || method == "send" {
                // Only intercept single-argument calls (address.transfer/send take 1 arg)
                if let ast::CallArgs::Unnamed(uargs) = &e.args {
                    if uargs.len() == 1 {
                        let (base_e, extra) = self.lower_expr(&mem.base)?;
                        stmts.extend(extra);
                        let (args, extra) = self.lower_call_args_exprs(&e.args)?;
                        stmts.extend(extra);
                        let mut pos = args.into_positional();
                        let amount = pos.remove(0);
                        let evm = if method == "transfer" {
                            EvmExpr::Transfer(EvmTransfer {
                                target: Box::new(base_e),
                                amount: Box::new(amount),
                                loc: Default::default(),
                            })
                        } else {
                            EvmExpr::Send(EvmSend {
                                target: Box::new(base_e),
                                value: Box::new(amount),
                                loc: Default::default(),
                            })
                        };
                        return Ok((Expr::Dialect(DialectExpr::Evm(evm)), stmts));
                    }
                }
            }
        }

        let (callee, extra) = self.lower_expr(&e.callee)?;
        stmts.extend(extra);
        let (args, extra) = self.lower_call_args_exprs(&e.args)?;
        stmts.extend(extra);
        let expr = Expr::FunctionCall(CallExpr { callee: Box::new(callee), args, ty, span });
        Ok((expr, stmts))
    }

    //-------------------------------------------------
    // Function call with options expression
    //-------------------------------------------------

    /// Lower `addr.call{value: v, gas: g}(data)` / `addr.delegatecall(data)`
    /// etc.
    fn lower_call_opts_expr(&mut self, e: &ast::CallOptsExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let span = loc_to_span(e.loc);

        // The callee is typically a CallExpr wrapping a MemberExpr:
        //   `addr.call{value: x}(data)` → callee=Call(Member(addr, "call"), [data])
        // Or it could be a direct MemberExpr without further call:
        //   We need to extract target, method name, and call args.

        // Extract call-options value/gas
        let mut opt_value: Option<Box<Expr>> = None;
        let mut opt_gas: Option<Box<Expr>> = None;
        for opt in &e.call_opts {
            let (val, extra) = self.lower_expr(&opt.value)?;
            stmts.extend(extra);
            match opt.name.as_str() {
                "value" => opt_value = Some(Box::new(val)),
                "gas" => opt_gas = Some(Box::new(val)),
                _ => {} // ignore unknown options
            }
        }

        // The inner callee should be a Call(Member(target, method), args)
        match &*e.callee {
            ast::Expr::Call(call) => {
                // Extract the member access to get target + method name.
                if let ast::Expr::Member(mem) = &*call.callee {
                    let method = mem.member.to_string();
                    let (target, extra) = self.lower_expr(&mem.base)?;
                    stmts.extend(extra);
                    let (args, extra) = self.lower_call_args_exprs(&call.args)?;
                    stmts.extend(extra);

                    // First arg is the data payload for low-level calls
                    let data = args
                        .into_positional()
                        .into_iter()
                        .next()
                        .unwrap_or(Expr::Lit(Lit::String(StringLit::new(String::new(), span))));

                    let evm = match method.as_str() {
                        "delegatecall" => EvmExpr::Delegatecall(EvmDelegatecall {
                            target: Box::new(target),
                            data: Box::new(data),
                            loc: span.unwrap_or_default(),
                        }),
                        "call" | "staticcall" => EvmExpr::LowLevelCall(EvmLowLevelCall {
                            target: Box::new(target),
                            data: Box::new(data),
                            value: opt_value,
                            gas: opt_gas,
                            loc: span.unwrap_or_default(),
                        }),
                        _ => {
                            // Fallback: emit as a generic low-level call
                            EvmExpr::LowLevelCall(EvmLowLevelCall {
                                target: Box::new(target),
                                data: Box::new(data),
                                value: opt_value,
                                gas: opt_gas,
                                loc: span.unwrap_or_default(),
                            })
                        }
                    };
                    let expr = Expr::Dialect(DialectExpr::Evm(evm));
                    return Ok((expr, stmts));
                }

                // Non-member call with options — fall through to generic call
                let (callee, extra) = self.lower_expr(&call.callee)?;
                stmts.extend(extra);
                let ty = self.lower_type(&call.typ)?;
                let (args, extra) = self.lower_call_args_exprs(&call.args)?;
                stmts.extend(extra);
                let expr =
                    Expr::FunctionCall(CallExpr { callee: Box::new(callee), args, ty, span });
                Ok((expr, stmts))
            }
            _ => {
                // Unexpected shape — lower as a generic expression
                let (expr, extra) = self.lower_expr(&e.callee)?;
                stmts.extend(extra);
                Ok((expr, stmts))
            }
        }
    }

    fn lower_call_args_exprs(&mut self, args: &ast::CallArgs) -> Result<(CallArgs, Vec<Stmt>)> {
        match args {
            ast::CallArgs::Unnamed(exprs) => {
                let mut stmts = vec![];
                let mut result = vec![];
                for e in exprs {
                    let (ne, extra) = self.lower_expr(e)?;
                    stmts.extend(extra);
                    result.push(ne);
                }
                Ok((CallArgs::Positional(result), stmts))
            }
            ast::CallArgs::Named(named) => {
                // Preserve named args — will be converted to positional at SIR → CIR level.
                let mut stmts = vec![];
                let mut result = vec![];
                for n in named {
                    let (ne, extra) = self.lower_expr(&n.value)?;
                    stmts.extend(extra);
                    result.push(NamedArg { name: n.name.clone(), value: ne });
                }
                Ok((CallArgs::Named(result), stmts))
            }
        }
    }

    //-------------------------------------------------
    // Tuple expression
    //-------------------------------------------------

    fn lower_tuple_expr(&mut self, e: &ast::TupleExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let ty = self.lower_type(&e.typ)?;
        let mut elems = vec![];
        for elem in &e.elems {
            match elem {
                Some(ex) => {
                    let (ne, extra) = self.lower_expr(ex)?;
                    stmts.extend(extra);
                    elems.push(Some(ne));
                }
                None => elems.push(None),
            }
        }
        let expr = Expr::Tuple(TupleExpr { elems, ty, span: loc_to_span(e.loc) });
        Ok((expr, stmts))
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------

    fn lower_index_expr(&mut self, e: &ast::IndexExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let ty = self.lower_type(&e.typ)?;
        let (base, extra) = self.lower_expr(&e.base_expr)?;
        stmts.extend(extra);
        let index = match &e.index {
            Some(idx) => {
                let (ne, extra) = self.lower_expr(idx)?;
                stmts.extend(extra);
                Some(Box::new(ne))
            }
            None => None,
        };
        let expr = Expr::IndexAccess(IndexAccessExpr {
            base: Box::new(base),
            index,
            ty,
            span: loc_to_span(e.loc),
        });
        Ok((expr, stmts))
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------

    fn lower_slice_expr(&mut self, e: &ast::SliceExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let ty = self.lower_type(&e.typ)?;
        let (base, extra) = self.lower_expr(&e.base_expr)?;
        stmts.extend(extra);
        let expr = Expr::IndexAccess(IndexAccessExpr {
            base: Box::new(base),
            index: None,
            ty,
            span: loc_to_span(e.loc),
        });
        Ok((expr, stmts))
    }

    //-------------------------------------------------
    // Member access expression
    //-------------------------------------------------

    fn lower_member_expr(&mut self, e: &ast::MemberExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let ty = self.lower_type(&e.typ)?;
        let member = e.member.to_string();
        let span = loc_to_span(e.loc);

        // Check for type query pattern: type(X).min, type(X).max
        if let ast::Expr::Call(call) = &*e.base {
            if call.callee.to_string() == "type" {
                let fname = match &call.args {
                    ast::CallArgs::Unnamed(args) => format!("{member}__type__{}", args[0]),
                    _ => fail!("Expected unnamed args in type query"),
                };
                // Fix 6: give callee a proper Function type
                let callee_ty = Type::Function { params: vec![], returns: vec![ty.clone()] };
                let callee = Expr::Var(VarExpr::new(fname, callee_ty, span));
                let expr = Expr::FunctionCall(CallExpr {
                    callee: Box::new(callee),
                    args: CallArgs::Positional(vec![]),
                    ty,
                    span,
                });
                return Ok((expr, stmts));
            }
        }

        // ── EVM global member accesses ──────────────────────────────
        if let ast::Expr::Ident(base_id) = &*e.base {
            let base_name = base_id.name.base.as_str();
            let evm_expr = match (base_name, member.as_str()) {
                ("msg", "sender") => {
                    Some(EvmExpr::MsgSender(EvmMsgSender { loc: Default::default() }))
                }
                ("msg", "value") => {
                    Some(EvmExpr::MsgValue(EvmMsgValue { loc: Default::default() }))
                }
                ("msg", "data") => Some(EvmExpr::MsgData(EvmMsgData { loc: Default::default() })),
                ("msg", "sig") => Some(EvmExpr::MsgSig(EvmMsgSig { loc: Default::default() })),
                ("tx", "origin") => {
                    Some(EvmExpr::TxOrigin(EvmTxOrigin { loc: Default::default() }))
                }
                ("block", "timestamp") => {
                    Some(EvmExpr::Timestamp(EvmTimestamp { loc: Default::default() }))
                }
                ("block", "number") => {
                    Some(EvmExpr::BlockNumber(EvmBlockNumber { loc: Default::default() }))
                }
                ("block", "difficulty") | ("block", "prevrandao") => {
                    Some(EvmExpr::BlockDifficulty(EvmBlockDifficulty { loc: Default::default() }))
                }
                ("block", "gaslimit") => {
                    Some(EvmExpr::BlockGaslimit(EvmBlockGaslimit { loc: Default::default() }))
                }
                ("block", "coinbase") => {
                    Some(EvmExpr::BlockCoinbase(EvmBlockCoinbase { loc: Default::default() }))
                }
                ("block", "chainid") => {
                    Some(EvmExpr::BlockChainid(EvmBlockChainid { loc: Default::default() }))
                }
                ("block", "basefee") => {
                    Some(EvmExpr::BlockBasefee(EvmBlockBasefee { loc: Default::default() }))
                }
                _ => None,
            };
            if let Some(evm) = evm_expr {
                return Ok((Expr::Dialect(DialectExpr::Evm(evm)), stmts));
            }
        }

        let (base, extra) = self.lower_expr(&e.base)?;
        stmts.extend(extra);
        let expr =
            Expr::FieldAccess(FieldAccessExpr { base: Box::new(base), field: member, ty, span });
        Ok((expr, stmts))
    }

    //-------------------------------------------------
    // Conditional expression
    //-------------------------------------------------

    fn lower_conditional_expr(&mut self, e: &ast::ConditionalExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut stmts = vec![];
        let (cond, extra) = self.lower_expr(&e.cond)?;
        stmts.extend(extra);
        let (then_expr, extra) = self.lower_expr(&e.true_br)?;
        stmts.extend(extra);
        let (else_expr, extra) = self.lower_expr(&e.false_br)?;
        stmts.extend(extra);
        let span = loc_to_span(e.loc);
        let expr = Expr::Ternary(TernaryExpr {
            cond: Box::new(cond),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
            span,
        });
        Ok((expr, stmts))
    }

    //-------------------------------------------------
    // Inline array expression
    //-------------------------------------------------

    fn lower_inline_array_expr(&mut self, e: &ast::InlineArrayExpr) -> Result<(Expr, Vec<Stmt>)> {
        let ty = self.lower_type(&e.typ)?;
        let mut stmts = vec![];
        let mut elems = vec![];
        for elem in &e.elems {
            let (ne, extra) = self.lower_expr(elem)?;
            stmts.extend(extra);
            elems.push(Some(ne));
        }
        let expr = Expr::Tuple(TupleExpr { elems, ty, span: loc_to_span(e.loc) });
        Ok((expr, stmts))
    }

    //-------------------------------------------------
    // New expression
    //-------------------------------------------------

    fn lower_new_expr(&mut self, e: &ast::NewExpr) -> Result<(Expr, Vec<Stmt>)> {
        let ty = self.lower_type(&e.typ)?;
        let span = loc_to_span(e.loc);
        let name = format!("new__{ty}");
        let callee_ty = Type::Function { params: vec![], returns: vec![ty.clone()] };
        let callee = Expr::Var(VarExpr::new(name, callee_ty, span));
        Ok((callee, vec![]))
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    fn lower_type(&mut self, typ: &ast::Type) -> Result<Type> {
        match typ {
            ast::Type::Bool => Ok(Type::Bool),
            ast::Type::Int(t) => Ok(Type::from_int(t.bitwidth, t.is_signed)),
            ast::Type::Fixed(_) => Ok(Type::Si256), // Fixed-point approximation
            ast::Type::String(_) => Ok(Type::String),
            ast::Type::Address(t) => {
                if t.payable {
                    Ok(Type::Dialect(DialectType::Evm(EvmType::AddressPayable)))
                } else {
                    Ok(Type::Dialect(DialectType::Evm(EvmType::Address)))
                }
            }
            ast::Type::Bytes(t) => match t.length {
                Some(n) => Ok(Type::FixedBytes(n)),
                None => Ok(Type::Bytes),
            },
            ast::Type::Array(t) => {
                let base = self.lower_type(&t.base)?;
                match &t.length {
                    Some(len) => {
                        let n = len.to_string().parse::<u64>().unwrap_or(0);
                        Ok(Type::FixedArray(Box::new(base), n))
                    }
                    None => Ok(Type::Array(Box::new(base))),
                }
            }
            ast::Type::Slice(t) => {
                let base = self.lower_type(&t.base)?;
                Ok(Type::Array(Box::new(base)))
            }
            ast::Type::Struct(t) => Ok(Type::TypeRef(t.name.to_string())),
            ast::Type::Enum(t) => Ok(Type::TypeRef(t.name.to_string())),
            ast::Type::Module(name) => Ok(Type::TypeRef(name.clone())),
            ast::Type::Tuple(t) => {
                let elems = t
                    .elems
                    .iter()
                    .map(|e| match e {
                        Some(ty) => self.lower_type(ty),
                        None => Ok(Type::None),
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Tuple(elems))
            }
            ast::Type::Func(t) => {
                let params = t
                    .params
                    .iter()
                    .map(|p| self.lower_type(p))
                    .collect::<Result<Vec<_>>>()?;
                let returns = t
                    .returns
                    .iter()
                    .map(|r| self.lower_type(r))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Function { params, returns })
            }
            ast::Type::Mapping(t) => {
                let key = self.lower_type(&t.key)?;
                let value = self.lower_type(&t.value)?;
                Ok(Type::Map(Box::new(key), Box::new(value)))
            }
            ast::Type::UserDefined(_) => fail!("User-defined type must be eliminated!"),
            ast::Type::Contract(t) => Ok(Type::TypeRef(t.name.to_string())),
            ast::Type::Magic(t) => match t {
                ast::MagicType::BlockType => Ok(Type::TypeRef("block".to_string())),
                ast::MagicType::MessageType => Ok(Type::TypeRef("msg".to_string())),
                ast::MagicType::TxnType => Ok(Type::TypeRef("tx".to_string())),
                ast::MagicType::ABIType => Ok(Type::TypeRef("abi".to_string())),
                ast::MagicType::MetaType(base) => {
                    let nbase = self.lower_type(base)?;
                    Ok(Type::TypeRef(format!("type({})", nbase)))
                }
            },
        }
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn lower_lit(&mut self, lit: &ast::Lit) -> Result<Lit> {
        match lit {
            ast::Lit::Bool(l) => Ok(Lit::Bool(BoolLit::new(l.value, loc_to_span(l.loc)))),
            ast::Lit::Num(l) => {
                let mut num = self.lower_num(&l.value)?;
                if let Some(unit) = &l.unit {
                    let multiplier = match unit {
                        ast::NumUnit::Wei | ast::NumUnit::Seconds => num_bigint::BigInt::from(1),
                        ast::NumUnit::Minutes => num_bigint::BigInt::from(60),
                        ast::NumUnit::Hours => num_bigint::BigInt::from(3600),
                        ast::NumUnit::Days => num_bigint::BigInt::from(86400),
                        ast::NumUnit::Weeks => num_bigint::BigInt::from(604800),
                        ast::NumUnit::Years => num_bigint::BigInt::from(31536000),
                        ast::NumUnit::Gwei => "1000000000".parse::<num_bigint::BigInt>().unwrap(),
                        ast::NumUnit::Szabo => "1000000000000".parse::<num_bigint::BigInt>().unwrap(),
                        ast::NumUnit::Finney => "1000000000000000".parse::<num_bigint::BigInt>().unwrap(),
                        ast::NumUnit::Ether => "1000000000000000000".parse::<num_bigint::BigInt>().unwrap(),
                    };
                    match &mut num {
                        Num::Int(n) => n.value *= multiplier,
                        Num::Fixed(_) => fail!("Fixed point numbers cannot have a unit"),
                        Num::Hex(_) => fail!("Hex numbers cannot have a unit"),
                    }
                }
                Ok(Lit::Num(NumLit::new(num, loc_to_span(l.loc))))
            }
            ast::Lit::String(l) => {
                Ok(Lit::String(StringLit::new(l.value.clone(), loc_to_span(l.loc))))
            }
            ast::Lit::Hex(l) => Ok(Lit::Hex(HexLit::new(l.value.clone(), loc_to_span(l.loc)))),
            ast::Lit::Unicode(l) => {
                Ok(Lit::Unicode(UnicodeLit::new(l.value.clone(), loc_to_span(l.loc))))
            }
        }
    }

    fn lower_num(&mut self, num: &ast::Num) -> Result<Num> {
        match num {
            ast::Num::Int(n) => {
                let ty = self.lower_type(&n.typ)?;
                Ok(Num::Int(IntNum::new(n.value.clone(), ty)))
            }
            ast::Num::FixedNum(n) => {
                let ty = self.lower_type(&n.typ)?;
                Ok(Num::Fixed(FixedNum::new(n.value, ty)))
            }
            ast::Num::Hex(n) => {
                let ty = self.lower_type(&n.typ)?;
                Ok(Num::Hex(HexNum::new(n.value.clone(), ty)))
            }
        }
    }
}

impl Default for Lowerer {
    fn default() -> Self {
        Self::new()
    }
}
