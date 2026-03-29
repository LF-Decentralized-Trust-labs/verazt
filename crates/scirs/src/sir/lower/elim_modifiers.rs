//! Eliminate modifiers — inline modifier bodies into function bodies.
//!
//! For each function that has non-empty `modifier_invocs`, this pass:
//! 1. Looks up the modifier definition in the contract's `ModifierDef` members.
//! 2. Substitutes modifier parameters with the call-site arguments.
//! 3. Replaces `EvmStmt::Placeholder` (`_;`) with the accumulated function
//!    body.
//! 4. Repeats for each modifier in order (outermost modifier applied last).
//! 5. Removes all `ModifierDef` member declarations from contracts.
//! 6. Clears `modifier_invocs` from all functions.

use crate::sir;
use crate::sir::dialect::evm::{EvmMemberDecl, EvmStmt};
use crate::sir::lower::CirLowerError;
use std::collections::HashMap;

/// Inline all modifier invocations.
pub fn run(module: &sir::Module) -> Result<sir::Module, CirLowerError> {
    let decls = module
        .decls
        .iter()
        .map(|d| process_decl(d))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sir::Module { id: module.id.clone(), attrs: module.attrs.clone(), decls })
}

fn process_decl(decl: &sir::Decl) -> Result<sir::Decl, CirLowerError> {
    match decl {
        sir::Decl::Contract(c) => Ok(sir::Decl::Contract(process_contract(c)?)),
        sir::Decl::Dialect(d) => Ok(sir::Decl::Dialect(d.clone())),
    }
}

fn process_contract(contract: &sir::ContractDecl) -> Result<sir::ContractDecl, CirLowerError> {
    // Collect all modifier definitions in this contract.
    let modifier_map = collect_modifiers(contract);

    // Process members: inline modifiers where needed, remove ModifierDef.
    let members = contract
        .members
        .iter()
        .filter_map(|m| process_member(m, &modifier_map).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sir::ContractDecl { members, ..contract.clone() })
}

/// Collect ModifierDef declarations keyed by name.
fn collect_modifiers(
    contract: &sir::ContractDecl,
) -> HashMap<String, (Vec<(String, sir::Type)>, Vec<sir::Stmt>)> {
    let mut map = HashMap::new();
    for member in &contract.members {
        if let sir::MemberDecl::Dialect(sir::DialectMemberDecl::Evm(EvmMemberDecl::ModifierDef(
            m,
        ))) = member
        {
            map.insert(m.name.clone(), (m.params.clone(), m.body.clone()));
        }
    }
    map
}

/// Process one member: inline modifiers for functions, drop ModifierDef.
/// Returns `None` to drop the member, `Some(Ok(...))` to keep/transform it.
fn process_member(
    member: &sir::MemberDecl,
    modifier_map: &HashMap<String, (Vec<(String, sir::Type)>, Vec<sir::Stmt>)>,
) -> Result<Option<sir::MemberDecl>, CirLowerError> {
    match member {
        sir::MemberDecl::Dialect(sir::DialectMemberDecl::Evm(EvmMemberDecl::ModifierDef(_))) => {
            // Drop modifier definitions — they're now inlined.
            Ok(None)
        }
        sir::MemberDecl::Function(f) => {
            Ok(Some(sir::MemberDecl::Function(inline_modifiers(f, modifier_map)?)))
        }
        _ => Ok(Some(member.clone())),
    }
}

/// Inline all modifier invocations for a single function.
fn inline_modifiers(
    func: &sir::FunctionDecl,
    modifier_map: &HashMap<String, (Vec<(String, sir::Type)>, Vec<sir::Stmt>)>,
) -> Result<sir::FunctionDecl, CirLowerError> {
    if func.modifier_invocs.is_empty() {
        return Ok(func.clone());
    }

    // Start with the original function body.
    let mut current_body: Vec<sir::Stmt> = func.body.clone().unwrap_or_default();

    // Apply modifiers in reverse order so the first modifier is the outermost
    // wrapper.
    for invoc in func.modifier_invocs.iter().rev() {
        let Some((params, modifier_body)) = modifier_map.get(&invoc.name) else {
            // Unknown modifier — skip (it may be inherited; handled by
            // resolve_inheritance).
            continue;
        };

        // Build substitution map: param_name → arg_expr.
        let subst: HashMap<String, sir::Expr> = params
            .iter()
            .zip(invoc.args.iter())
            .map(|((name, _ty), arg)| (name.clone(), arg.clone()))
            .collect();

        // Replace placeholder with current_body; substitute params.
        current_body = inline_modifier_body(modifier_body, &current_body, &subst);
    }

    Ok(sir::FunctionDecl { body: Some(current_body), modifier_invocs: vec![], ..func.clone() })
}

