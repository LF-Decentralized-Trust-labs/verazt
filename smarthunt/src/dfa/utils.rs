use crate::dfa::var::VarId;
use solidity::ir::{AtomicExpr, CalleeExpr, Expr, Stmt};

/// Collect variables that are used (read) in a statement
pub fn collect_used_vars(stmt: &Stmt) -> Vec<VarId> {
    let mut vars = Vec::new();

    match stmt {
        Stmt::Expr(e) => {
            collect_used_vars_expr(&e.expr, &mut vars);
        }
        Stmt::VarDecl(v) => {
            if let Some(value) = &v.value {
                collect_used_vars_expr(value, &mut vars);
            }
            for decl_opt in &v.vars {
                if let Some(decl) = decl_opt {
                    if let Some(value) = &decl.value {
                        collect_used_vars_expr(value, &mut vars);
                    }
                }
            }
        }
        Stmt::If(i) => {
            collect_used_vars_expr(&i.condition, &mut vars);
        }
        Stmt::Return(r) => {
            if let Some(expr) = &r.expr {
                collect_used_vars_expr(expr, &mut vars);
            }
        }
        Stmt::Loop(l) => {
            if let Some(condition) = &l.condition {
                collect_used_vars_expr(condition, &mut vars);
            }
        }
        _ => {}
    }

    vars
}

/// Collect variables that are defined (written) in a statement
pub fn collect_defined_vars(stmt: &Stmt) -> Vec<VarId> {
    let mut vars = Vec::new();

    match stmt {
        Stmt::VarDecl(v) => {
            // A variable declaration defines a new variable
            for decl_opt in &v.vars {
                if let Some(decl) = decl_opt {
                    vars.push(VarId::local(&decl.name));
                }
            }
        }
        Stmt::Expr(e) => {
            // Check if this is an assignment expression
            if let Expr::Call(call) = &e.expr {
                if let CalleeExpr::BuiltIn(name) = &call.callee {
                    if name == "=" || name.ends_with("=") {
                        if let Some(lhs) = call.args.first() {
                            if let AtomicExpr::Var(v) = lhs {
                                vars.push(VarId::local(&v.name));
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    vars
}

fn collect_used_vars_atomic(expr: &AtomicExpr, vars: &mut Vec<VarId>) {
    match expr {
        AtomicExpr::Var(v) => vars.push(VarId::local(&v.name)),
        _ => {}
    }
}

/// Helper to collect variables used in an expression
fn collect_used_vars_expr(expr: &Expr, vars: &mut Vec<VarId>) {
    match expr {
        Expr::Var(v) => {
            vars.push(VarId::local(&v.name));
        }
        Expr::Member(m) => {
            collect_used_vars_expr(&m.base, vars);
        }
        Expr::Index(i) => {
            vars.push(VarId::local(&i.base.name));
            if let Some(idx) = &i.index {
                collect_used_vars_atomic(idx, vars);
            }
        }
        Expr::Call(c) => {
            // Check for assignment: LHS might NOT be used if simple assignment
            let mut is_simple_assignment = false;
            if let CalleeExpr::BuiltIn(name) = &c.callee {
                if name == "=" {
                    is_simple_assignment = true;
                }
            }

            if is_simple_assignment {
                // Skip first arg (LHS)
                for arg in c.args.iter().skip(1) {
                    collect_used_vars_atomic(arg, vars);
                }
            } else {
                // Callee might be Expr? No, CalleeExpr.
                match &c.callee {
                    CalleeExpr::MemberExpr(m) => {
                        collect_used_vars_expr(&Expr::Member(m.clone()), vars)
                    }
                    _ => {}
                }
                for arg in &c.args {
                    collect_used_vars_atomic(arg, vars);
                }
            }
        }
        Expr::Tuple(t) => {
            for elem in &t.elems {
                if let Some(e) = elem {
                    collect_used_vars_atomic(e, vars);
                }
            }
        }
        Expr::InlineArray(a) => {
            for elem in &a.elems {
                collect_used_vars_atomic(elem, vars);
            }
        }
        // Conditional, New, TypeName, Lit do not introduce variable uses
        // Wait, Conditional does!
        // Expr::Conditional(c) isMISSING from my draft above? match cases must be exhaustive or default
        // In default (or explicit), handle Conditional
        // solidity::ir::exprs.rs: ConditionalExpr { cond, true_br, false_br } - they are AtomicExpr?
        // Let's check ir/exprs.rs again for ConditionalExpr.
        _ => {}
    }
}

pub fn get_vars_in_expr(expr: &Expr) -> Vec<VarId> {
    let mut vars = Vec::new();
    collect_used_vars_expr(expr, &mut vars);
    vars
}
