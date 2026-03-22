//! Inter-Procedural Call Graph from SIR Call Sites
//!
//! Builds a call graph by structurally scanning SIR `Module` declarations
//! for `Expr::FunctionCall` nodes. The result is a `petgraph::DiGraph`
//! that can be queried for callers, callees, SCCs, etc.
//!
//! This complements the BIR-level `scirs::bir::call_graph::CallGraph` (which
//! is produced during SIR→BIR lowering) by providing a SIR-native view
//! usable before BIR is available or for SIR-only analyses.

use petgraph::graph::{DiGraph, NodeIndex};
use scirs::sir::defs::{FunctionDecl, MemberDecl};
use scirs::sir::exprs::Expr;
use scirs::sir::module::{Decl, Module};
use scirs::sir::stmts::Stmt;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// SirCallGraph
// ═══════════════════════════════════════════════════════════════════

/// A call graph built from structural scan of SIR call-site expressions.
///
/// Nodes are fully-qualified function names (`"Contract.function"`);
/// edges represent static call relationships extracted from
/// `Expr::FunctionCall`.
#[derive(Debug, Clone)]
pub struct SirCallGraph {
    /// The underlying directed graph.
    pub graph: DiGraph<String, ()>,
    /// Name → node index lookup.
    name_to_node: HashMap<String, NodeIndex>,
}

impl SirCallGraph {
    /// Build a call graph from an SIR module.
    ///
    /// Walks every contract and free function, collects call-site edges.
    pub fn build(module: &Module) -> Self {
        let mut cg = SirCallGraph { graph: DiGraph::new(), name_to_node: HashMap::new() };

        // Phase 1: Register all declared functions as nodes.
        for decl in &module.decls {
            match decl {
                Decl::Contract(contract) => {
                    for member in &contract.members {
                        if let MemberDecl::Function(f) = member {
                            let name = qualified_name(&contract.name, &f.name);
                            cg.get_or_insert_node(&name);
                        }
                    }
                }
                Decl::Dialect(_) => {}
            }
        }

        // Phase 2: Walk function bodies to collect call edges.
        for decl in &module.decls {
            match decl {
                Decl::Contract(contract) => {
                    for member in &contract.members {
                        if let MemberDecl::Function(f) = member {
                            let caller = qualified_name(&contract.name, &f.name);
                            let callees = collect_callees(f, &contract.name);
                            for callee in callees {
                                let caller_node = cg.get_or_insert_node(&caller);
                                let callee_node = cg.get_or_insert_node(&callee);
                                cg.graph.add_edge(caller_node, callee_node, ());
                            }
                        }
                    }
                }
                Decl::Dialect(_) => {}
            }
        }

        cg
    }

    /// Look up the node index for a function name.
    pub fn node_index(&self, name: &str) -> Option<NodeIndex> {
        self.name_to_node.get(name).copied()
    }

    /// Get all callee names for a function.
    pub fn callees_of(&self, name: &str) -> Vec<&str> {
        let Some(&node) = self.name_to_node.get(name) else {
            return Vec::new();
        };
        self.graph
            .neighbors(node)
            .map(|n| self.graph[n].as_str())
            .collect()
    }

    /// Get all caller names for a function.
    pub fn callers_of(&self, name: &str) -> Vec<&str> {
        let Some(&node) = self.name_to_node.get(name) else {
            return Vec::new();
        };
        self.graph
            .neighbors_directed(node, petgraph::Direction::Incoming)
            .map(|n| self.graph[n].as_str())
            .collect()
    }

    /// Number of functions (nodes) in the call graph.
    pub fn function_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Number of call edges.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Iterate over all function names.
    pub fn functions(&self) -> impl Iterator<Item = &str> {
        self.name_to_node.keys().map(|s| s.as_str())
    }

