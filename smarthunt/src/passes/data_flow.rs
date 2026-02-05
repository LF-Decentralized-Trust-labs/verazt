//! Data Flow Analysis Pass.
//!
//! This pass performs data flow analysis including taint tracking.

use crate::engine::context::{AnalysisContext, TaintGraph, TaintSource, TaintSink, DefUseChains};
use crate::graph::FunctionId;
use crate::passes::{AnalysisPass, PassId, PassResult};
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, Name, SourceUnit, SourceUnitElem, Stmt,
};
use std::collections::{HashMap, HashSet};

/// Pass for data flow analysis.
pub struct DataFlowPass;

impl DataFlowPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DataFlowPass {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisPass for DataFlowPass {
    fn id(&self) -> PassId {
        PassId::DataFlow
    }

    fn name(&self) -> &'static str {
        "Data Flow Analysis"
    }

    fn description(&self) -> &'static str {
        "Track how data flows through functions (taint analysis, reaching definitions)"
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::Cfg, PassId::CallGraph]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let mut analyzer = DataFlowAnalyzer::new();
        
        for source_unit in &context.source_units {
            analyzer.visit_source_unit(source_unit);
        }
        
        context.taint_graph = Some(analyzer.taint_graph);
        context.def_use_chains = Some(analyzer.def_use);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.taint_graph.is_some() && context.def_use_chains.is_some()
    }
}

/// Data flow analyzer.
struct DataFlowAnalyzer {
    taint_graph: TaintGraph,
    def_use: DefUseChains,
    current_contract: Option<Name>,
    current_function: Option<FunctionId>,
    current_tainted: HashSet<Name>,
}

