//! Call graph structures.
//!
//! This module provides data structures for representing call graphs.

use solidity::ast::{
    CallExpr, ContractDef, Expr, FuncDef, Loc, Name, SourceUnit, SourceUnitElem,
    Stmt, ContractElem, Block, CallKind,
};
use std::collections::{HashMap, HashSet};
use crate::graph::symbol_table::FunctionId;

/// An edge in the call graph.
#[derive(Debug, Clone)]
pub struct CallEdge {
    /// Caller function
    pub caller: FunctionId,
    
    /// Callee function
    pub callee: FunctionId,
    
    /// Location of the call
    pub loc: Option<Loc>,
    
    /// Is this an external call?
    pub is_external: bool,
    
    /// Is this a delegate call?
    pub is_delegate_call: bool,
    
    /// Is this a static call?
    pub is_static_call: bool,
}

/// Call site information.
#[derive(Debug, Clone)]
pub struct CallSite {
    /// Function containing the call
    pub caller: FunctionId,
    
    /// Call expression
    pub call_expr: CallExpr,
    
    /// Location
    pub loc: Option<Loc>,
    
    /// Is this an external call?
    pub is_external: bool,
    
    /// Resolved callee (if known)
    pub resolved_callee: Option<FunctionId>,
}

/// Call graph for the analyzed contracts.
#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    /// Edges indexed by caller
    pub edges_by_caller: HashMap<FunctionId, Vec<CallEdge>>,
    
    /// Edges indexed by callee
    pub edges_by_callee: HashMap<FunctionId, Vec<CallEdge>>,
    
    /// All call sites
    pub call_sites: Vec<CallSite>,
    
    /// External call sites
    pub external_call_sites: Vec<CallSite>,
    
    /// Recursive functions
    pub recursive_functions: HashSet<FunctionId>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an edge to the call graph.
    pub fn add_edge(&mut self, edge: CallEdge) {
        self.edges_by_caller
            .entry(edge.caller.clone())
            .or_default()
            .push(edge.clone());
        self.edges_by_callee
            .entry(edge.callee.clone())
            .or_default()
            .push(edge);
    }

    /// Add a call site.
    pub fn add_call_site(&mut self, site: CallSite) {
        if site.is_external {
            self.external_call_sites.push(site.clone());
        }
        self.call_sites.push(site);
    }

    /// Get all callees of a function.
    pub fn get_callees(&self, caller: &FunctionId) -> Vec<&FunctionId> {
        self.edges_by_caller
            .get(caller)
            .map(|edges| edges.iter().map(|e| &e.callee).collect())
            .unwrap_or_default()
    }

    /// Get all callers of a function.
    pub fn get_callers(&self, callee: &FunctionId) -> Vec<&FunctionId> {
        self.edges_by_callee
            .get(callee)
            .map(|edges| edges.iter().map(|e| &e.caller).collect())
            .unwrap_or_default()
    }

    /// Check if a function is recursive.
    pub fn is_recursive(&self, func: &FunctionId) -> bool {
        self.recursive_functions.contains(func)
    }

    /// Get all external call sites in a function.
    pub fn get_external_calls(&self, func: &FunctionId) -> Vec<&CallSite> {
        self.external_call_sites
            .iter()
            .filter(|site| &site.caller == func)
            .collect()
    }

    /// Compute transitive callees (all functions reachable from a function).
    pub fn transitive_callees(&self, func: &FunctionId) -> HashSet<FunctionId> {
        let mut visited = HashSet::new();
        self.collect_callees(func, &mut visited);
        visited
    }

    fn collect_callees(&self, func: &FunctionId, visited: &mut HashSet<FunctionId>) {
        if visited.contains(func) {
            return;
        }
        visited.insert(func.clone());
        
        for callee in self.get_callees(func) {
            self.collect_callees(callee, visited);
        }
    }

    /// Detect recursive functions.
    pub fn detect_recursion(&mut self) {
        let funcs: Vec<FunctionId> = self.edges_by_caller.keys().cloned().collect();
        
        for func in funcs {
            if self.can_reach(&func, &func, &mut HashSet::new()) {
                self.recursive_functions.insert(func);
            }
        }
    }

    fn can_reach(&self, from: &FunctionId, target: &FunctionId, visited: &mut HashSet<FunctionId>) -> bool {
        if visited.contains(from) {
            return false;
        }
        visited.insert(from.clone());
        
        for callee in self.get_callees(from) {
            if callee == target {
                return true;
            }
            if self.can_reach(callee, target, visited) {
                return true;
            }
        }
        false
    }
}

/// Build a call graph from source units.
pub struct CallGraphBuilder<'a> {
    source_units: &'a [SourceUnit],
    call_graph: CallGraph,
    current_contract: Option<Name>,
    current_function: Option<FunctionId>,
}

impl<'a> CallGraphBuilder<'a> {
    pub fn new(source_units: &'a [SourceUnit]) -> Self {
        Self {
            source_units,
            call_graph: CallGraph::new(),
            current_contract: None,
            current_function: None,
        }
    }

