//! Flatten complex sub-expressions into temporary local variables.
//!
//! This pass produces a 3-address-like form by hoisting nested
//! sub-expressions in function calls, binary ops, etc. into
//! temporary local variables.

use crate::vyper::ast::defs::*;
use crate::vyper::ast::exprs::*;
use crate::vyper::ast::source_unit::*;
use crate::vyper::ast::stmts::*;
use crate::vyper::ast::types::Type;

/// Flatten expressions in the source unit.
pub fn flatten_expr(su: &SourceUnit) -> SourceUnit {
    let body = su.body.iter().map(|e| flatten_elem(e)).collect();
    SourceUnit { path: su.path.clone(), body, loc: su.loc.clone() }
}

fn flatten_elem(elem: &SourceUnitElem) -> SourceUnitElem {
    match elem {
        SourceUnitElem::Func(f) => SourceUnitElem::Func(flatten_func(f)),
        other => other.clone(),
    }
}

fn flatten_func(f: &FuncDef) -> FuncDef {
    let mut flattener = ExprFlattener::new();
    let body = flattener.flatten_stmts(&f.body);
    FuncDef {
        name: f.name.clone(),
        params: f.params.clone(),
        return_type: f.return_type.clone(),
        decorators: f.decorators.clone(),
        doc_string: f.doc_string.clone(),
        body,
        loc: f.loc.clone(),
    }
}

struct ExprFlattener {
    counter: usize,
}

impl ExprFlattener {
    fn new() -> Self {
        ExprFlattener { counter: 0 }
    }

    fn fresh_tmp(&mut self) -> String {
        self.counter += 1;
        format!("__tmp__{}", self.counter)
    }

    fn flatten_stmts(&mut self, stmts: &[Stmt]) -> Vec<Stmt> {
        let mut result = Vec::new();
        for stmt in stmts {
            self.flatten_stmt(stmt, &mut result);
        }
        result
    }

    fn flatten_stmt(&mut self, stmt: &Stmt, out: &mut Vec<Stmt>) {
        match stmt {
            Stmt::Assign(s) => {
                let (value, mut pre) = self.flatten_expr_to_simple(&s.value);
                out.append(&mut pre);
                out.push(Stmt::Assign(AssignStmt {
                    target: s.target.clone(),
                    value,
                    loc: s.loc.clone(),
                }));
            }
            Stmt::AugAssign(s) => {
                let (value, mut pre) = self.flatten_expr_to_simple(&s.value);
                out.append(&mut pre);
                out.push(Stmt::AugAssign(AugAssignStmt {
                    target: s.target.clone(),
                    op: s.op,
                    value,
                    loc: s.loc.clone(),
                }));
            }
            Stmt::AnnAssign(s) => {
                let value = s.value.as_ref().map(|v| {
                    let (val, mut pre) = self.flatten_expr_to_simple(v);
                    out.append(&mut pre);
                    val
                });
                out.push(Stmt::AnnAssign(AnnAssignStmt {
                    target: s.target.clone(),
                    annotation: s.annotation.clone(),
                    value,
                    loc: s.loc.clone(),
                }));
            }
            Stmt::If(s) => {
                let (cond, mut pre) = self.flatten_expr_to_simple(&s.cond);
                out.append(&mut pre);
                out.push(Stmt::If(IfStmt {
                    cond,
                    then_body: self.flatten_stmts(&s.then_body),
                    else_body: self.flatten_stmts(&s.else_body),
                    loc: s.loc.clone(),
                }));
            }
            Stmt::For(s) => {
                out.push(Stmt::For(ForStmt {
                    target: s.target.clone(),
                    iter: s.iter.clone(),
                    body: self.flatten_stmts(&s.body),
                    loc: s.loc.clone(),
                }));
            }
            Stmt::Return(s) => {
                let value = s.value.as_ref().map(|v| {
                    let (val, mut pre) = self.flatten_expr_to_simple(v);
                    out.append(&mut pre);
                    val
                });
                out.push(Stmt::Return(ReturnStmt { value, loc: s.loc.clone() }));
            }
            Stmt::Assert(s) => {
                let (test, mut pre) = self.flatten_expr_to_simple(&s.test);
                out.append(&mut pre);
                out.push(Stmt::Assert(AssertStmt {
                    test,
                    msg: s.msg.clone(),
                    loc: s.loc.clone(),
                }));
            }
            Stmt::Log(s) => {
                out.push(Stmt::Log(s.clone()));
            }
            Stmt::Expr(s) => {
                let (value, mut pre) = self.flatten_expr_to_simple(&s.value);
                out.append(&mut pre);
                out.push(Stmt::Expr(ExprStmt { value, loc: s.loc.clone() }));
            }
            other => out.push(other.clone()),
        }
    }

    /// Flatten an expression, returning the simplified expression and
    /// any prerequisite statements that need to be emitted first.
    fn flatten_expr_to_simple(&mut self, expr: &Expr) -> (Expr, Vec<Stmt>) {
        match expr {
            Expr::BinOp(e) => {
                let (left, mut pre_l) = self.flatten_expr_to_simple(&e.left);
                let (right, mut pre_r) = self.flatten_expr_to_simple(&e.right);
                let mut pre = Vec::new();
                pre.append(&mut pre_l);
                pre.append(&mut pre_r);
                (
                    Expr::BinOp(BinOpExpr {
                        left: Box::new(left),
                        op: e.op,
                        right: Box::new(right),
                        loc: e.loc.clone(),
                    }),
                    pre,
                )
            }
            Expr::Call(e) => {
                let mut pre = Vec::new();
                let mut new_args = Vec::new();
                for arg in &e.args {
                    let (a, mut p) = self.flatten_expr_to_simple(arg);
                    pre.append(&mut p);
                    // Hoist complex args into temps
                    if is_complex(&a) {
                        let tmp_name = self.fresh_tmp();
                        pre.push(Stmt::AnnAssign(AnnAssignStmt {
                            target: Expr::Ident(Identifier { name: tmp_name.clone(), loc: None }),
                            annotation: Type::UInt(crate::vyper::ast::types::UIntType {
                                bits: 256,
                            }),
                            value: Some(a),
                            loc: None,
                        }));
                        new_args.push(Expr::Ident(Identifier { name: tmp_name, loc: None }));
                    } else {
                        new_args.push(a);
                    }
                }
                (
                    Expr::Call(CallExpr {
                        func: e.func.clone(),
                        args: new_args,
                        keywords: e.keywords.clone(),
                        loc: e.loc.clone(),
                    }),
                    pre,
                )
            }
            _ => (expr.clone(), vec![]),
        }
    }
}

fn is_complex(expr: &Expr) -> bool {
    matches!(expr, Expr::BinOp(_) | Expr::Call(_) | Expr::BoolOp(_))
}
