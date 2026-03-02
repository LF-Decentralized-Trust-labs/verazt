//! Call Graph Pass
//!
//! This pass builds a call graph for function call relationships.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, Pass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::ast::symbol_table::FunctionId;
use solidity::ast::{
    Block, CallArgs, ContractDef, ContractElem, Expr, FuncDef, FuncKind, Name, SourceUnit,
    SourceUnitElem, Stmt,
};
use std::collections::{HashMap, HashSet};

/// Call site information.
#[derive(Debug, Clone)]
pub struct CallSite {
    /// The calling function.
    pub caller: FunctionId,
    /// The called function (if resolvable).
    pub callee: Option<FunctionId>,
    /// Location in source.
    pub loc: Option<solidity::ast::Loc>,
    /// Whether this is an external call.
    pub is_external: bool,
    /// Whether this is a delegate call.
    pub is_delegatecall: bool,
}

/// Call graph for function call relationships.
#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    /// Edges: function -> set of functions it calls.
    pub callees: HashMap<FunctionId, HashSet<FunctionId>>,

    /// Reverse edges: function -> set of functions that call it.
    pub callers: HashMap<FunctionId, HashSet<FunctionId>>,

    /// All call sites.
    pub call_sites: Vec<CallSite>,

    /// External calls.
    pub external_calls: HashMap<FunctionId, Vec<CallSite>>,

    /// Delegate calls.
    pub delegate_calls: HashMap<FunctionId, Vec<CallSite>>,
}

impl CallGraph {
    /// Create a new empty call graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a call graph from source units.
    pub fn from_source_units(source_units: &[SourceUnit]) -> Self {
        let mut graph = Self::new();
        let mut builder = CallGraphBuilder::new(&mut graph);

        for source_unit in source_units {
            builder.visit_source_unit(source_unit);
        }

        graph
    }

    /// Add a call edge.
    pub fn add_call(&mut self, caller: FunctionId, callee: FunctionId) {
        self.callees
            .entry(caller.clone())
            .or_default()
            .insert(callee.clone());
        self.callers.entry(callee).or_default().insert(caller);
    }

    /// Get callees of a function.
    pub fn get_callees(&self, func: &FunctionId) -> HashSet<FunctionId> {
        self.callees.get(func).cloned().unwrap_or_default()
    }

    /// Get callers of a function.
    pub fn get_callers(&self, func: &FunctionId) -> HashSet<FunctionId> {
        self.callers.get(func).cloned().unwrap_or_default()
    }

    /// Check if a function calls another.
    pub fn calls(&self, caller: &FunctionId, callee: &FunctionId) -> bool {
        self.callees
            .get(caller)
            .map(|c| c.contains(callee))
            .unwrap_or(false)
    }

    /// Get all functions reachable from a given function.
    pub fn reachable_from(&self, func: &FunctionId) -> HashSet<FunctionId> {
        let mut reachable = HashSet::new();
        let mut worklist = vec![func.clone()];

        while let Some(current) = worklist.pop() {
            if reachable.insert(current.clone()) {
                if let Some(callees) = self.callees.get(&current) {
                    worklist.extend(callees.iter().cloned());
                }
            }
        }

        reachable
    }

    /// Check if a function makes external calls.
    pub fn has_external_calls(&self, func: &FunctionId) -> bool {
        self.external_calls
            .get(func)
            .map(|c| !c.is_empty())
            .unwrap_or(false)
    }

    /// Check if a function makes delegate calls.
    pub fn has_delegate_calls(&self, func: &FunctionId) -> bool {
        self.delegate_calls
            .get(func)
            .map(|c| !c.is_empty())
            .unwrap_or(false)
    }

    /// Get external calls made by a function.
    pub fn get_external_calls(&self, func: &FunctionId) -> Vec<&CallSite> {
        self.external_calls
            .get(func)
            .map(|c| c.iter().collect())
            .unwrap_or_default()
    }