impl DataFlowAnalyzer {
    fn new() -> Self {
        Self {
            taint_graph: TaintGraph::new(),
            def_use: DefUseChains::new(),
            current_contract: None,
            current_function: None,
            current_tainted: HashSet::new(),
        }
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
                    self.visit_function(func, None);
                }
                _ => {}
            }
        }
    }

    fn visit_contract(&mut self, contract: &ContractDef) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.visit_function(func, Some(contract));
            }
        }
    }

    fn visit_function(&mut self, func: &FuncDef, contract: Option<&ContractDef>) {
        let func_id = FunctionId::from_func(func, contract);
        self.current_function = Some(func_id.clone());
        self.current_tainted.clear();
        
        // Mark function parameters as tainted (user input)
        for param in &func.params {
            let name = &param.name;
            self.current_tainted.insert(name.clone());
            self.taint_graph.tainted_vars
                .entry(func_id.clone())
                .or_default()
                .insert(name.clone());
            self.taint_graph.sources
                .insert(
                    (func_id.clone(), name.clone()),
                    vec![TaintSource::FunctionParameter(name.clone())],
                );
        }
        
        // Analyze function body
        if let Some(body) = &func.body {
            self.visit_block(body);
        }
        
        self.current_function = None;
    }

    fn visit_block(&mut self, block: &Block) {
        for stmt in &block.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(block) => self.visit_block(block),
            
            Stmt::VarDecl(var_decl) => {
                if let Some(value) = &var_decl.value {
                    let is_tainted = self.expr_is_tainted(value);
                    
                    for var in var_decl.var_decls.iter().flatten() {
                        // Record definition
                        if let Some(loc) = var_decl.loc {
                            self.def_use.definitions
                                .entry(var.name.clone())
                                .or_default()
                                .push(loc);
                        }
                        
                        // Propagate taint
                        if is_tainted {
                            self.mark_tainted(&var.name, self.get_expr_sources(value));
                        }
                    }
                }
            }
            
            Stmt::Expr(expr_stmt) => {
                if let Expr::Assign(assign) = &expr_stmt.expr {
                    let is_tainted = self.expr_is_tainted(&assign.right);
                    
                    // Get assigned variable name
                    if let Some(name) = self.get_assigned_var(&assign.left) {
                        // Record definition
                        if let Some(loc) = expr_stmt.loc {
                            self.def_use.definitions
                                .entry(name.clone())
                                .or_default()
                                .push(loc);
                        }
                        
                        // Propagate taint
                        if is_tainted {
                            self.mark_tainted(&name, self.get_expr_sources(&assign.right));
                        }
                    }
                }
                
                // Check for sensitive sinks
                self.check_sinks(&expr_stmt.expr);
            }
            
            Stmt::If(if_stmt) => {
                self.check_expr_uses(&if_stmt.condition);
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
                    self.check_expr_uses(cond);
                }
                if let Some(post) = &for_stmt.post_loop {
                    self.visit_stmt(post);
                }
                self.visit_stmt(&for_stmt.body);
            }
            
            Stmt::While(while_stmt) => {
                self.check_expr_uses(&while_stmt.condition);
                self.visit_stmt(&while_stmt.body);
            }
            
            Stmt::DoWhile(do_while) => {
                self.visit_stmt(&do_while.body);
                self.check_expr_uses(&do_while.condition);
            }
            
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.check_expr_uses(expr);
                }
            }
            
            Stmt::Try(try_stmt) => {
                self.check_expr_uses(&try_stmt.guarded_expr);
                self.visit_block(&try_stmt.body);
                for clause in &try_stmt.catch_clauses {
                    self.visit_block(&clause.body);
                }
            }
            
            _ => {}
        }
    }

    fn expr_is_tainted(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(ident) => self.current_tainted.contains(&ident.name),
            
            Expr::Member(member) => {
                // Check for msg.sender, msg.value, tx.origin, block.timestamp
                if let Expr::Ident(base) = member.base.as_ref() {
                    let base_name = base.name.base.as_str();
                    let member_name = member.member.base.as_str();
                    
                    if base_name == "msg" && matches!(member_name, "sender" | "value" | "data") {
                        return true;
                    }
                    if base_name == "tx" && member_name == "origin" {
                        return true;
                    }
                    if base_name == "block" && matches!(member_name, "timestamp" | "number") {
                        return true;
                    }
                }
                
                self.expr_is_tainted(&member.base)
            }
            
            Expr::Binary(bin) => {
                self.expr_is_tainted(&bin.left) || self.expr_is_tainted(&bin.right)
            }
            
            Expr::Unary(unary) => self.expr_is_tainted(&unary.body),
            
            Expr::Call(call) => {
                // External calls are taint sources
                if self.is_external_call(call) {
                    return true;
                }
                
                // Check arguments
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        args.iter().any(|a| self.expr_is_tainted(a))
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        args.iter().any(|a| self.expr_is_tainted(&a.value))
                    }
                }
            }
            
            Expr::Conditional(cond) => {
                self.expr_is_tainted(&cond.cond)
                    || self.expr_is_tainted(&cond.true_br)
                    || self.expr_is_tainted(&cond.false_br)
            }
            
            Expr::Index(index) => {
                self.expr_is_tainted(&index.base_expr)
                    || index.index.as_ref().map(|i| self.expr_is_tainted(i)).unwrap_or(false)
            }
            
            Expr::Tuple(tuple) => {
                tuple.elems.iter().flatten().any(|e| self.expr_is_tainted(e))
            }
            
            _ => false,
        }
    }

    fn get_expr_sources(&self, expr: &Expr) -> Vec<TaintSource> {
        let mut sources = Vec::new();
        self.collect_sources(expr, &mut sources);
        sources
    }

    fn collect_sources(&self, expr: &Expr, sources: &mut Vec<TaintSource>) {
        match expr {
            Expr::Ident(ident) => {
                if let Some(func) = &self.current_function {
                    if let Some(existing) = self.taint_graph.sources.get(&(func.clone(), ident.name.clone())) {
                        sources.extend(existing.clone());
                    }
                }
            }
            
            Expr::Member(member) => {
                if let Expr::Ident(base) = member.base.as_ref() {
                    let base_name = base.name.base.as_str();
                    let member_name = member.member.base.as_str();
                    
                    if base_name == "msg" && member_name == "sender" {
                        sources.push(TaintSource::MsgSender);
                    } else if base_name == "msg" && member_name == "value" {
                        sources.push(TaintSource::MsgValue);
                    } else if base_name == "tx" && member_name == "origin" {
                        sources.push(TaintSource::TxOrigin);
                    } else if base_name == "block" && member_name == "timestamp" {
                        sources.push(TaintSource::BlockTimestamp);
                    }
                }
                self.collect_sources(&member.base, sources);
            }
            
            Expr::Binary(bin) => {
                self.collect_sources(&bin.left, sources);
                self.collect_sources(&bin.right, sources);
            }
            
            Expr::Call(call) if self.is_external_call(call) => {
                sources.push(TaintSource::ExternalCallReturn);
            }
            
            _ => {}
        }
    }

    fn mark_tainted(&mut self, name: &Name, sources: Vec<TaintSource>) {
        self.current_tainted.insert(name.clone());
        
        if let Some(func) = &self.current_function {
            self.taint_graph.tainted_vars
                .entry(func.clone())
                .or_default()
                .insert(name.clone());
            self.taint_graph.sources
                .insert((func.clone(), name.clone()), sources);
        }
    }

    fn check_sinks(&mut self, expr: &Expr) {
        let Some(func) = &self.current_function else { return };
        
        // Check for external call sinks
        if let Expr::Call(call) = expr {
            if self.is_external_call(call) {
                // Check if target is tainted
                if let Expr::Member(member) = call.callee.as_ref() {
                    if self.expr_is_tainted(&member.base) {
                        let sources = self.get_expr_sources(&member.base);
                        self.taint_graph.sinks.push((
                            func.clone(),
                            TaintSink::ExternalCallTarget,
                            sources,
                        ));
                    }
                }
            }
        }
    }

    fn check_expr_uses(&mut self, expr: &Expr) {
        // Record uses for def-use chains
        match expr {
            Expr::Ident(ident) => {
                if let Some(loc) = ident.loc {
                    self.def_use.uses
                        .entry(ident.name.clone())
                        .or_default()
                        .push(loc);
                }
            }
            Expr::Binary(bin) => {
                self.check_expr_uses(&bin.left);
                self.check_expr_uses(&bin.right);
            }
            Expr::Unary(unary) => {
                self.check_expr_uses(&unary.body);
            }
            Expr::Call(call) => {
                self.check_expr_uses(&call.callee);
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.check_expr_uses(arg);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.check_expr_uses(&arg.value);
                        }
                    }
                }
            }
            Expr::Member(member) => {
                self.check_expr_uses(&member.base);
            }
            Expr::Index(index) => {
                self.check_expr_uses(&index.base_expr);
                if let Some(idx) = &index.index {
                    self.check_expr_uses(idx);
                }
            }
            _ => {}
        }
    }

    fn get_assigned_var(&self, expr: &Expr) -> Option<Name> {
        match expr {
            Expr::Ident(ident) => Some(ident.name.clone()),
            Expr::Member(member) => self.get_assigned_var(&member.base),
            Expr::Index(index) => self.get_assigned_var(&index.base_expr),
            Expr::Tuple(tuple) => {
                // Return first non-None element
                tuple.elems.iter().flatten().find_map(|e| self.get_assigned_var(e))
            }
            _ => None,
        }
    }

    fn is_external_call(&self, call: &solidity::ast::CallExpr) -> bool {
        if let Expr::Member(member) = call.callee.as_ref() {
            let method = member.member.base.as_str();
            if matches!(method, "call" | "delegatecall" | "staticcall" | "transfer" | "send") {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_flow_pass() {
        let pass = DataFlowPass::new();
        assert_eq!(pass.id(), PassId::DataFlow);
        assert!(pass.dependencies().contains(&PassId::Cfg));
    }
}
