//! Dead code detector.
//!
//! Detects unreachable or unused code.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{ContractDef, ContractElem, FuncVis, Loc, Stmt};
use std::collections::HashSet;

/// Detector for dead code.
pub struct DeadCodeDetector;

impl DeadCodeDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeadCodeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for DeadCodeDetector {
    fn id(&self) -> &'static str {
        "dead-code"
    }

    fn name(&self) -> &'static str {
        "Dead Code"
    }

    fn description(&self) -> &'static str {
        "Detects unreachable code (code after return/revert/throw) and unused \
         private/internal functions that are never called."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::Cfg, PassId::CallGraph]
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Refactoring
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![561] // CWE-561: Dead Code
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();
        
        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let solidity::ast::SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(&contract.name.base, &contract, context, &mut bugs);
                }
            }
        }
        
        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Remove unreachable code and unused functions to improve code clarity \
         and reduce gas costs during deployment."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://cwe.mitre.org/data/definitions/561.html",
        ]
    }
}

impl DeadCodeDetector {
    fn check_contract(
        &self,
        contract_name: &str,
        contract: &ContractDef,
        context: &AnalysisContext,
        bugs: &mut Vec<Bug>,
    ) {
        // Collect all function names for internal call tracking
        let mut private_functions: HashSet<String> = HashSet::new();
        let mut called_functions: HashSet<String> = HashSet::new();
        
        // First pass: collect private/internal functions
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                let is_private_or_internal = matches!(
                    func.visibility,
                    FuncVis::Private | FuncVis::Internal
                );
                
                if is_private_or_internal {
                    private_functions.insert(func.name.base.clone());
                }
            }
        }
        
        // Second pass: check each function
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                if let Some(func_body) = &func.body {
                    // Check for unreachable code
                    self.check_unreachable_code(
                        contract_name,
                        func.name.base.as_str(),
                        func_body,
                        bugs,
                    );
                    
                    // Collect internal calls
                    self.collect_internal_calls_from_block(func_body, &mut called_functions);
                }
            }
        }
        
        // Use call graph if available
        if let Some(call_graph) = &context.call_graph {
            for (func_id, _edges) in &call_graph.edges_by_caller {
                called_functions.insert(func_id.name.base.clone());
            }
        }
        
        // Report unused private/internal functions
        for func_name in &private_functions {
            if !called_functions.contains(func_name) {
                // Find the function location
                for elem in &contract.body {
                    if let ContractElem::Func(func) = elem {
                        if &func.name.base == func_name {
                            let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                            let bug = create_bug(
                                self,
                                Some(&format!(
                                    "Private/internal function '{}' in contract '{}' is never called. \
                                     Consider removing it.",
                                    func_name, contract_name
                                )),
                                loc,
                            );
                            bugs.push(bug);
                            break;
                        }
                    }
                }
            }
        }
    }

    fn check_unreachable_code(
        &self,
        contract_name: &str,
        func_name: &str,
        block: &solidity::ast::Block,
        bugs: &mut Vec<Bug>,
    ) {
        let mut found_terminating = false;
        
        for s in &block.body {
            if found_terminating {
                // This code is unreachable
                let loc = self.get_stmt_loc(s);
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "Unreachable code in function '{}' of contract '{}' after return/revert.",
                        func_name, contract_name
                    )),
                    loc,
                );
                bugs.push(bug);
                break; // Only report once per block
            }
            
            if self.is_terminating_statement(s) {
                found_terminating = true;
            }
            
            // Recursively check nested blocks
            match s {
                Stmt::Block(inner_block) => {
                    self.check_unreachable_code(contract_name, func_name, inner_block, bugs);
                }
                Stmt::If(if_stmt) => {
                    if let Stmt::Block(true_block) = &*if_stmt.true_branch {
                        self.check_unreachable_code(contract_name, func_name, true_block, bugs);
                    }
                    if let Some(false_br) = &if_stmt.false_branch {
                        if let Stmt::Block(false_block) = &**false_br {
                            self.check_unreachable_code(contract_name, func_name, false_block, bugs);
                        }
                    }
                }
                Stmt::While(while_stmt) => {
                    if let Stmt::Block(body_block) = &*while_stmt.body {
                        self.check_unreachable_code(contract_name, func_name, body_block, bugs);
                    }
                }
                Stmt::For(for_stmt) => {
                    if let Stmt::Block(body_block) = &*for_stmt.body {
                        self.check_unreachable_code(contract_name, func_name, body_block, bugs);
                    }
                }
                Stmt::DoWhile(do_while) => {
                    if let Stmt::Block(body_block) = &*do_while.body {
                        self.check_unreachable_code(contract_name, func_name, body_block, bugs);
                    }
                }
                _ => {}
            }
        }
    }

    fn is_terminating_statement(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return(_) => true,
            Stmt::Revert(_) => true,
            Stmt::Expr(expr_stmt) => {
                // Check for revert() or require() with false
                if let solidity::ast::Expr::Call(call) = &expr_stmt.expr {
                    if let solidity::ast::Expr::Ident(ident) = &*call.callee {
                        return matches!(ident.name.base.as_str(), "revert" | "assert");
                    }
                }
                false
            }
            Stmt::Asm(_) => {
                // Assembly might contain revert, but we can't easily tell
                false
            }
            _ => false,
        }
    }

    fn collect_internal_calls_from_block(&self, block: &solidity::ast::Block, called: &mut HashSet<String>) {
        for s in &block.body {
            self.collect_internal_calls(s, called);
        }
    }

    fn collect_internal_calls(&self, stmt: &Stmt, called: &mut HashSet<String>) {
        match stmt {
            Stmt::Block(block) => {
                self.collect_internal_calls_from_block(block, called);
            }
            Stmt::Expr(expr_stmt) => {
                self.collect_calls_from_expr(&expr_stmt.expr, called);
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.collect_calls_from_expr(expr, called);
                }
            }
            Stmt::VarDecl(var_decl) => {
                if let Some(expr) = &var_decl.value {
                    self.collect_calls_from_expr(expr, called);
                }
            }
            Stmt::If(if_stmt) => {
                self.collect_calls_from_expr(&if_stmt.condition, called);
                self.collect_internal_calls(&if_stmt.true_branch, called);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.collect_internal_calls(false_br, called);
                }
            }
            Stmt::While(while_stmt) => {
                self.collect_calls_from_expr(&while_stmt.condition, called);
                self.collect_internal_calls(&while_stmt.body, called);
            }
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.collect_internal_calls(pre, called);
                }
                if let Some(cond) = &for_stmt.condition {
                    self.collect_calls_from_expr(cond, called);
                }
                if let Some(post) = &for_stmt.post_loop {
                    self.collect_internal_calls(post, called);
                }
                self.collect_internal_calls(&for_stmt.body, called);
            }
            Stmt::DoWhile(do_while) => {
                self.collect_internal_calls(&do_while.body, called);
                self.collect_calls_from_expr(&do_while.condition, called);
            }
            _ => {}
        }
    }

    fn collect_calls_from_expr(&self, expr: &solidity::ast::Expr, called: &mut HashSet<String>) {
        match expr {
            solidity::ast::Expr::Call(call) => {
                if let solidity::ast::Expr::Ident(ident) = &*call.callee {
                    called.insert(ident.name.base.clone());
                }
                self.collect_calls_from_expr(&call.callee, called);
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.collect_calls_from_expr(arg, called);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.collect_calls_from_expr(&arg.value, called);
                        }
                    }
                }
            }
            solidity::ast::Expr::Member(member) => {
                self.collect_calls_from_expr(&member.base, called);
            }
            solidity::ast::Expr::Index(index) => {
                self.collect_calls_from_expr(&index.base_expr, called);
                if let Some(idx) = &index.index {
                    self.collect_calls_from_expr(idx, called);
                }
            }
            solidity::ast::Expr::Unary(unary) => {
                self.collect_calls_from_expr(&unary.body, called);
            }
            solidity::ast::Expr::Binary(binary) => {
                self.collect_calls_from_expr(&binary.left, called);
                self.collect_calls_from_expr(&binary.right, called);
            }
            solidity::ast::Expr::Conditional(cond_expr) => {
                self.collect_calls_from_expr(&cond_expr.cond, called);
                self.collect_calls_from_expr(&cond_expr.true_br, called);
                self.collect_calls_from_expr(&cond_expr.false_br, called);
            }
            solidity::ast::Expr::Assign(assign) => {
                self.collect_calls_from_expr(&assign.left, called);
                self.collect_calls_from_expr(&assign.right, called);
            }
            solidity::ast::Expr::Tuple(tuple) => {
                for elem in &tuple.elems {
                    if let Some(e) = elem {
                        self.collect_calls_from_expr(e, called);
                    }
                }
            }
            _ => {}
        }
    }

    fn get_stmt_loc(&self, stmt: &Stmt) -> Loc {
        match stmt {
            Stmt::Block(b) => b.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Expr(e) => e.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Return(r) => r.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::VarDecl(v) => v.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::If(i) => i.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::While(w) => w.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::For(f) => f.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::DoWhile(d) => d.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Continue(c) => c.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Break(b) => b.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Try(t) => t.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Revert(r) => r.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Emit(e) => e.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Asm(a) => a.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Placeholder(p) => p.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
            Stmt::Throw(t) => t.loc.unwrap_or(Loc::new(1, 1, 1, 1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_code_detector() {
        let detector = DeadCodeDetector::new();
        assert_eq!(detector.id(), "dead-code");
        assert_eq!(detector.cwe_ids(), vec![561]);
    }
}