    /// Get the number of functions in the graph.
    pub fn function_count(&self) -> usize {
        let mut funcs = HashSet::new();
        funcs.extend(self.callees.keys().cloned());
        funcs.extend(self.callers.keys().cloned());
        funcs.len()
    }

    /// Get the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.callees.values().map(|c| c.len()).sum()
    }
}

/// Builder for constructing call graph.
struct CallGraphBuilder<'a> {
    graph: &'a mut CallGraph,
    current_contract: Option<Name>,
    current_function: Option<FunctionId>,
}

impl<'a> CallGraphBuilder<'a> {
    fn new(graph: &'a mut CallGraph) -> Self {
        Self { graph, current_contract: None, current_function: None }
    }

    fn visit_source_unit(&mut self, source_unit: &SourceUnit) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.current_contract = Some(contract.name.clone());
                    self.visit_contract(contract);
                    self.current_contract = None;
                }
                SourceUnitElem::Func(func) => {
                    let func_id = FunctionId::from_func(func, None);
                    self.current_function = Some(func_id);
                    self.visit_func(func);
                    self.current_function = None;
                }
                _ => {}
            }
        }
    }

    fn visit_contract(&mut self, contract: &ContractDef) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                let func_id = FunctionId::from_func(func, Some(contract));
                self.current_function = Some(func_id);
                self.visit_func(func);
                self.current_function = None;
            }
        }
    }

    fn visit_func(&mut self, func: &FuncDef) {
        // Visit function body for call expressions
        if let Some(body) = &func.body {
            self.visit_block(body);
        }
    }

    fn visit_block(&mut self, block: &Block) {
        for stmt in &block.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => self.visit_expr(&e.expr),
            Stmt::Block(b) => {
                self.visit_block(b);
            }
            Stmt::If(i) => {
                self.visit_expr(&i.condition);
                self.visit_stmt(&i.true_branch);
                if let Some(ref else_stmt) = i.false_branch {
                    self.visit_stmt(else_stmt);
                }
            }
            Stmt::While(w) => {
                self.visit_expr(&w.condition);
                self.visit_stmt(&w.body);
            }
            Stmt::DoWhile(d) => {
                self.visit_expr(&d.condition);
                self.visit_stmt(&d.body);
            }
            Stmt::For(f) => {
                if let Some(ref pre) = f.pre_loop {
                    self.visit_stmt(pre);
                }
                if let Some(ref cond) = f.condition {
                    self.visit_expr(cond);
                }
                if let Some(ref post) = f.post_loop {
                    self.visit_stmt(post);
                }
                self.visit_stmt(&f.body);
            }
            Stmt::Return(r) => {
                if let Some(ref expr) = r.expr {
                    self.visit_expr(expr);
                }
            }
            Stmt::VarDecl(v) => {
                if let Some(ref value) = v.value {
                    self.visit_expr(value);
                }
            }
            Stmt::Emit(e) => {
                self.visit_expr(&e.event);
                self.visit_call_args(&e.args);
            }
            Stmt::Revert(r) => {
                if let Some(ref error) = r.error {
                    self.visit_expr(error);
                }
                self.visit_call_args(&r.args);
            }
            Stmt::Try(t) => {
                self.visit_expr(&t.guarded_expr);
                self.visit_block(&t.body);
                for catch in &t.catch_clauses {
                    self.visit_block(&catch.body);
                }
            }
            _ => {}
        }
    }

    fn visit_call_args(&mut self, args: &CallArgs) {
        match args {
            CallArgs::Unnamed(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            CallArgs::Named(named_args) => {
                for arg in named_args {
                    self.visit_expr(&arg.value);
                }
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Call(call) => {
                // Record call
                if let Some(ref caller) = self.current_function {
                    // Try to resolve callee
                    if let Some(callee_id) = self.resolve_callee(&call.callee) {
                        self.graph.add_call(caller.clone(), callee_id.clone());
                    }

                    // Check for external/delegate calls
                    let is_external = self.is_external_call(&call.callee);
                    let is_delegatecall = self.is_delegatecall(&call.callee);

                    let call_site = CallSite {
                        caller: caller.clone(),
                        callee: self.resolve_callee(&call.callee),
                        loc: call.loc,
                        is_external,
                        is_delegatecall,
                    };

                    self.graph.call_sites.push(call_site.clone());

                    if is_external {
                        self.graph
                            .external_calls
                            .entry(caller.clone())
                            .or_default()
                            .push(call_site.clone());
                    }

                    if is_delegatecall {
                        self.graph
                            .delegate_calls
                            .entry(caller.clone())
                            .or_default()
                            .push(call_site);
                    }
                }

                // Visit callee expression
                self.visit_expr(&call.callee);
                // Visit call arguments
                self.visit_call_args(&call.args);
            }
            Expr::Binary(b) => {
                self.visit_expr(&b.left);
                self.visit_expr(&b.right);
            }
            Expr::Unary(u) => {
                self.visit_expr(&u.body);
            }
            Expr::Member(m) => {
                self.visit_expr(&m.base);
            }
            Expr::Index(i) => {
                self.visit_expr(&i.base_expr);
                if let Some(ref idx) = i.index {
                    self.visit_expr(idx);
                }
            }
            Expr::Tuple(t) => {
                for e in t.elems.iter().flatten() {
                    self.visit_expr(e);
                }
            }
            Expr::Conditional(c) => {
                self.visit_expr(&c.cond);
                self.visit_expr(&c.true_br);
                self.visit_expr(&c.false_br);
            }
            Expr::Assign(a) => {
                self.visit_expr(&a.left);
                self.visit_expr(&a.right);
            }
            _ => {}
        }
    }

    fn resolve_callee(&self, callee: &Expr) -> Option<FunctionId> {
        match callee {
            Expr::Ident(ident) => {
                // Simple function call within same contract
                Some(FunctionId {
                    contract: self.current_contract.clone(),
                    name: ident.name.clone(),
                    kind: FuncKind::ContractFunc,
                })
            }
            Expr::Member(member) => {
                // Could be contract.function or object.method
                if let Expr::Ident(base) = member.base.as_ref() {
                    Some(FunctionId {
                        contract: Some(base.name.clone()),
                        name: member.member.clone(),
                        kind: FuncKind::ContractFunc,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn is_external_call(&self, callee: &Expr) -> bool {
        // Check for .call(), .send(), .transfer() on addresses
        if let Expr::Member(member) = callee {
            let member_name = member.member.base.as_str();
            return matches!(member_name, "call" | "send" | "transfer" | "staticcall");
        }
        false
    }

    fn is_delegatecall(&self, callee: &Expr) -> bool {
        if let Expr::Member(member) = callee {
            return member.member.base.as_str() == "delegatecall";
        }
        false
    }
}

/// Pass for building the call graph.
pub struct CallGraphPass;

impl CallGraphPass {
    /// Create a new call graph pass.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CallGraphPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for CallGraphPass {
    fn id(&self) -> PassId {
        PassId::CallGraph
    }

    fn name(&self) -> &'static str {
        "Call Graph"
    }

    fn description(&self) -> &'static str {
        "Builds a call graph representing function call relationships"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }
}

impl AnalysisPass for CallGraphPass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let call_graph = CallGraph::from_source_units(&context.source_units);
        context.store_artifact("call_graph", call_graph);
        context.record_ast_traversal();
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.has_artifact("call_graph")
    }
}

/// Convenience trait to get call graph from context.
pub trait CallGraphExt {
    fn call_graph(&self) -> Option<&CallGraph>;
}

impl CallGraphExt for AnalysisContext {
    fn call_graph(&self) -> Option<&CallGraph> {
        self.get_artifact::<CallGraph>("call_graph")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_graph_pass() {
        let pass = CallGraphPass::new();
        assert_eq!(pass.id(), PassId::CallGraph);
        assert_eq!(pass.dependencies(), vec![PassId::SymbolTable]);
    }
}
