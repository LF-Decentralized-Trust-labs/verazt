//! Reentrancy detector.
//!
//! Detects potential reentrancy vulnerabilities.
#![allow(dead_code, unused_variables)]

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug, AnalysisContext};
use solidity::analysis::FunctionId;
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, Loc, SourceUnit, SourceUnitElem, Stmt,
};

/// Detector for reentrancy vulnerabilities.
pub struct ReentrancyDetector;

impl ReentrancyDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReentrancyDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for ReentrancyDetector {
    fn id(&self) -> &'static str {
        "reentrancy"
    }

    fn name(&self) -> &'static str {
        "Reentrancy Vulnerability"
    }

    fn description(&self) -> &'static str {
        "External calls are made before state updates, which may allow an attacker \
         to re-enter the function and exploit the inconsistent state."
    }


    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Critical
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![841] // CWE-841: Improper Enforcement of Behavioral Workflow
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![107] // SWC-107: Reentrancy
    }

    fn detect(&self, _context: &AnalysisContext) -> Vec<Bug> {
        // TODO: Reimplement using new analysis framework
        // This detector requires call graph and state mutation analysis
        vec![]
    }

    fn recommendation(&self) -> &'static str {
        "Follow the Checks-Effects-Interactions pattern: perform all state changes \
         before making external calls. Consider using a reentrancy guard (e.g., OpenZeppelin's ReentrancyGuard)."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-107",
            "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/",
        ]
    }
}

impl ReentrancyDetector {
    fn visit_source_unit(&self, source_unit: &SourceUnit, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.visit_contract(contract, context, bugs);
                }
                SourceUnitElem::Func(func) => {
                    self.visit_function(func, None, context, bugs);
                }
                _ => {}
            }
        }
    }

    fn visit_contract(&self, contract: &ContractDef, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.visit_function(func, Some(contract), context, bugs);
            }
        }
    }

    fn visit_function(
        &self,
        func: &FuncDef,
        contract: Option<&ContractDef>,
        context: &AnalysisContext,
        bugs: &mut Vec<Bug>,
    ) {
        let func_id = FunctionId::from_func(func, contract);
        
        // Check if function has nonReentrant modifier
        for modifier in &func.modifier_invocs {
            if let Expr::Ident(ident) = modifier.callee.as_ref() {
                if ident.name.base.as_str() == "nonReentrant" {
                    return; // Protected by reentrancy guard
                }
            }
        }
        
        // Check if function modifies state after external calls
        if let Some(body) = &func.body {
            let mut analyzer = ReentrancyAnalyzer::new(&func_id, context);
            analyzer.analyze_block(body);
            
            for issue in analyzer.issues {
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "State update after external call. External call at line {}, state update at line {}.",
                        issue.external_call_line,
                        issue.state_update_line,
                    )),
                    issue.loc,
                );
                bugs.push(bug);
            }
        }
    }
}

/// Issue found by reentrancy analysis.
struct ReentrancyIssue {
    loc: Loc,
    external_call_line: usize,
    state_update_line: usize,
}

/// Analyzer for reentrancy in a single function.
struct ReentrancyAnalyzer<'a> {
    func_id: &'a FunctionId,
    context: &'a AnalysisContext,
    external_call_locs: Vec<Loc>,
    issues: Vec<ReentrancyIssue>,
    seen_external_call: bool,
}

impl<'a> ReentrancyAnalyzer<'a> {
    fn new(func_id: &'a FunctionId, context: &'a AnalysisContext) -> Self {
        Self {
            func_id,
            context,
            external_call_locs: Vec::new(),
            issues: Vec::new(),
            seen_external_call: false,
        }
    }

