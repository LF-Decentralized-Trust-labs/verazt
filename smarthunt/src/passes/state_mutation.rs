//! State Mutation Analysis Pass.
//!
//! This pass analyzes state variable read/write patterns.

use crate::engine::context::{AnalysisContext, StateMutationMap};
use crate::graph::FunctionId;
use crate::passes::{AnalysisPass, PassId, PassResult};
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, Name, SourceUnit, SourceUnitElem, Stmt,
};
use std::collections::HashSet;

/// Pass for analyzing state mutations.
pub struct StateMutationPass;

impl StateMutationPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StateMutationPass {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisPass for StateMutationPass {
    fn id(&self) -> PassId {
        PassId::StateMutation
    }

    fn name(&self) -> &'static str {
        "State Mutation Analysis"
    }

    fn description(&self) -> &'static str {
        "Track state changes across function calls"
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::CallGraph]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let mut analyzer = StateMutationAnalyzer::new(context);
        analyzer.analyze();
        context.state_mutations = Some(analyzer.mutation_map);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.state_mutations.is_some()
    }
}

/// State mutation analyzer.
struct StateMutationAnalyzer<'a> {
    context: &'a AnalysisContext,
    mutation_map: StateMutationMap,
    state_vars: HashSet<Name>,
    current_function: Option<FunctionId>,
}

impl<'a> StateMutationAnalyzer<'a> {
    fn new(context: &'a AnalysisContext) -> Self {
        // Collect all state variable names
        let mut state_vars = HashSet::new();
        
        if let Some(symbols) = &context.symbols {
            for ((_, var_name), _) in &symbols.state_variables {
                state_vars.insert(var_name.clone());
            }
        }
        
        Self {
            context,
            mutation_map: StateMutationMap::new(),
            state_vars,
            current_function: None,
        }
    }

    fn analyze(&mut self) {
        for source_unit in &self.context.source_units {
            self.visit_source_unit(source_unit);
        }
    }

    fn visit_source_unit(&mut self, source_unit: &SourceUnit) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    // Add contract's state variables
                    for body_elem in &contract.body {
                        if let ContractElem::Var(var) = body_elem {
                            self.state_vars.insert(var.name.clone());
                        }
                    }
                    
                    self.visit_contract(contract);
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
        self.current_function = Some(func_id);
        
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
                    self.check_reads(value);
                }
            }
            
            Stmt::Expr(expr_stmt) => {
                if let Expr::Assign(assign) = &expr_stmt.expr {
                    // Check for state variable write
                    self.check_write(&assign.left);
                    // Check for reads in the RHS
                    self.check_reads(&assign.right);
                } else {
                    self.check_reads(&expr_stmt.expr);
                }
            }
            
            Stmt::If(if_stmt) => {
                self.check_reads(&if_stmt.condition);
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
                    self.check_reads(cond);
                }
                if let Some(post) = &for_stmt.post_loop {
                    self.visit_stmt(post);
                }
                self.visit_stmt(&for_stmt.body);
            }
            
            Stmt::While(while_stmt) => {
                self.check_reads(&while_stmt.condition);
                self.visit_stmt(&while_stmt.body);
            }
            
            Stmt::DoWhile(do_while) => {
                self.visit_stmt(&do_while.body);
                self.check_reads(&do_while.condition);
            }
            
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.check_reads(expr);
                }
            }
            
            Stmt::Try(try_stmt) => {
                self.check_reads(&try_stmt.guarded_expr);
                self.visit_block(&try_stmt.body);
                for clause in &try_stmt.catch_clauses {
                    self.visit_block(&clause.body);
                }
            }
            
            _ => {}
        }
    }

    fn check_write(&mut self, expr: &Expr) {
        if let Some(name) = self.get_base_name(expr) {
            if self.state_vars.contains(&name) {
                if let Some(func) = &self.current_function {
                    self.mutation_map.add_write(func.clone(), name);
                }
            }
        }
    }

    fn check_reads(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(ident) => {
                if self.state_vars.contains(&ident.name) {
                    if let Some(func) = &self.current_function {
                        self.mutation_map.add_read(func.clone(), ident.name.clone());
                    }
                }
            }
            
            Expr::Member(member) => {
                if let Some(name) = self.get_base_name(expr) {
                    if self.state_vars.contains(&name) {
                        if let Some(func) = &self.current_function {
                            self.mutation_map.add_read(func.clone(), name);
                        }
                    }
                }
                self.check_reads(&member.base);
            }
            
            Expr::Binary(bin) => {
                self.check_reads(&bin.left);
                self.check_reads(&bin.right);
            }
            
            Expr::Unary(unary) => {
                self.check_reads(&unary.body);
            }
            
            Expr::Call(call) => {
                self.check_reads(&call.callee);
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.check_reads(arg);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.check_reads(&arg.value);
                        }
                    }
                }
            }
            
            Expr::Index(index) => {
                self.check_reads(&index.base_expr);
                if let Some(idx) = &index.index {
                    self.check_reads(idx);
                }
            }
            
            Expr::Conditional(cond) => {
                self.check_reads(&cond.cond);
                self.check_reads(&cond.true_br);
                self.check_reads(&cond.false_br);
            }
            
            Expr::Tuple(tuple) => {
                for elem in tuple.elems.iter().flatten() {
                    self.check_reads(elem);
                }
            }
            
            _ => {}
        }
    }

    fn get_base_name(&self, expr: &Expr) -> Option<Name> {
        match expr {
            Expr::Ident(ident) => Some(ident.name.clone()),
            Expr::Member(member) => self.get_base_name(&member.base),
            Expr::Index(index) => self.get_base_name(&index.base_expr),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_mutation_pass() {
        let pass = StateMutationPass::new();
        assert_eq!(pass.id(), PassId::StateMutation);
    }
}