/// Expand one modifier body by:
/// - Replacing `Dialect(Evm(Placeholder))` with `func_body`.
/// - Substituting parameter references with argument values.
fn inline_modifier_body(
    modifier_body: &[sir::Stmt],
    func_body: &[sir::Stmt],
    subst: &HashMap<String, sir::Expr>,
) -> Vec<sir::Stmt> {
    let mut result = Vec::new();
    for stmt in modifier_body {
        if is_placeholder(stmt) {
            result.extend(func_body.iter().cloned());
        } else {
            result.push(subst_stmt(stmt, subst));
        }
    }
    result
}

fn is_placeholder(stmt: &sir::Stmt) -> bool {
    matches!(stmt, sir::Stmt::Dialect(sir::DialectStmt::Evm(EvmStmt::Placeholder(_))))
}

// ─── Substitution of parameter variables ─────────────────────────────────────

fn subst_stmt(stmt: &sir::Stmt, subst: &HashMap<String, sir::Expr>) -> sir::Stmt {
    match stmt {
        sir::Stmt::LocalVar(s) => sir::Stmt::LocalVar(sir::LocalVarStmt {
            init: s.init.as_ref().map(|e| subst_expr(e, subst)),
            ..s.clone()
        }),
        sir::Stmt::Assign(s) => sir::Stmt::Assign(sir::AssignStmt {
            lhs: subst_expr(&s.lhs, subst),
            rhs: subst_expr(&s.rhs, subst),
            ..s.clone()
        }),
        sir::Stmt::AugAssign(s) => sir::Stmt::AugAssign(sir::AugAssignStmt {
            lhs: subst_expr(&s.lhs, subst),
            rhs: subst_expr(&s.rhs, subst),
            ..s.clone()
        }),
        sir::Stmt::Expr(s) => {
            sir::Stmt::Expr(sir::ExprStmt { expr: subst_expr(&s.expr, subst), ..s.clone() })
        }
        sir::Stmt::If(s) => sir::Stmt::If(sir::IfStmt {
            cond: subst_expr(&s.cond, subst),
            then_body: s.then_body.iter().map(|st| subst_stmt(st, subst)).collect(),
            else_body: s
                .else_body
                .as_ref()
                .map(|b| b.iter().map(|st| subst_stmt(st, subst)).collect()),
            ..s.clone()
        }),
        sir::Stmt::While(s) => sir::Stmt::While(sir::WhileStmt {
            cond: subst_expr(&s.cond, subst),
            body: s.body.iter().map(|st| subst_stmt(st, subst)).collect(),
            invariant: s.invariant.as_ref().map(|e| subst_expr(e, subst)),
            ..s.clone()
        }),
        sir::Stmt::For(s) => sir::Stmt::For(sir::ForStmt {
            init: s.init.as_ref().map(|i| Box::new(subst_stmt(i, subst))),
            cond: s.cond.as_ref().map(|e| subst_expr(e, subst)),
            update: s.update.as_ref().map(|u| Box::new(subst_stmt(u, subst))),
            body: s.body.iter().map(|st| subst_stmt(st, subst)).collect(),
            invariant: s.invariant.as_ref().map(|e| subst_expr(e, subst)),
            ..s.clone()
        }),
        sir::Stmt::Return(s) => sir::Stmt::Return(sir::ReturnStmt {
            value: s.value.as_ref().map(|e| subst_expr(e, subst)),
            ..s.clone()
        }),
        sir::Stmt::Revert(s) => sir::Stmt::Revert(sir::RevertStmt {
            args: s.args.iter().map(|e| subst_expr(e, subst)).collect(),
            ..s.clone()
        }),
        sir::Stmt::Assert(s) => sir::Stmt::Assert(sir::AssertStmt {
            cond: subst_expr(&s.cond, subst),
            message: s.message.as_ref().map(|e| subst_expr(e, subst)),
            ..s.clone()
        }),
        sir::Stmt::Block(stmts) => {
            sir::Stmt::Block(stmts.iter().map(|st| subst_stmt(st, subst)).collect())
        }
        sir::Stmt::Break | sir::Stmt::Continue | sir::Stmt::Dialect(_) => stmt.clone(),
    }
}