    fn analyze_block(&mut self, block: &Block) {
        for stmt in &block.body {
            self.analyze_stmt(stmt);
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(block) => self.analyze_block(block),
            
            Stmt::Expr(expr_stmt) => {
                // Check for external calls
                if let Some(loc) = self.find_external_call(&expr_stmt.expr) {
                    self.external_call_locs.push(loc);
                    self.seen_external_call = true;
                }
                
                // Check for state updates after external calls
                if self.seen_external_call {
                    if let Expr::Assign(assign) = &expr_stmt.expr {
                        if self.is_state_update(&assign.left) {
                            if let (Some(call_loc), Some(assign_loc)) = 
                                (self.external_call_locs.last(), assign.loc) {
                                self.issues.push(ReentrancyIssue {
                                    loc: assign_loc,
                                    external_call_line: call_loc.start_line,
                                    state_update_line: assign_loc.start_line,
                                });
                            }
                        }
                    }
                }
            }
            
            Stmt::VarDecl(var_decl) => {
                // Check for external call in initialization
                if let Some(value) = &var_decl.value {
                    if let Some(loc) = self.find_external_call(value) {
                        self.external_call_locs.push(loc);
                        self.seen_external_call = true;
                    }
                }
            }
            
            Stmt::If(if_stmt) => {
                if let Some(loc) = self.find_external_call(&if_stmt.condition) {
                    self.external_call_locs.push(loc);
                    self.seen_external_call = true;
                }
                self.analyze_stmt(&if_stmt.true_branch);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.analyze_stmt(false_br);
                }
            }
            
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.analyze_stmt(pre);
                }
                if let Some(cond) = &for_stmt.condition {
                    if let Some(loc) = self.find_external_call(cond) {
                        self.external_call_locs.push(loc);
                        self.seen_external_call = true;
                    }
                }
                self.analyze_stmt(&for_stmt.body);
                if let Some(post) = &for_stmt.post_loop {
                    self.analyze_stmt(post);
                }
            }
            
            Stmt::While(while_stmt) => {
                if let Some(loc) = self.find_external_call(&while_stmt.condition) {
                    self.external_call_locs.push(loc);
                    self.seen_external_call = true;
                }
                self.analyze_stmt(&while_stmt.body);
            }
            
            Stmt::Try(try_stmt) => {
                if let Some(loc) = self.find_external_call(&try_stmt.guarded_expr) {
                    self.external_call_locs.push(loc);
                    self.seen_external_call = true;
                }
                self.analyze_block(&try_stmt.body);
                for clause in &try_stmt.catch_clauses {
                    self.analyze_block(&clause.body);
                }
            }
            
            _ => {}
        }
    }

    fn find_external_call(&self, expr: &Expr) -> Option<Loc> {
        match expr {
            Expr::Call(call) => {
                if self.is_external_call(call) {
                    return call.loc;
                }
                // Check callee
                if let Some(loc) = self.find_external_call(&call.callee) {
                    return Some(loc);
                }
                // Check arguments
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            if let Some(loc) = self.find_external_call(arg) {
                                return Some(loc);
                            }
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            if let Some(loc) = self.find_external_call(&arg.value) {
                                return Some(loc);
                            }
                        }
                    }
                }
                None
            }
            Expr::Binary(bin) => {
                self.find_external_call(&bin.left)
                    .or_else(|| self.find_external_call(&bin.right))
            }
            Expr::Unary(unary) => self.find_external_call(&unary.body),
            Expr::Conditional(cond) => {
                self.find_external_call(&cond.cond)
                    .or_else(|| self.find_external_call(&cond.true_br))
                    .or_else(|| self.find_external_call(&cond.false_br))
            }
            Expr::Member(member) => self.find_external_call(&member.base),
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

    fn is_state_update(&self, expr: &Expr) -> bool {
        // TODO: Re-enable when migrating to new analysis framework
        // Check if the expression is a state variable access
        // if let Some(context) = &self.context.state_mutations {
        //     if let Some(name) = self.get_base_name(expr) {
        //         // Check if this is a known state variable
        //         return context.writes.contains_key(&name);
        //     }
        // }
        
        // Fall back to simple heuristic: check if it's a member access on self/this
        // or a simple identifier that could be a state variable
        match expr {
            Expr::Ident(_) => true, // Could be a state variable
            Expr::Member(member) => {
                if let Expr::Ident(base) = member.base.as_ref() {
                    // Exclude msg, block, tx
                    !matches!(base.name.base.as_str(), "msg" | "block" | "tx")
                } else {
                    true
                }
            }
            Expr::Index(index) => self.is_state_update(&index.base_expr),
            _ => false,
        }
    }

    fn get_base_name(&self, expr: &Expr) -> Option<solidity::ast::Name> {
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
    fn test_reentrancy_detector() {
        let detector = ReentrancyDetector::new();
        assert_eq!(detector.id(), "reentrancy");
        assert_eq!(detector.swc_ids(), vec![107]);
    }
}
