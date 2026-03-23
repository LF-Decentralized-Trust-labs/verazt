//! Flatten expressions — ensure all call arguments are atoms.
//!
//! Introduces `LocalVar` temporaries for any call argument that is not already
//! an atom (variable or literal), so CIR functions satisfy the invariant that
//! call arguments are atomic.
//!
//! Example:
//! ```text
//! f(g(x), 1)
//! →
//! let tmp1 = g(x);
//! f(tmp1, 1)
//! ```

use crate::sir;
use crate::sir::lower::CirLowerError;

/// Flatten all call arguments to atoms.
pub fn run(module: &sir::Module) -> Result<sir::Module, CirLowerError> {
    let decls = module
        .decls
        .iter()
        .map(flatten_decl)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(sir::Module { id: module.id.clone(), attrs: module.attrs.clone(), decls })
}

fn flatten_decl(decl: &sir::Decl) -> Result<sir::Decl, CirLowerError> {
    match decl {
        sir::Decl::Contract(c) => Ok(sir::Decl::Contract(flatten_contract(c)?)),
        sir::Decl::Dialect(d) => Ok(sir::Decl::Dialect(d.clone())),
    }
}

fn flatten_contract(contract: &sir::ContractDecl) -> Result<sir::ContractDecl, CirLowerError> {
    let members = contract
        .members
        .iter()
        .map(flatten_member)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(sir::ContractDecl { members, ..contract.clone() })
}

fn flatten_member(member: &sir::MemberDecl) -> Result<sir::MemberDecl, CirLowerError> {
    match member {
        sir::MemberDecl::Function(f) => Ok(sir::MemberDecl::Function(flatten_function(f)?)),
        sir::MemberDecl::Storage(s) => {
            let (pre, init) = match &s.init {
                Some(e) => {
                    let (p, e2) = flatten_expr(e, &mut 0);
                    (p, Some(e2))
                }
                None => (vec![], None),
            };
            // pre-stmts in storage init are unusual; just drop them (no body to insert
            // into).
            let _ = pre;
            Ok(sir::MemberDecl::Storage(sir::StorageDecl { init, ..s.clone() }))
        }
        _ => Ok(member.clone()),
    }
}

fn flatten_function(func: &sir::FunctionDecl) -> Result<sir::FunctionDecl, CirLowerError> {
    let body = match &func.body {
        Some(stmts) => {
            let mut counter: usize = 0;
            Some(flatten_stmts(stmts, &mut counter))
        }
        None => None,
    };
    Ok(sir::FunctionDecl { body, ..func.clone() })
}

fn flatten_stmts(stmts: &[sir::Stmt], counter: &mut usize) -> Vec<sir::Stmt> {
    let mut result = Vec::new();
    for stmt in stmts {
        let (pre, new_stmt) = flatten_stmt(stmt, counter);
        result.extend(pre);
        result.push(new_stmt);
    }
    result
}

