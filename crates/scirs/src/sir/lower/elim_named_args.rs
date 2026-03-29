//! Eliminate named arguments — convert `f({a: x, b: y})` to positional `f(x,
//! y)`.
//!
//! The pass collects parameter names for every function, event and error in
//! the module, then rewrites `CallArgs::Named` to `CallArgs::Positional` in
//! the correct order by matching argument names against parameter names.

use crate::sir;
use crate::sir::lower::CirLowerError;
use std::collections::HashMap;

/// Convert all named-argument call-sites to positional form.
pub fn run(module: &sir::Module) -> Result<sir::Module, CirLowerError> {
    // Phase 1: collect param names for all known callables.
    let param_names = collect_param_names(module);

    // Phase 2: rewrite named args to positional.
    let decls = module
        .decls
        .iter()
        .map(|d| rewrite_decl(d, &param_names))
        .collect();

    Ok(sir::Module { id: module.id.clone(), attrs: module.attrs.clone(), decls })
}

// ─── Phase 1: collect param names ────────────────────────────────────────────

fn collect_param_names(module: &sir::Module) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();

    for decl in &module.decls {
        if let sir::Decl::Contract(c) = decl {
            collect_from_contract(c, &mut map);
        }
    }
    map
}

fn collect_from_contract(contract: &sir::ContractDecl, map: &mut HashMap<String, Vec<String>>) {
    for member in &contract.members {
        match member {
            sir::MemberDecl::Function(f) => {
                let names: Vec<String> = f.params.iter().map(|p| p.name.clone()).collect();
                let qualified = format!("{}.{}", contract.name, f.name);
                map.insert(qualified, names.clone());
                map.entry(f.name.clone()).or_insert(names);
            }
            sir::MemberDecl::Dialect(sir::DialectMemberDecl::Evm(d)) => {
                collect_from_evm_member(d, &contract.name, map);
            }
            _ => {}
        }
    }
}

fn collect_from_evm_member(
    d: &sir::dialect::evm::EvmMemberDecl,
    contract_name: &str,
    map: &mut HashMap<String, Vec<String>>,
) {
    use sir::dialect::evm::EvmMemberDecl;
    match d {
        EvmMemberDecl::EventDef(e) => {
            let names: Vec<String> = e
                .params
                .iter()
                .map(|(n, _): &(String, _)| n.clone())
                .collect();
            let qualified = format!("{}.{}", contract_name, e.name);
            map.insert(qualified, names.clone());
            map.entry(e.name.clone()).or_insert(names);
        }
        EvmMemberDecl::ErrorDef(e) => {
            let names: Vec<String> = e
                .params
                .iter()
                .map(|(n, _): &(String, _)| n.clone())
                .collect();
            let qualified = format!("{}.{}", contract_name, e.name);
            map.insert(qualified, names.clone());
            map.entry(e.name.clone()).or_insert(names);
        }
        EvmMemberDecl::ModifierDef(e) => {
            let names: Vec<String> = e
                .params
                .iter()
                .map(|(n, _): &(String, _)| n.clone())
                .collect();
            let qualified = format!("{}.{}", contract_name, e.name);
            map.insert(qualified, names.clone());
            map.entry(e.name.clone()).or_insert(names);
        }
        _ => {}
    }
}

// ─── Phase 2: rewrite named args ─────────────────────────────────────────────

fn rewrite_decl(decl: &sir::Decl, map: &HashMap<String, Vec<String>>) -> sir::Decl {
    match decl {
        sir::Decl::Contract(c) => sir::Decl::Contract(rewrite_contract(c, map)),
        sir::Decl::Dialect(d) => sir::Decl::Dialect(d.clone()),
    }
}

fn rewrite_contract(
    contract: &sir::ContractDecl,
    map: &HashMap<String, Vec<String>>,
) -> sir::ContractDecl {
    let members = contract
        .members
        .iter()
        .map(|m| rewrite_member(m, map))
        .collect();
    sir::ContractDecl { members, ..contract.clone() }
}

fn rewrite_member(
    member: &sir::MemberDecl,
    map: &HashMap<String, Vec<String>>,
) -> sir::MemberDecl {
    match member {
        sir::MemberDecl::Function(f) => sir::MemberDecl::Function(rewrite_function(f, map)),
        sir::MemberDecl::Storage(s) => {
            let init = s.init.as_ref().map(|e| rewrite_expr(e, map));
            sir::MemberDecl::Storage(sir::StorageDecl { init, ..s.clone() })
        }
        sir::MemberDecl::GlobalInvariant(e) => {
            sir::MemberDecl::GlobalInvariant(rewrite_expr(e, map))
        }
        _ => member.clone(),
    }
}

