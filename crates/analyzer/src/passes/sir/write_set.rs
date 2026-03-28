//! Write-Set Analysis
//!
//! For each function in each contract, computes the set of storage variable
//! names that the function *may write*.  Interprocedural extension: callee
//! write sets are unioned into the caller (conservative, fixed-point).

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::sir::ContractDecl;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{CallExpr, Decl, Expr, MemberDecl, Stmt};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for the write-set analysis result.
///
/// Maps `(contract_name, function_name)` → set of storage variable names
/// that the function may write (directly or transitively).
pub struct WriteSetArtifact;

impl ContextKey for WriteSetArtifact {
    type Value = HashMap<(String, String), HashSet<String>>;
    const NAME: &'static str = "write_set";
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Write-set analysis pass.
pub struct WriteSetPass;

impl Pass for WriteSetPass {
    fn name(&self) -> &'static str {
        "write-set"
    }

    fn description(&self) -> &'static str {
        "Compute per-function storage write sets"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl AnalysisPass for WriteSetPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut result: HashMap<(String, String), HashSet<String>> = HashMap::new();

        if let Some(modules) = &ctx.ir_units {
            for module in modules {
                for decl in &module.decls {
                    if let Decl::Contract(contract) = decl {
                        let storage_vars = contract.storage_names();
                        if storage_vars.is_empty() {
                            continue;
                        }

                        // Phase 1: direct write sets per function
                        let mut direct: HashMap<String, HashSet<String>> = HashMap::new();
                        // Collect internal call targets per function
                        let mut calls: HashMap<String, HashSet<String>> = HashMap::new();

                        for member in &contract.members {
                            if let MemberDecl::Function(func) = member {
                                let mut writes = HashSet::new();
                                let mut callees = HashSet::new();
                                if let Some(body) = &func.body {
                                    collect_writes(body, &storage_vars, &mut writes);
                                    collect_internal_calls(body, &mut callees);
                                }
                                direct.insert(func.name.clone(), writes);
                                calls.insert(func.name.clone(), callees);
                            }
                        }

                        // Phase 2: fixed-point interprocedural union
                        let mut changed = true;
                        let mut iteration = 0;
                        const MAX_ITER: usize = 50;
                        while changed && iteration < MAX_ITER {
                            changed = false;
                            iteration += 1;
                            let func_names: Vec<String> = direct.keys().cloned().collect();
                            for fname in &func_names {
                                if let Some(callees) = calls.get(fname).cloned() {
                                    for callee in &callees {
                                        if let Some(callee_writes) = direct.get(callee).cloned() {
                                            let entry = direct.entry(fname.clone()).or_default();
                                            for w in callee_writes {
                                                if entry.insert(w) {
                                                    changed = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Store results keyed by (contract, function)
                        for (fname, writes) in direct {
                            result.insert((contract.name.clone(), fname), writes);
                        }
                    }
                }
            }
        }

        ctx.store::<WriteSetArtifact>(result);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

/// Collect storage names written by assignment statements (direct writes).
fn collect_writes(stmts: &[Stmt], storage_vars: &[String], out: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Assign(a) => {
                collect_written_storage(&a.lhs, storage_vars, out);
            }
            Stmt::AugAssign(a) => {
                collect_written_storage(&a.lhs, storage_vars, out);
            }
            Stmt::If(s) => {
                collect_writes(&s.then_body, storage_vars, out);
                if let Some(else_body) = &s.else_body {
                    collect_writes(else_body, storage_vars, out);
                }
            }
            Stmt::While(s) => collect_writes(&s.body, storage_vars, out),
            Stmt::For(s) => {
                if let Some(init) = &s.init {
                    collect_writes(&[*init.clone()], storage_vars, out);
                }
                if let Some(update) = &s.update {
                    collect_writes(&[*update.clone()], storage_vars, out);
                }
                collect_writes(&s.body, storage_vars, out);
            }
            Stmt::Block(stmts) => collect_writes(stmts, storage_vars, out),
            _ => {}
        }
    }
}

/// If a LHS expression references a storage variable, add its name to the set.
fn collect_written_storage(expr: &Expr, storage_vars: &[String], out: &mut HashSet<String>) {
    if ContractDecl::expr_references_storage(expr, storage_vars) {
        if let Some(name) = extract_storage_name(expr) {
            out.insert(name);
        }
    }
}

/// Extract the base storage variable name from an expression.
fn extract_storage_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Var(v) => Some(v.name.clone()),
        Expr::IndexAccess(ia) => extract_storage_name(&ia.base),
        Expr::FieldAccess(fa) => extract_storage_name(&fa.base),
        _ => None,
    }
}

/// Collect names of internal (same-contract) function calls from statement
/// body.
fn collect_internal_calls(stmts: &[Stmt], out: &mut HashSet<String>) {
    struct CallCollector<'b> {
        out: &'b mut HashSet<String>,
    }
    impl<'a, 'b> Visit<'a> for CallCollector<'b> {
        fn visit_call_expr(&mut self, call: &'a CallExpr) {
            if let Expr::Var(v) = &*call.callee {
                self.out.insert(v.name.clone());
            }
            visit::default::visit_call_expr(self, call);
        }
    }
    let mut collector = CallCollector { out };
    collector.visit_stmts(stmts);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::sir::*;

    fn make_module_with_writes() -> scirs::sir::Module {
        // Contract "C" with storage var "x" and two functions:
        // - `write_x` which assigns to x
        // - `indirect` which calls write_x
        let storage_x =
            MemberDecl::Storage(StorageDecl::new("x".to_string(), Type::Si256, None, None));

        let write_x_body = vec![Stmt::Assign(AssignStmt {
            lhs: Expr::Var(VarExpr::new("x".to_string(), Type::Si256, None)),
            rhs: Expr::Lit(Lit::one(None)),
            span: None,
        })];

        let write_x_fn = MemberDecl::Function(FunctionDecl::new(
            "write_x".to_string(),
            vec![],
            vec![],
            Some(write_x_body),
            None,
        ));

        let indirect_body = vec![Stmt::Expr(ExprStmt {
            expr: Expr::FunctionCall(CallExpr {
                callee: Box::new(Expr::Var(VarExpr::new("write_x".to_string(), Type::None, None))),
                args: CallArgs::Positional(vec![]),
                ty: Type::None,
                span: None,
            }),
            span: None,
        })];

        let indirect_fn = MemberDecl::Function(FunctionDecl::new(
            "indirect".to_string(),
            vec![],
            vec![],
            Some(indirect_body),
            None,
        ));

        let no_write_body = vec![Stmt::Return(ReturnStmt {
            value: Some(Expr::Lit(Lit::one(None))),
            span: None,
        })];

        let no_write_fn = MemberDecl::Function(FunctionDecl::new(
            "no_write".to_string(),
            vec![],
            vec![],
            Some(no_write_body),
            None,
        ));

        let contract = ContractDecl {
            name: "C".to_string(),
            parents: vec![],
            attrs: vec![],
            members: vec![storage_x, write_x_fn, indirect_fn, no_write_fn],
            span: None,
        };

        scirs::sir::Module {
            id: "test".to_string(),
            attrs: vec![],
            decls: vec![Decl::Contract(contract)],
        }
    }

    #[test]
    fn test_write_set_basic() {
        let module = make_module_with_writes();
        let mut ctx = AnalysisContext::new(vec![module], AnalysisConfig::default());
        let pass = WriteSetPass;
        pass.run(&mut ctx).unwrap();

        let ws = ctx.get::<WriteSetArtifact>().unwrap();

        // write_x directly writes to x
        let writes = ws.get(&("C".to_string(), "write_x".to_string())).unwrap();
        assert!(writes.contains("x"));

        // indirect transitively writes to x (calls write_x)
        let writes = ws.get(&("C".to_string(), "indirect".to_string())).unwrap();
        assert!(writes.contains("x"));

        // no_write has empty write set
        let writes = ws.get(&("C".to_string(), "no_write".to_string())).unwrap();
        assert!(writes.is_empty());
    }
}
