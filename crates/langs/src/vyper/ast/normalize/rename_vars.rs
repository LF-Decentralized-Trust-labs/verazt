//! Rename local variables to ensure unique names and avoid shadowing.

use crate::vyper::ast::defs::*;
use crate::vyper::ast::exprs::*;
use crate::vyper::ast::source_unit::*;
use crate::vyper::ast::stmts::*;

/// Rename all local variables in a source unit with unique names.
pub fn rename_vars(su: &SourceUnit) -> SourceUnit {
    let mut renamer = VarRenamer::new(&su.path);
    renamer.rename_source_unit(su)
}

struct VarRenamer {
    prefix: String,
    counter: usize,
    scope: Vec<Vec<(String, String)>>,
}

impl VarRenamer {
    fn new(path: &str) -> Self {
        let prefix = std::path::Path::new(path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "vyper".to_string());
        VarRenamer { prefix, counter: 0, scope: vec![vec![]] }
    }

    fn fresh_name(&mut self, original: &str) -> String {
        self.counter += 1;
        format!("{}__{}__{}", self.prefix, original, self.counter)
    }

    fn push_scope(&mut self) {
        self.scope.push(vec![]);
    }

    fn pop_scope(&mut self) {
        self.scope.pop();
    }

    fn bind(&mut self, original: &str) -> String {
        let new_name = self.fresh_name(original);
        if let Some(scope) = self.scope.last_mut() {
            scope.push((original.to_string(), new_name.clone()));
        }
        new_name
    }

    fn lookup(&self, name: &str) -> String {
        for scope in self.scope.iter().rev() {
            for (orig, renamed) in scope.iter().rev() {
                if orig == name {
                    return renamed.clone();
                }
            }
        }
        name.to_string()
    }

    fn rename_source_unit(&mut self, su: &SourceUnit) -> SourceUnit {
        let body = su
            .body
            .iter()
            .map(|e| self.rename_source_unit_elem(e))
            .collect();
        SourceUnit { path: su.path.clone(), body, loc: su.loc.clone() }
    }

    fn rename_source_unit_elem(&mut self, elem: &SourceUnitElem) -> SourceUnitElem {
        match elem {
            SourceUnitElem::Func(f) => SourceUnitElem::Func(self.rename_func_def(f)),
            other => other.clone(),
        }
    }

    fn rename_func_def(&mut self, f: &FuncDef) -> FuncDef {
        self.push_scope();

        let params: Vec<Param> = f
            .params
            .iter()
            .map(|p| {
                let new_name = self.bind(&p.name);
                Param {
                    name: new_name,
                    typ: p.typ.clone(),
                    default: p.default.as_ref().map(|e| self.rename_expr(e)),
                    loc: p.loc.clone(),
                }
            })
            .collect();

        let body = self.rename_stmts(&f.body);

        self.pop_scope();

        FuncDef {
            name: f.name.clone(),
            params,
            return_type: f.return_type.clone(),
            decorators: f.decorators.clone(),
            doc_string: f.doc_string.clone(),
            body,
            loc: f.loc.clone(),
        }
    }

    fn rename_stmts(&mut self, stmts: &[Stmt]) -> Vec<Stmt> {
        stmts.iter().map(|s| self.rename_stmt(s)).collect()
    }