    fn get_or_insert_node(&mut self, name: &str) -> NodeIndex {
        if let Some(&idx) = self.name_to_node.get(name) {
            idx
        } else {
            let idx = self.graph.add_node(name.to_string());
            self.name_to_node.insert(name.to_string(), idx);
            idx
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// SIR tree walking
// ═══════════════════════════════════════════════════════════════════

fn qualified_name(contract: &str, function: &str) -> String {
    format!("{contract}.{function}")
}

/// Collect all callee names found in a function's body.
fn collect_callees(func: &FunctionDecl, contract_name: &str) -> Vec<String> {
    let mut callees = Vec::new();
    if let Some(body) = &func.body {
        for stmt in body {
            walk_stmt_for_calls(stmt, contract_name, &mut callees);
        }
    }
    callees
}

fn walk_stmt_for_calls(stmt: &Stmt, contract_name: &str, callees: &mut Vec<String>) {
    match stmt {
        Stmt::Expr(e) => walk_expr_for_calls(&e.expr, contract_name, callees),
        Stmt::LocalVar(lv) => {
            if let Some(init) = &lv.init {
                walk_expr_for_calls(init, contract_name, callees);
            }
        }
        Stmt::Assign(a) => {
            walk_expr_for_calls(&a.lhs, contract_name, callees);
            walk_expr_for_calls(&a.rhs, contract_name, callees);
        }
        Stmt::AugAssign(a) => {
            walk_expr_for_calls(&a.lhs, contract_name, callees);
            walk_expr_for_calls(&a.rhs, contract_name, callees);
        }
        Stmt::If(i) => {
            walk_expr_for_calls(&i.cond, contract_name, callees);
            for s in &i.then_body {
                walk_stmt_for_calls(s, contract_name, callees);
            }
            if let Some(else_body) = &i.else_body {
                for s in else_body {
                    walk_stmt_for_calls(s, contract_name, callees);
                }
            }
        }
        Stmt::While(w) => {
            walk_expr_for_calls(&w.cond, contract_name, callees);
            for s in &w.body {
                walk_stmt_for_calls(s, contract_name, callees);
            }
        }
        Stmt::For(f) => {
            if let Some(init) = &f.init {
                walk_stmt_for_calls(init, contract_name, callees);
            }
            if let Some(cond) = &f.cond {
                walk_expr_for_calls(cond, contract_name, callees);
            }
            if let Some(update) = &f.update {
                walk_stmt_for_calls(update, contract_name, callees);
            }
            for s in &f.body {
                walk_stmt_for_calls(s, contract_name, callees);
            }
        }
        Stmt::Return(r) => {
            if let Some(expr) = &r.value {
                walk_expr_for_calls(expr, contract_name, callees);
            }
        }
        Stmt::Revert(r) => {
            for arg in &r.args {
                walk_expr_for_calls(arg, contract_name, callees);
            }
        }
        Stmt::Assert(a) => {
            walk_expr_for_calls(&a.cond, contract_name, callees);
            if let Some(msg) = &a.message {
                walk_expr_for_calls(msg, contract_name, callees);
            }
        }
        Stmt::Block(stmts) => {
            for s in stmts {
                walk_stmt_for_calls(s, contract_name, callees);
            }
        }
        Stmt::Break | Stmt::Continue => {}
        Stmt::Dialect(_) => {}
    }
}

fn walk_expr_for_calls(expr: &Expr, contract_name: &str, callees: &mut Vec<String>) {
    match expr {
        Expr::FunctionCall(call) => {
            let callee_name = resolve_callee_name(&call.callee, contract_name);
            callees.push(callee_name);
            // Also walk arguments — they may contain nested calls.
            for arg in &call.args {
                walk_expr_for_calls(arg, contract_name, callees);
            }
        }
        Expr::BinOp(b) => {
            walk_expr_for_calls(&b.lhs, contract_name, callees);
            walk_expr_for_calls(&b.rhs, contract_name, callees);
        }
        Expr::UnOp(u) => {
            walk_expr_for_calls(&u.operand, contract_name, callees);
        }
        Expr::IndexAccess(i) => {
            walk_expr_for_calls(&i.base, contract_name, callees);
            if let Some(idx) = &i.index {
                walk_expr_for_calls(idx, contract_name, callees);
            }
        }
        Expr::FieldAccess(f) => {
            walk_expr_for_calls(&f.base, contract_name, callees);
        }
        Expr::TypeCast(t) => {
            walk_expr_for_calls(&t.expr, contract_name, callees);
        }
        Expr::Ternary(t) => {
            walk_expr_for_calls(&t.cond, contract_name, callees);
            walk_expr_for_calls(&t.then_expr, contract_name, callees);
            walk_expr_for_calls(&t.else_expr, contract_name, callees);
        }
        Expr::Tuple(t) => {
            for e in t.elems.iter().flatten() {
                walk_expr_for_calls(e, contract_name, callees);
            }
        }
        Expr::Old(inner) => walk_expr_for_calls(inner, contract_name, callees),
        Expr::Forall { body, .. } | Expr::Exists { body, .. } => {
            walk_expr_for_calls(body, contract_name, callees);
        }
        Expr::Var(_) | Expr::Lit(_) | Expr::Result(_) | Expr::Dialect(_) => {}
    }
}

/// Resolve a call-site expression to a callee name.
///
/// For `Expr::Var("foo")` → `"Contract.foo"` (same-contract call).
/// For `Expr::FieldAccess(base, "bar")` → `"<base>.bar"` (cross-contract).
fn resolve_callee_name(callee: &Expr, contract_name: &str) -> String {
    match callee {
        Expr::Var(v) => qualified_name(contract_name, &v.name),
        Expr::FieldAccess(f) => {
            let base = expr_name(&f.base);
            format!("{base}.{}", f.field)
        }
        _ => format!("{contract_name}.<unknown>"),
    }
}

/// Best-effort name extraction from an expression.
fn expr_name(expr: &Expr) -> String {
    match expr {
        Expr::Var(v) => v.name.clone(),
        Expr::FieldAccess(f) => {
            let base = expr_name(&f.base);
            format!("{base}.{}", f.field)
        }
        _ => "<expr>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scirs::sir::defs::*;
    use scirs::sir::exprs::*;
    use scirs::sir::stmts::*;
    use scirs::sir::types::Type;

    fn make_call_expr(name: &str) -> Expr {
        Expr::FunctionCall(CallExpr {
            callee: Box::new(Expr::Var(VarExpr {
                name: name.to_string(),
                ty: Type::None,
                span: None,
            })),
            args: vec![],
            ty: Type::None,
            span: None,
        })
    }

    fn make_function(name: &str, body: Vec<Stmt>) -> FunctionDecl {
        FunctionDecl {
            name: name.to_string(),
            type_params: vec![],
            params: vec![],
            returns: vec![],
            attrs: vec![],
            spec: None,
            body: Some(body),
            span: None,
        }
    }

    #[test]
    fn test_simple_call_graph() {
        // Contract Foo { fn a() { b(); } fn b() {} }
        let func_a = make_function(
            "a",
            vec![Stmt::Expr(ExprStmt {
                expr: make_call_expr("b"),
                span: None,
            })],
        );
        let func_b = make_function("b", vec![]);

        let module = Module {
            id: "test".into(),
            attrs: vec![],
            decls: vec![Decl::Contract(ContractDecl {
                name: "Foo".into(),
                parents: vec![],
                attrs: vec![],
                members: vec![MemberDecl::Function(func_a), MemberDecl::Function(func_b)],
                span: None,
            })],
        };

        let cg = SirCallGraph::build(&module);

        assert_eq!(cg.function_count(), 2);
        assert_eq!(cg.edge_count(), 1);
        assert_eq!(cg.callees_of("Foo.a"), vec!["Foo.b"]);
        assert!(cg.callees_of("Foo.b").is_empty());
        assert_eq!(cg.callers_of("Foo.b"), vec!["Foo.a"]);
    }

    #[test]
    fn test_no_calls() {
        let func = make_function("f", vec![]);
        let module = Module {
            id: "test".into(),
            attrs: vec![],
            decls: vec![Decl::Contract(ContractDecl {
                name: "C".into(),
                parents: vec![],
                attrs: vec![],
                members: vec![MemberDecl::Function(func)],
                span: None,
            })],
        };

        let cg = SirCallGraph::build(&module);
        assert_eq!(cg.function_count(), 1);
        assert_eq!(cg.edge_count(), 0);
    }

    #[test]
    fn test_recursive_call() {
        let func = make_function(
            "rec",
            vec![Stmt::Expr(ExprStmt {
                expr: make_call_expr("rec"),
                span: None,
            })],
        );
        let module = Module {
            id: "test".into(),
            attrs: vec![],
            decls: vec![Decl::Contract(ContractDecl {
                name: "C".into(),
                parents: vec![],
                attrs: vec![],
                members: vec![MemberDecl::Function(func)],
                span: None,
            })],
        };

        let cg = SirCallGraph::build(&module);
        assert_eq!(cg.function_count(), 1);
        assert_eq!(cg.edge_count(), 1);
        assert_eq!(cg.callees_of("C.rec"), vec!["C.rec"]);
    }

    #[test]
    fn test_empty_module() {
        let module = Module { id: "empty".into(), attrs: vec![], decls: vec![] };

        let cg = SirCallGraph::build(&module);
        assert_eq!(cg.function_count(), 0);
        assert_eq!(cg.edge_count(), 0);
    }
}