/// Flatten a statement.  Returns a (possibly empty) list of pre-statements
/// needed to bind temporary variables, followed by the rewritten statement.
fn flatten_stmt(stmt: &sir::Stmt, counter: &mut usize) -> (Vec<sir::Stmt>, sir::Stmt) {
    match stmt {
        sir::Stmt::LocalVar(s) => {
            let (pre, init) = match &s.init {
                Some(e) => {
                    // The init expression itself can be complex — that's fine
                    // (it's the definition site, not an argument).
                    // But if the init is a call, flatten its args.
                    let (p, e2) = flatten_call_args_in_expr(e, counter);
                    (p, Some(e2))
                }
                None => (vec![], None),
            };
            (pre, sir::Stmt::LocalVar(sir::LocalVarStmt { init, ..s.clone() }))
        }
        sir::Stmt::Assign(s) => {
            let (pre_lhs, lhs) = flatten_call_args_in_expr(&s.lhs, counter);
            let (pre_rhs, rhs) = flatten_call_args_in_expr(&s.rhs, counter);
            let mut pre = pre_lhs;
            pre.extend(pre_rhs);
            (pre, sir::Stmt::Assign(sir::AssignStmt { lhs, rhs, ..s.clone() }))
        }
        sir::Stmt::AugAssign(s) => {
            let (pre_lhs, lhs) = flatten_call_args_in_expr(&s.lhs, counter);
            let (pre_rhs, rhs) = flatten_call_args_in_expr(&s.rhs, counter);
            let mut pre = pre_lhs;
            pre.extend(pre_rhs);
            (pre, sir::Stmt::AugAssign(sir::AugAssignStmt { lhs, rhs, ..s.clone() }))
        }
        sir::Stmt::Expr(s) => {
            let (pre, expr) = flatten_call_args_in_expr(&s.expr, counter);
            (pre, sir::Stmt::Expr(sir::ExprStmt { expr, ..s.clone() }))
        }
        sir::Stmt::If(s) => {
            let (pre_cond, cond) = flatten_call_args_in_expr(&s.cond, counter);
            let then_body = flatten_stmts(&s.then_body, counter);
            let else_body = s.else_body.as_ref().map(|b| flatten_stmts(b, counter));
            (pre_cond, sir::Stmt::If(sir::IfStmt { cond, then_body, else_body, ..s.clone() }))
        }
        sir::Stmt::While(s) => {
            let (pre_cond, cond) = flatten_call_args_in_expr(&s.cond, counter);
            let body = flatten_stmts(&s.body, counter);
            let invariant = s.invariant.as_ref().map(|e| {
                let (_, e2) = flatten_call_args_in_expr(e, counter);
                e2
            });
            (pre_cond, sir::Stmt::While(sir::WhileStmt { cond, body, invariant, ..s.clone() }))
        }
        sir::Stmt::For(s) => {
            let init = s.init.as_ref().map(|i| {
                let (pre, s2) = flatten_stmt(i, counter);
                if pre.is_empty() {
                    Box::new(s2)
                } else {
                    let mut stmts = pre;
                    stmts.push(s2);
                    Box::new(sir::Stmt::Block(stmts))
                }
            });
            let (pre_cond, cond) = match &s.cond {
                Some(e) => {
                    let (p, e2) = flatten_call_args_in_expr(e, counter);
                    (p, Some(e2))
                }
                None => (vec![], None),
            };
            let update = s.update.as_ref().map(|u| {
                let (pre, s2) = flatten_stmt(u, counter);
                if pre.is_empty() {
                    Box::new(s2)
                } else {
                    let mut stmts = pre;
                    stmts.push(s2);
                    Box::new(sir::Stmt::Block(stmts))
                }
            });
            let body = flatten_stmts(&s.body, counter);
            (pre_cond, sir::Stmt::For(sir::ForStmt { init, cond, update, body, ..s.clone() }))
        }
        sir::Stmt::Return(s) => {
            let (pre, value) = match &s.value {
                Some(e) => {
                    let (p, e2) = flatten_call_args_in_expr(e, counter);
                    (p, Some(e2))
                }
                None => (vec![], None),
            };
            (pre, sir::Stmt::Return(sir::ReturnStmt { value, ..s.clone() }))
        }
        sir::Stmt::Revert(s) => {
            let mut pre = vec![];
            let args = s
                .args
                .iter()
                .map(|e| {
                    let (p, e2) = flatten_call_args_in_expr(e, counter);
                    pre.extend(p);
                    e2
                })
                .collect();
            (pre, sir::Stmt::Revert(sir::RevertStmt { args, ..s.clone() }))
        }
        sir::Stmt::Assert(s) => {
            let (pre_cond, cond) = flatten_call_args_in_expr(&s.cond, counter);
            let (pre_msg, message) = match &s.message {
                Some(e) => {
                    let (p, e2) = flatten_call_args_in_expr(e, counter);
                    (p, Some(e2))
                }
                None => (vec![], None),
            };
            let mut pre = pre_cond;
            pre.extend(pre_msg);
            (pre, sir::Stmt::Assert(sir::AssertStmt { cond, message, ..s.clone() }))
        }
        sir::Stmt::Block(stmts) => (vec![], sir::Stmt::Block(flatten_stmts(stmts, counter))),
        sir::Stmt::Break | sir::Stmt::Continue | sir::Stmt::Dialect(_) => (vec![], stmt.clone()),
    }
}