    fn rename_stmt(&mut self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::AnnAssign(s) => {
                let value = s.value.as_ref().map(|v| self.rename_expr(v));
                let target = match &s.target {
                    Expr::Ident(id) => {
                        let new_name = self.bind(&id.name);
                        Expr::Ident(Identifier { name: new_name, loc: id.loc.clone() })
                    }
                    other => self.rename_expr(other),
                };
                Stmt::AnnAssign(AnnAssignStmt {
                    target,
                    annotation: s.annotation.clone(),
                    value,
                    loc: s.loc.clone(),
                })
            }
            Stmt::Assign(s) => Stmt::Assign(AssignStmt {
                target: self.rename_expr(&s.target),
                value: self.rename_expr(&s.value),
                loc: s.loc.clone(),
            }),
            Stmt::AugAssign(s) => Stmt::AugAssign(AugAssignStmt {
                target: self.rename_expr(&s.target),
                op: s.op,
                value: self.rename_expr(&s.value),
                loc: s.loc.clone(),
            }),
            Stmt::If(s) => Stmt::If(IfStmt {
                cond: self.rename_expr(&s.cond),
                then_body: self.rename_stmts(&s.then_body),
                else_body: self.rename_stmts(&s.else_body),
                loc: s.loc.clone(),
            }),
            Stmt::For(s) => {
                self.push_scope();
                let target = match &s.target {
                    Expr::Ident(id) => {
                        let new_name = self.bind(&id.name);
                        Expr::Ident(Identifier { name: new_name, loc: id.loc.clone() })
                    }
                    other => self.rename_expr(other),
                };
                let body = self.rename_stmts(&s.body);
                self.pop_scope();
                Stmt::For(ForStmt { target, iter: s.iter.clone(), body, loc: s.loc.clone() })
            }
            Stmt::Return(s) => Stmt::Return(ReturnStmt {
                value: s.value.as_ref().map(|v| self.rename_expr(v)),
                loc: s.loc.clone(),
            }),
            Stmt::Assert(s) => Stmt::Assert(AssertStmt {
                test: self.rename_expr(&s.test),
                msg: s.msg.as_ref().map(|m| self.rename_expr(m)),
                loc: s.loc.clone(),
            }),
            Stmt::Log(s) => {
                Stmt::Log(LogStmt { event: self.rename_expr(&s.event), loc: s.loc.clone() })
            }
            Stmt::Expr(s) => {
                Stmt::Expr(ExprStmt { value: self.rename_expr(&s.value), loc: s.loc.clone() })
            }
            Stmt::Raise(s) => Stmt::Raise(RaiseStmt {
                exc: s.exc.as_ref().map(|e| self.rename_expr(e)),
                loc: s.loc.clone(),
            }),
            other => other.clone(),
        }
    }

    fn rename_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::Ident(id) => {
                Expr::Ident(Identifier { name: self.lookup(&id.name), loc: id.loc.clone() })
            }
            Expr::Attribute(e) => Expr::Attribute(AttributeExpr {
                value: Box::new(self.rename_expr(&e.value)),
                attr: e.attr.clone(),
                loc: e.loc.clone(),
            }),
            Expr::Subscript(e) => Expr::Subscript(SubscriptExpr {
                value: Box::new(self.rename_expr(&e.value)),
                index: Box::new(self.rename_expr(&e.index)),
                loc: e.loc.clone(),
            }),
            Expr::Call(e) => Expr::Call(CallExpr {
                func: Box::new(self.rename_expr(&e.func)),
                args: e.args.iter().map(|a| self.rename_expr(a)).collect(),
                keywords: e.keywords.clone(),
                loc: e.loc.clone(),
            }),
            Expr::BinOp(e) => Expr::BinOp(BinOpExpr {
                left: Box::new(self.rename_expr(&e.left)),
                op: e.op,
                right: Box::new(self.rename_expr(&e.right)),
                loc: e.loc.clone(),
            }),
            Expr::BoolOp(e) => Expr::BoolOp(BoolOpExpr {
                op: e.op,
                values: e.values.iter().map(|v| self.rename_expr(v)).collect(),
                loc: e.loc.clone(),
            }),
            Expr::Compare(e) => Expr::Compare(CompareExpr {
                left: Box::new(self.rename_expr(&e.left)),
                ops: e.ops.clone(),
                comparators: e.comparators.iter().map(|c| self.rename_expr(c)).collect(),
                loc: e.loc.clone(),
            }),
            Expr::UnaryOp(e) => Expr::UnaryOp(UnaryOpExpr {
                op: e.op,
                operand: Box::new(self.rename_expr(&e.operand)),
                loc: e.loc.clone(),
            }),
            Expr::Tuple(elems, loc) => {
                Expr::Tuple(elems.iter().map(|e| self.rename_expr(e)).collect(), loc.clone())
            }
            Expr::IfExp(e) => Expr::IfExp(IfExpExpr {
                test: Box::new(self.rename_expr(&e.test)),
                body: Box::new(self.rename_expr(&e.body)),
                orelse: Box::new(self.rename_expr(&e.orelse)),
                loc: e.loc.clone(),
            }),
            other => other.clone(),
        }
    }
}