fn subst_expr(expr: &sir::Expr, subst: &HashMap<String, sir::Expr>) -> sir::Expr {
    match expr {
        sir::Expr::Var(v) => {
            if let Some(replacement) = subst.get(&v.name) {
                replacement.clone()
            } else {
                expr.clone()
            }
        }
        sir::Expr::FunctionCall(e) => {
            let callee = subst_expr(&e.callee, subst);
            let args = match &e.args {
                sir::CallArgs::Positional(args) => {
                    sir::CallArgs::Positional(args.iter().map(|a| subst_expr(a, subst)).collect())
                }
                sir::CallArgs::Named(named) => sir::CallArgs::Named(
                    named
                        .iter()
                        .map(|n| sir::NamedArg {
                            name: n.name.clone(),
                            value: subst_expr(&n.value, subst),
                        })
                        .collect(),
                ),
            };
            sir::Expr::FunctionCall(sir::CallExpr {
                callee: Box::new(callee),
                args,
                ty: e.ty.clone(),
                span: e.span.clone(),
            })
        }
        sir::Expr::BinOp(e) => sir::Expr::BinOp(sir::BinOpExpr {
            lhs: Box::new(subst_expr(&e.lhs, subst)),
            rhs: Box::new(subst_expr(&e.rhs, subst)),
            ..e.clone()
        }),
        sir::Expr::UnOp(e) => sir::Expr::UnOp(sir::UnOpExpr {
            operand: Box::new(subst_expr(&e.operand, subst)),
            ..e.clone()
        }),
        sir::Expr::IndexAccess(e) => sir::Expr::IndexAccess(sir::IndexAccessExpr {
            base: Box::new(subst_expr(&e.base, subst)),
            index: e.index.as_ref().map(|i| Box::new(subst_expr(i, subst))),
            ..e.clone()
        }),
        sir::Expr::FieldAccess(e) => sir::Expr::FieldAccess(sir::FieldAccessExpr {
            base: Box::new(subst_expr(&e.base, subst)),
            ..e.clone()
        }),
        sir::Expr::TypeCast(e) => sir::Expr::TypeCast(sir::TypeCastExpr {
            expr: Box::new(subst_expr(&e.expr, subst)),
            ..e.clone()
        }),
        sir::Expr::Ternary(e) => sir::Expr::Ternary(sir::TernaryExpr {
            cond: Box::new(subst_expr(&e.cond, subst)),
            then_expr: Box::new(subst_expr(&e.then_expr, subst)),
            else_expr: Box::new(subst_expr(&e.else_expr, subst)),
            ..e.clone()
        }),
        sir::Expr::Tuple(e) => sir::Expr::Tuple(sir::TupleExpr {
            elems: e
                .elems
                .iter()
                .map(|opt| opt.as_ref().map(|ex| subst_expr(ex, subst)))
                .collect(),
            ..e.clone()
        }),
        sir::Expr::Old(inner) => sir::Expr::Old(Box::new(subst_expr(inner, subst))),
        sir::Expr::Forall { var, ty, body } => sir::Expr::Forall {
            var: var.clone(),
            ty: ty.clone(),
            body: Box::new(subst_expr(body, subst)),
        },
        sir::Expr::Exists { var, ty, body } => sir::Expr::Exists {
            var: var.clone(),
            ty: ty.clone(),
            body: Box::new(subst_expr(body, subst)),
        },
        sir::Expr::Lit(_) | sir::Expr::Result(_) | sir::Expr::Dialect(_) => expr.clone(),
    }
}