/// Flatten call arguments within an expression (not the top-level call itself).
///
/// Returns `(pre_stmts, new_expr)`.
fn flatten_call_args_in_expr(
    expr: &sir::Expr,
    counter: &mut usize,
) -> (Vec<sir::Stmt>, sir::Expr) {
    match expr {
        sir::Expr::FunctionCall(call) => {
            let (pre_callee, callee) = flatten_call_args_in_expr(&call.callee, counter);
            let mut pre = pre_callee;
            let args = match &call.args {
                sir::CallArgs::Positional(args) => {
                    let lifted: Vec<sir::Expr> = args
                        .iter()
                        .map(|a| lift_to_atom(a, &mut pre, counter))
                        .collect();
                    sir::CallArgs::Positional(lifted)
                }
                sir::CallArgs::Named(named) => {
                    let lifted: Vec<sir::NamedArg> = named
                        .iter()
                        .map(|n| sir::NamedArg {
                            name: n.name.clone(),
                            value: lift_to_atom(&n.value, &mut pre, counter),
                        })
                        .collect();
                    sir::CallArgs::Named(lifted)
                }
            };
            let new_call = sir::Expr::FunctionCall(sir::CallExpr {
                callee: Box::new(callee),
                args,
                ty: call.ty.clone(),
                span: call.span,
            });
            (pre, new_call)
        }
        sir::Expr::BinOp(e) => {
            let (pre_lhs, lhs) = flatten_call_args_in_expr(&e.lhs, counter);
            let (pre_rhs, rhs) = flatten_call_args_in_expr(&e.rhs, counter);
            let mut pre = pre_lhs;
            pre.extend(pre_rhs);
            (
                pre,
                sir::Expr::BinOp(sir::BinOpExpr {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    ..e.clone()
                }),
            )
        }
        sir::Expr::UnOp(e) => {
            let (pre, operand) = flatten_call_args_in_expr(&e.operand, counter);
            (pre, sir::Expr::UnOp(sir::UnOpExpr { operand: Box::new(operand), ..e.clone() }))
        }
        sir::Expr::IndexAccess(e) => {
            let (pre_base, base) = flatten_call_args_in_expr(&e.base, counter);
            let (pre_idx, index) = match &e.index {
                Some(i) => {
                    let (p, i2) = flatten_call_args_in_expr(i, counter);
                    (p, Some(Box::new(i2)))
                }
                None => (vec![], None),
            };
            let mut pre = pre_base;
            pre.extend(pre_idx);
            (
                pre,
                sir::Expr::IndexAccess(sir::IndexAccessExpr {
                    base: Box::new(base),
                    index,
                    ..e.clone()
                }),
            )
        }
        sir::Expr::FieldAccess(e) => {
            let (pre, base) = flatten_call_args_in_expr(&e.base, counter);
            (
                pre,
                sir::Expr::FieldAccess(sir::FieldAccessExpr { base: Box::new(base), ..e.clone() }),
            )
        }
        sir::Expr::TypeCast(e) => {
            let (pre, inner) = flatten_call_args_in_expr(&e.expr, counter);
            (pre, sir::Expr::TypeCast(sir::TypeCastExpr { expr: Box::new(inner), ..e.clone() }))
        }
        sir::Expr::Ternary(e) => {
            let (pre_cond, cond) = flatten_call_args_in_expr(&e.cond, counter);
            let (pre_then, then_expr) = flatten_call_args_in_expr(&e.then_expr, counter);
            let (pre_else, else_expr) = flatten_call_args_in_expr(&e.else_expr, counter);
            let mut pre = pre_cond;
            pre.extend(pre_then);
            pre.extend(pre_else);
            (
                pre,
                sir::Expr::Ternary(sir::TernaryExpr {
                    cond: Box::new(cond),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                    ..e.clone()
                }),
            )
        }
        sir::Expr::Tuple(e) => {
            let mut pre = vec![];
            let elems = e
                .elems
                .iter()
                .map(|opt| {
                    opt.as_ref().map(|ex| {
                        let (p, e2) = flatten_call_args_in_expr(ex, counter);
                        pre.extend(p);
                        e2
                    })
                })
                .collect();
            (pre, sir::Expr::Tuple(sir::TupleExpr { elems, ..e.clone() }))
        }
        sir::Expr::Old(inner) => {
            let (pre, e2) = flatten_call_args_in_expr(inner, counter);
            (pre, sir::Expr::Old(Box::new(e2)))
        }
        sir::Expr::Forall { var, ty, body } => {
            let (pre, body2) = flatten_call_args_in_expr(body, counter);
            (pre, sir::Expr::Forall { var: var.clone(), ty: ty.clone(), body: Box::new(body2) })
        }
        sir::Expr::Exists { var, ty, body } => {
            let (pre, body2) = flatten_call_args_in_expr(body, counter);
            (pre, sir::Expr::Exists { var: var.clone(), ty: ty.clone(), body: Box::new(body2) })
        }
        // Atoms — no flattening needed.
        sir::Expr::Var(_) | sir::Expr::Lit(_) | sir::Expr::Result(_) | sir::Expr::Dialect(_) => {
            (vec![], expr.clone())
        }
    }
}

/// Flatten an expression and, if the result is still complex (not an atom),
/// introduce a `LocalVar` temp.  The temp is appended to `pre`.
fn lift_to_atom(expr: &sir::Expr, pre: &mut Vec<sir::Stmt>, counter: &mut usize) -> sir::Expr {
    let (p, e) = flatten_call_args_in_expr(expr, counter);
    pre.extend(p);

    if is_atom(&e) {
        e
    } else {
        *counter += 1;
        let tmp_name = format!("__tmp_{counter}");
        let ty = e.typ();
        let span = e.span();
        let decl = sir::LocalVarDecl { name: tmp_name.clone(), ty: ty.clone() };
        pre.push(sir::Stmt::LocalVar(sir::LocalVarStmt {
            vars: vec![Some(decl)],
            init: Some(e),
            span,
        }));
        sir::Expr::Var(sir::VarExpr::new(tmp_name, ty, span))
    }
}

fn is_atom(expr: &sir::Expr) -> bool {
    matches!(expr, sir::Expr::Var(_) | sir::Expr::Lit(_))
}

/// Flatten an expression recursively, hoisting any sub-calls that appear as
/// call arguments into temporaries.
fn flatten_expr(expr: &sir::Expr, counter: &mut usize) -> (Vec<sir::Stmt>, sir::Expr) {
    flatten_call_args_in_expr(expr, counter)
}