fn rewrite_function(
    func: &sir::FunctionDecl,
    map: &HashMap<String, Vec<String>>,
) -> sir::FunctionDecl {
    let body = func.body.as_ref().map(|stmts| rewrite_stmts(stmts, map));
    let modifier_invocs = func
        .modifier_invocs
        .iter()
        .map(|m| sir::ModifierInvoc {
            name: m.name.clone(),
            args: rewrite_exprs(&m.args, map),
            span: m.span.clone(),
        })
        .collect();
    sir::FunctionDecl { body, modifier_invocs, ..func.clone() }
}

fn rewrite_stmts(stmts: &[sir::Stmt], map: &HashMap<String, Vec<String>>) -> Vec<sir::Stmt> {
    stmts.iter().map(|s| rewrite_stmt(s, map)).collect()
}

fn rewrite_stmt(stmt: &sir::Stmt, map: &HashMap<String, Vec<String>>) -> sir::Stmt {
    match stmt {
        sir::Stmt::LocalVar(s) => sir::Stmt::LocalVar(sir::LocalVarStmt {
            init: s.init.as_ref().map(|e| rewrite_expr(e, map)),
            ..s.clone()
        }),
        sir::Stmt::Assign(s) => sir::Stmt::Assign(sir::AssignStmt {
            lhs: rewrite_expr(&s.lhs, map),
            rhs: rewrite_expr(&s.rhs, map),
            ..s.clone()
        }),
        sir::Stmt::AugAssign(s) => sir::Stmt::AugAssign(sir::AugAssignStmt {
            lhs: rewrite_expr(&s.lhs, map),
            rhs: rewrite_expr(&s.rhs, map),
            ..s.clone()
        }),
        sir::Stmt::Expr(s) => {
            sir::Stmt::Expr(sir::ExprStmt { expr: rewrite_expr(&s.expr, map), ..s.clone() })
        }
        sir::Stmt::If(s) => sir::Stmt::If(sir::IfStmt {
            cond: rewrite_expr(&s.cond, map),
            then_body: rewrite_stmts(&s.then_body, map),
            else_body: s.else_body.as_ref().map(|b| rewrite_stmts(b, map)),
            ..s.clone()
        }),
        sir::Stmt::While(s) => sir::Stmt::While(sir::WhileStmt {
            cond: rewrite_expr(&s.cond, map),
            body: rewrite_stmts(&s.body, map),
            invariant: s.invariant.as_ref().map(|e| rewrite_expr(e, map)),
            ..s.clone()
        }),
        sir::Stmt::For(s) => sir::Stmt::For(sir::ForStmt {
            init: s.init.as_ref().map(|i| Box::new(rewrite_stmt(i, map))),
            cond: s.cond.as_ref().map(|e| rewrite_expr(e, map)),
            update: s.update.as_ref().map(|u| Box::new(rewrite_stmt(u, map))),
            body: rewrite_stmts(&s.body, map),
            invariant: s.invariant.as_ref().map(|e| rewrite_expr(e, map)),
            ..s.clone()
        }),
        sir::Stmt::Return(s) => sir::Stmt::Return(sir::ReturnStmt {
            value: s.value.as_ref().map(|e| rewrite_expr(e, map)),
            ..s.clone()
        }),
        sir::Stmt::Revert(s) => {
            sir::Stmt::Revert(sir::RevertStmt { args: rewrite_exprs(&s.args, map), ..s.clone() })
        }
        sir::Stmt::Assert(s) => sir::Stmt::Assert(sir::AssertStmt {
            cond: rewrite_expr(&s.cond, map),
            message: s.message.as_ref().map(|e| rewrite_expr(e, map)),
            ..s.clone()
        }),
        sir::Stmt::Block(stmts) => sir::Stmt::Block(rewrite_stmts(stmts, map)),
        sir::Stmt::Break | sir::Stmt::Continue | sir::Stmt::Dialect(_) => stmt.clone(),
    }
}

