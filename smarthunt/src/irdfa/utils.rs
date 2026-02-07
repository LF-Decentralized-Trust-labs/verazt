use crate::irdfa::var::{VarId, VarScope};
use solidity::ir::{Expr, Stmt};

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
        }
        Stmt::If(i) => {
            collect_used_vars_expr(&i.condition, &mut vars);
        }
        Stmt::Return(r) => {
            if let Some(value) = &r.value {
                collect_used_vars_expr(value, &mut vars);
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
            vars.push(VarId::local(&v.name));
        }
        Stmt::Expr(e) => {
            // Check if this is an assignment expression
            if let Expr::Assign(assign) = &e.expr {
                collect_defined_vars_expr(&assign.lhs, &mut vars);
            }
        }
        _ => {}
    }

    vars
}

/// Helper to collect variables used in an expression
fn collect_used_vars_expr(expr: &Expr, vars: &mut Vec<VarId>) {
    match expr {
        Expr::Ident(id) => {
            // Identifier references a variable
            vars.push(VarId::local(&id.name));
        }
        Expr::Member(m) => {
            // For member access, the object is used
            collect_used_vars_expr(&m.object, vars);
        }
        Expr::Index(i) => {
            // Both array/mapping and index are used
            collect_used_vars_expr(&i.object, vars);
            collect_used_vars_expr(&i.index, vars);
        }
        Expr::Binary(b) => {
            collect_used_vars_expr(&b.left, vars);
            collect_used_vars_expr(&b.right, vars);
        }
        Expr::Unary(u) => {
            collect_used_vars_expr(&u.operand, vars);
        }
        Expr::Assign(a) => {
            // In assignment, RHS is used, LHS is defined
            // But for nested expressions, LHS might also be read (e.g., x += 1)
            collect_used_vars_expr(&a.rhs, vars);

            // For compound assignments, the LHS is also read
            if a.op != solidity::ir::AssignOp::Assign {
                collect_used_vars_expr(&a.lhs, vars);
            }
        }
        Expr::Call(c) => {
            collect_used_vars_expr(&c.callee, vars);
            for arg in c.args.args.iter() {
                collect_used_vars_expr(arg, vars);
            }
        }
        Expr::Conditional(c) => {
            collect_used_vars_expr(&c.condition, vars);
            collect_used_vars_expr(&c.true_expr, vars);
            collect_used_vars_expr(&c.false_expr, vars);
        }
        Expr::Tuple(t) => {
            for elem in &t.elems {
                collect_used_vars_expr(elem, vars);
            }
        }
        _ => {}
    }
}

/// Helper to collect variables defined in an expression (LHS of assignment)
fn collect_defined_vars_expr(expr: &Expr, vars: &mut Vec<VarId>) {
    match expr {
        Expr::Ident(id) => {
            vars.push(VarId::local(&id.name));
        }
        Expr::Member(m) => {
            // Member assignment might be to a state variable
            if let Expr::Ident(obj) = &*m.object {
                // Could be a state variable access
                vars.push(VarId::state(&m.member, &obj.name));
            }
        }
        Expr::Index(i) => {
            // Array/mapping element assignment
            collect_defined_vars_expr(&i.object, vars);
        }
        Expr::Tuple(t) => {
            // Tuple destructuring assignment
            for elem in &t.elems {
                collect_defined_vars_expr(elem, vars);
            }
        }
        _ => {}
    }
}
