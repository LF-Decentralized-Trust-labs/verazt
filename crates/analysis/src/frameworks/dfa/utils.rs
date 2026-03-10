use crate::frameworks::dfa::var::VarId;
use mlir::sir::{Expr, Stmt};

/// Collect variables that are used (read) in a statement
pub fn collect_used_vars(stmt: &Stmt) -> Vec<VarId> {
    let mut vars = Vec::new();

    match stmt {
        Stmt::Expr(e) => {
            collect_used_vars_expr(&e.expr, &mut vars);
        }
        Stmt::LocalVar(v) => {
            if let Some(init) = &v.init {
                collect_used_vars_expr(init, &mut vars);
            }
        }
        Stmt::Assign(a) => {
            // RHS is always used
            collect_used_vars_expr(&a.rhs, &mut vars);
            // LHS may involve used variables (e.g. a[i] = v  uses i)
            collect_lhs_used_vars(&a.lhs, &mut vars);
        }
        Stmt::AugAssign(a) => {
            // Both sides are used (x += y  reads x and y)
            collect_used_vars_expr(&a.lhs, &mut vars);
            collect_used_vars_expr(&a.rhs, &mut vars);
        }
        Stmt::If(i) => {
            collect_used_vars_expr(&i.cond, &mut vars);
        }
        Stmt::Return(r) => {
            if let Some(value) = &r.value {
                collect_used_vars_expr(value, &mut vars);
            }
        }
        Stmt::While(w) => {
            collect_used_vars_expr(&w.cond, &mut vars);
        }
        Stmt::For(f) => {
            if let Some(cond) = &f.cond {
                collect_used_vars_expr(cond, &mut vars);
            }
        }
        Stmt::Revert(r) => {
            for arg in &r.args {
                collect_used_vars_expr(arg, &mut vars);
            }
        }
        Stmt::Assert(a) => {
            collect_used_vars_expr(&a.cond, &mut vars);
        }
        _ => {}
    }

    vars
}

/// Collect variables that are defined (written) in a statement
pub fn collect_defined_vars(stmt: &Stmt) -> Vec<VarId> {
    let mut vars = Vec::new();

    match stmt {
        Stmt::LocalVar(v) => {
            for decl_opt in &v.vars {
                if let Some(decl) = decl_opt {
                    vars.push(VarId::local(&decl.name));
                }
            }
        }
        Stmt::Assign(a) => {
            if let Expr::Var(v) = &a.lhs {
                vars.push(VarId::local(&v.name));
            }
        }
        Stmt::AugAssign(a) => {
            if let Expr::Var(v) = &a.lhs {
                vars.push(VarId::local(&v.name));
            }
        }
        _ => {}
    }

    vars
}

/// Collect used variables from the LHS of an assignment.
/// For a simple variable, we don't count the variable itself as "used"
/// (it's being defined). But for complex LHS like `a[i]` or `a.f`,
/// any index expressions or base objects are "used".
fn collect_lhs_used_vars(lhs: &Expr, vars: &mut Vec<VarId>) {
    match lhs {
        Expr::Var(_) => {
            // Simple variable assignment: LHS variable is defined, not used
        }
        Expr::IndexAccess(i) => {
            // a[idx] = ...: the base and index are used
            collect_used_vars_expr(&i.base, vars);
            if let Some(idx) = &i.index {
                collect_used_vars_expr(idx, vars);
            }
        }
        Expr::FieldAccess(f) => {
            // a.field = ...: the base is used
            collect_used_vars_expr(&f.base, vars);
        }
        _ => {
            collect_used_vars_expr(lhs, vars);
        }
    }
}

/// Helper to collect variables used in an expression
fn collect_used_vars_expr(expr: &Expr, vars: &mut Vec<VarId>) {
    match expr {
        Expr::Var(v) => {
            vars.push(VarId::local(&v.name));
        }
        Expr::Lit(_) => {}
        Expr::BinOp(b) => {
            collect_used_vars_expr(&b.lhs, vars);
            collect_used_vars_expr(&b.rhs, vars);
        }
        Expr::UnOp(u) => {
            collect_used_vars_expr(&u.operand, vars);
        }
        Expr::FunctionCall(c) => {
            collect_used_vars_expr(&c.callee, vars);
            for arg in &c.args {
                collect_used_vars_expr(arg, vars);
            }
        }
        Expr::FieldAccess(f) => {
            collect_used_vars_expr(&f.base, vars);
        }
        Expr::IndexAccess(i) => {
            collect_used_vars_expr(&i.base, vars);
            if let Some(idx) = &i.index {
                collect_used_vars_expr(idx, vars);
            }
        }
        Expr::Tuple(t) => {
            for elem in &t.elems {
                if let Some(e) = elem {
                    collect_used_vars_expr(e, vars);
                }
            }
        }
        Expr::Ternary(t) => {
            collect_used_vars_expr(&t.cond, vars);
            collect_used_vars_expr(&t.then_expr, vars);
            collect_used_vars_expr(&t.else_expr, vars);
        }
        Expr::TypeCast(tc) => {
            collect_used_vars_expr(&tc.expr, vars);
        }
        Expr::Old(inner) => {
            collect_used_vars_expr(inner, vars);
        }
        Expr::Forall { body, .. } | Expr::Exists { body, .. } => {
            collect_used_vars_expr(body, vars);
        }
        Expr::Result(_) | Expr::Dialect(_) => {}
    }
}

pub fn get_vars_in_expr(expr: &Expr) -> Vec<VarId> {
    let mut vars = Vec::new();
    collect_used_vars_expr(expr, &mut vars);
    vars
}