    pub fn build(mut self) -> CallGraph {
        for source_unit in self.source_units {
            self.visit_source_unit(source_unit);
        }
        self.call_graph.detect_recursion();
        self.call_graph
    }

    fn visit_source_unit(&mut self, source_unit: &SourceUnit) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.visit_contract(contract);
                }
                SourceUnitElem::Func(func) => {
                    self.current_contract = None;
                    self.visit_function(func);
                }
                _ => {}
            }
        }
    }

    fn visit_contract(&mut self, contract: &ContractDef) {
        self.current_contract = Some(contract.name.clone());
        
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.visit_function(func);
            }
        }
    }

    fn visit_function(&mut self, func: &FuncDef) {
        let func_id = FunctionId::from_func(func, 
            self.current_contract.as_ref().map(|n| {
                // We don't have the full ContractDef here, so create a minimal reference
                // This is a simplification - in practice, we'd look up the contract
                ContractDef::new(
                    None, None, n.clone(),
                    solidity::ast::ContractKind::Contract,
                    false, vec![], vec![], None,
                )
            }).as_ref());
        
        self.current_function = Some(func_id);
        
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
            Stmt::Block(block) => self.visit_block(block),
            Stmt::If(if_stmt) => {
                self.visit_expr(&if_stmt.condition);
                self.visit_stmt(&if_stmt.true_branch);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.visit_stmt(false_br);
                }
            }
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.visit_stmt(pre);
                }
                if let Some(cond) = &for_stmt.condition {
                    self.visit_expr(cond);
                }
                if let Some(post) = &for_stmt.post_loop {
                    self.visit_stmt(post);
                }
                self.visit_stmt(&for_stmt.body);
            }
            Stmt::While(while_stmt) => {
                self.visit_expr(&while_stmt.condition);
                self.visit_stmt(&while_stmt.body);
            }
            Stmt::DoWhile(do_while) => {
                self.visit_stmt(&do_while.body);
                self.visit_expr(&do_while.condition);
            }
            Stmt::Expr(expr_stmt) => self.visit_expr(&expr_stmt.expr),
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.visit_expr(expr);
                }
            }
            Stmt::VarDecl(var_decl) => {
                if let Some(value) = &var_decl.value {
                    self.visit_expr(value);
                }
            }
            Stmt::Try(try_stmt) => {
                self.visit_expr(&try_stmt.guarded_expr);
                self.visit_block(&try_stmt.body);
                for clause in &try_stmt.catch_clauses {
                    self.visit_block(&clause.body);
                }
            }
            _ => {}
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Call(call) => {
                self.handle_call(call);
                // Visit callee and arguments
                self.visit_expr(&call.callee);
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.visit_expr(arg);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.visit_expr(&arg.value);
                        }
                    }
                }
            }
            Expr::Binary(bin) => {
                self.visit_expr(&bin.left);
                self.visit_expr(&bin.right);
            }
            Expr::Unary(unary) => {
                self.visit_expr(&unary.body);
            }
            Expr::Conditional(cond) => {
                self.visit_expr(&cond.cond);
                self.visit_expr(&cond.true_br);
                self.visit_expr(&cond.false_br);
            }
            Expr::Assign(assign) => {
                self.visit_expr(&assign.left);
                self.visit_expr(&assign.right);
            }
            Expr::Member(member) => {
                self.visit_expr(&member.base);
            }
            Expr::Index(index) => {
                self.visit_expr(&index.base_expr);
                if let Some(idx) = &index.index {
                    self.visit_expr(idx);
                }
            }
            Expr::Tuple(tuple) => {
                for elem in tuple.elems.iter().flatten() {
                    self.visit_expr(elem);
                }
            }
            _ => {}
        }
    }

    fn handle_call(&mut self, call: &CallExpr) {
        let Some(caller) = &self.current_function else { return };
        
        let is_external = self.is_external_call(call);
        
        let site = CallSite {
            caller: caller.clone(),
            call_expr: call.clone(),
            loc: call.loc,
            is_external,
            resolved_callee: None, // TODO: implement callee resolution
        };
        
        self.call_graph.add_call_site(site);
    }

    fn is_external_call(&self, call: &CallExpr) -> bool {
        // Check if it's a low-level call (call, delegatecall, staticcall)
        if let Expr::Member(member) = call.callee.as_ref() {
            let method = member.member.base.as_str();
            if matches!(method, "call" | "delegatecall" | "staticcall" | "transfer" | "send") {
                return true;
            }
        }
        
        // Check for function calls on contract instances
        if let Expr::Member(member) = call.callee.as_ref() {
            // If the base is not 'this' or 'super', it might be external
            if let Expr::Ident(ident) = member.base.as_ref() {
                let name = ident.name.base.as_str();
                if name != "this" && name != "super" {
                    // Could be external - need type info to confirm
                    return call.kind == CallKind::FuncCall;
                }
            }
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_graph_new() {
        let cg = CallGraph::new();
        assert!(cg.call_sites.is_empty());
        assert!(cg.external_call_sites.is_empty());
    }
}