fn rewrite_expr(expr: &sir::Expr, map: &HashMap<String, Vec<String>>) -> sir::Expr {
    match expr {
        sir::Expr::FunctionCall(e) => {
            let callee = rewrite_expr(&e.callee, map);
            let callee_name = extract_callee_name(&callee);
            let args = match &e.args {
                sir::CallArgs::Positional(args) => {
                    sir::CallArgs::Positional(rewrite_exprs(args, map))
                }
                sir::CallArgs::Named(named) => {
                    let positional = reorder_named_args(named, &callee_name, map);
                    sir::CallArgs::Positional(rewrite_exprs(&positional, map))
                }
            };
            sir::Expr::FunctionCall(sir::CallExpr {
                callee: Box::new(callee),
                args,
                ty: e.ty.clone(),
                span: e.span.clone(),
            })
        }
        sir::Expr::BinOp(e) => sir::Expr::BinOp(sir::BinOpExpr {
            lhs: Box::new(rewrite_expr(&e.lhs, map)),
            rhs: Box::new(rewrite_expr(&e.rhs, map)),
            ..e.clone()
        }),
        sir::Expr::UnOp(e) => sir::Expr::UnOp(sir::UnOpExpr {
            operand: Box::new(rewrite_expr(&e.operand, map)),
            ..e.clone()
        }),
        sir::Expr::IndexAccess(e) => sir::Expr::IndexAccess(sir::IndexAccessExpr {
            base: Box::new(rewrite_expr(&e.base, map)),
            index: e.index.as_ref().map(|i| Box::new(rewrite_expr(i, map))),
            ..e.clone()
        }),
        sir::Expr::FieldAccess(e) => sir::Expr::FieldAccess(sir::FieldAccessExpr {
            base: Box::new(rewrite_expr(&e.base, map)),
            ..e.clone()
        }),
        sir::Expr::TypeCast(e) => sir::Expr::TypeCast(sir::TypeCastExpr {
            expr: Box::new(rewrite_expr(&e.expr, map)),
            ..e.clone()
        }),
        sir::Expr::Ternary(e) => sir::Expr::Ternary(sir::TernaryExpr {
            cond: Box::new(rewrite_expr(&e.cond, map)),
            then_expr: Box::new(rewrite_expr(&e.then_expr, map)),
            else_expr: Box::new(rewrite_expr(&e.else_expr, map)),
            ..e.clone()
        }),
        sir::Expr::Tuple(e) => sir::Expr::Tuple(sir::TupleExpr {
            elems: e
                .elems
                .iter()
                .map(|opt| opt.as_ref().map(|ex| rewrite_expr(ex, map)))
                .collect(),
            ..e.clone()
        }),
        sir::Expr::Old(inner) => sir::Expr::Old(Box::new(rewrite_expr(inner, map))),
        sir::Expr::Forall { var, ty, body } => sir::Expr::Forall {
            var: var.clone(),
            ty: ty.clone(),
            body: Box::new(rewrite_expr(body, map)),
        },
        sir::Expr::Exists { var, ty, body } => sir::Expr::Exists {
            var: var.clone(),
            ty: ty.clone(),
            body: Box::new(rewrite_expr(body, map)),
        },
        sir::Expr::Var(_) | sir::Expr::Lit(_) | sir::Expr::Result(_) | sir::Expr::Dialect(_) => {
            expr.clone()
        }
    }
}

fn rewrite_exprs(exprs: &[sir::Expr], map: &HashMap<String, Vec<String>>) -> Vec<sir::Expr> {
    exprs.iter().map(|e| rewrite_expr(e, map)).collect()
}

/// Extract a best-effort function name from the callee expression.
fn extract_callee_name(callee: &sir::Expr) -> String {
    match callee {
        sir::Expr::Var(v) => v.name.clone(),
        sir::Expr::FieldAccess(fa) => fa.field.clone(),
        _ => String::new(),
    }
}

/// Reorder named args to match parameter order.
/// Falls back to declaration order if the function is unknown.
fn reorder_named_args(
    named: &[sir::NamedArg],
    callee_name: &str,
    map: &HashMap<String, Vec<String>>,
) -> Vec<sir::Expr> {
    if let Some(param_order) = map.get(callee_name) {
        let mut result = Vec::with_capacity(param_order.len());
        for param_name in param_order {
            if let Some(arg) = named.iter().find(|a| &a.name == param_name) {
                result.push(arg.value.clone());
            }
        }
        if result.len() == named.len() {
            return result;
        }
    }
    // Fallback: use declaration order
    named.iter().map(|a| a.value.clone()).collect()
}
