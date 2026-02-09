//! Reentrancy Detector (DFA-based)
//!
//! Detects potential reentrancy vulnerabilities using data flow analysis.
//!
//! This detector uses the DFA framework to:
//! 1. Build control flow graphs for each function
//! 2. Track external calls (taint sources)
//! 3. Track state mutations after external calls
//! 4. Detect patterns where state is modified after an external call

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{Block, CallArgs, ContractElem, Expr, FuncDef, Loc, SourceUnitElem, Stmt};

/// DFA-based detector for reentrancy vulnerabilities.
///
/// Reentrancy occurs when an external call allows the called contract
/// to re-enter the calling contract before the first invocation is complete,
/// potentially exploiting inconsistent state.
///
/// This detector identifies functions where:
/// 1. An external call is made (call, delegatecall, transfer, send)
/// 2. State variables are modified after the external call
#[derive(Debug, Default)]
pub struct ReentrancyDfaDetector;

impl ReentrancyDfaDetector {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a function for reentrancy patterns.
    fn check_function(&self, func: &FuncDef, contract_name: &str, bugs: &mut Vec<Bug>) {
        // Skip if function has nonReentrant modifier
        for modifier in &func.modifier_invocs {
            if let Expr::Ident(ident) = modifier.callee.as_ref() {
                let name = ident.name.base.as_str().to_lowercase();
                if name == "nonreentrant" || name.contains("reentrancy") {
                    return;
                }
            }
        }

        if let Some(body) = &func.body {
            let mut analyzer = ReentrancyAnalyzer::new();
            analyzer.analyze_block(body);

            for issue in analyzer.violations {
                let func_name = func.name.base.as_str();
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "Potential reentrancy in '{}.{}': state modification after external call. \
                         External call at line {}, state update at line {}.",
                        contract_name,
                        func_name,
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

/// Violation found by reentrancy analysis.
struct ReentrancyViolation {
    loc: Loc,
    external_call_line: usize,
    state_update_line: usize,
}

/// Analyzer that tracks external calls and state mutations.
struct ReentrancyAnalyzer {
    /// Whether we've seen an external call in this scope.
    seen_external_call: bool,
    /// Location of the first external call.
    external_call_loc: Option<Loc>,
    /// Detected violations.
    violations: Vec<ReentrancyViolation>,
}

impl ReentrancyAnalyzer {
    fn new() -> Self {
        Self { seen_external_call: false, external_call_loc: None, violations: Vec::new() }
    }

    fn analyze_block(&mut self, block: &Block) {
        for stmt in &block.body {
            self.analyze_stmt(stmt);
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(block) => {
                self.analyze_block(block);
            }

            Stmt::Expr(expr_stmt) => {
                // Check for external calls
                if let Some(call_loc) = self.find_external_call(&expr_stmt.expr) {
                    if !self.seen_external_call {
                        self.seen_external_call = true;
                        self.external_call_loc = Some(call_loc);
                    }
                }

                // Check for state updates after external call
                if self.seen_external_call {
                    self.check_state_write(&expr_stmt.expr);
                }
            }

            Stmt::If(if_stmt) => {
                // Check condition for external calls
                if let Some(call_loc) = self.find_external_call(&if_stmt.condition) {
                    if !self.seen_external_call {
                        self.seen_external_call = true;
                        self.external_call_loc = Some(call_loc);
                    }
                }

                // Analyze branches (conservative: consider both paths)
                let saved = self.seen_external_call;
                let saved_loc = self.external_call_loc;

                self.analyze_stmt(&if_stmt.true_branch);

                if let Some(false_br) = &if_stmt.false_branch {
                    // Restore state for false branch, then analyze
                    let true_seen = self.seen_external_call;
                    let true_loc = self.external_call_loc;
                    self.seen_external_call = saved;
                    self.external_call_loc = saved_loc;
                    self.analyze_stmt(false_br);

                    // After both branches: merge (seen in either branch)
                    self.seen_external_call = true_seen || self.seen_external_call;
                    if self.external_call_loc.is_none() {
                        self.external_call_loc = true_loc;
                    }
                }
            }

            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.analyze_stmt(pre);
                }
                self.analyze_stmt(&for_stmt.body);
                if let Some(post) = &for_stmt.post_loop {
                    self.analyze_stmt(post);
                }
            }

            Stmt::While(while_stmt) => {
                self.analyze_stmt(&while_stmt.body);
            }

            Stmt::DoWhile(do_while) => {
                self.analyze_stmt(&do_while.body);
            }

            Stmt::VarDecl(var_decl) => {
                if let Some(value) = &var_decl.value {
                    if let Some(call_loc) = self.find_external_call(value) {
                        if !self.seen_external_call {
                            self.seen_external_call = true;
                            self.external_call_loc = Some(call_loc);
                        }
                    }
                }
            }

            Stmt::Try(try_stmt) => {
                if let Some(call_loc) = self.find_external_call(&try_stmt.guarded_expr) {
                    if !self.seen_external_call {
                        self.seen_external_call = true;
                        self.external_call_loc = Some(call_loc);
                    }
                }
                self.analyze_block(&try_stmt.body);
                for catch in &try_stmt.catch_clauses {
                    self.analyze_block(&catch.body);
                }
            }

            _ => {}
        }
    }

    /// Check if an expression contains a state write.
    fn check_state_write(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign(assign) => {
                if self.is_state_variable(&assign.left) {
                    if let Some(call_loc) = self.external_call_loc {
                        let update_loc = assign.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        self.violations.push(ReentrancyViolation {
                            loc: update_loc,
                            external_call_line: call_loc.start_line,
                            state_update_line: update_loc.start_line,
                        });
                    }
                }
            }
            Expr::Call(call) => {
                // Check for state-modifying calls like array.push(), map operations
                if let Expr::Member(member) = call.callee.as_ref() {
                    let method = member.member.base.as_str();
                    if matches!(method, "push" | "pop") && self.is_state_variable(&member.base) {
                        if let Some(call_loc) = self.external_call_loc {
                            let update_loc = call.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                            self.violations.push(ReentrancyViolation {
                                loc: update_loc,
                                external_call_line: call_loc.start_line,
                                state_update_line: update_loc.start_line,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Check if an expression refers to a state variable.
    fn is_state_variable(&self, expr: &Expr) -> bool {
        match expr {
            // Simple identifier (could be a state variable)
            Expr::Ident(_) => true,
            // Member access on state variable (e.g., balances[addr])
            Expr::Member(m) => self.is_state_variable(&m.base),
            // Index access on state variable (e.g., mapping[key])
            Expr::Index(i) => self.is_state_variable(&i.base_expr),
            _ => false,
        }
    }

    /// Find an external call in an expression.
    fn find_external_call(&self, expr: &Expr) -> Option<Loc> {
        match expr {
            Expr::Call(call) => {
                if self.is_external_call_expr(&call.callee) {
                    return call.loc;
                }
                // Check arguments
                match &call.args {
                    CallArgs::Unnamed(args) => {
                        for arg in args {
                            if let Some(loc) = self.find_external_call(arg) {
                                return Some(loc);
                            }
                        }
                    }
                    CallArgs::Named(args) => {
                        for arg in args {
                            if let Some(loc) = self.find_external_call(&arg.value) {
                                return Some(loc);
                            }
                        }
                    }
                }
                None
            }
            Expr::CallOpts(call_opts) => {
                if let Expr::Member(member) = call_opts.callee.as_ref() {
                    let method = member.member.base.as_str();
                    if matches!(
                        method,
                        "call" | "delegatecall" | "staticcall" | "transfer" | "send"
                    ) {
                        return call_opts.loc;
                    }
                }
                None
            }
            Expr::Member(member) => self.find_external_call(&member.base),
            Expr::Binary(binary) => self
                .find_external_call(&binary.left)
                .or_else(|| self.find_external_call(&binary.right)),
            Expr::Unary(unary) => self.find_external_call(&unary.body),
            Expr::Assign(assign) => self
                .find_external_call(&assign.left)
                .or_else(|| self.find_external_call(&assign.right)),
            _ => None,
        }
    }

    /// Check if an expression is an external call callee.
    fn is_external_call_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Member(member) => {
                let method = member.member.base.as_str();
                matches!(method, "call" | "delegatecall" | "staticcall" | "transfer" | "send")
            }
            _ => false,
        }
    }
}

impl Pass for ReentrancyDfaDetector {
    fn id(&self) -> PassId {
        PassId::Reentrancy
    }

    fn name(&self) -> &'static str {
        "Reentrancy (DFA)"
    }

    fn description(&self) -> &'static str {
        "Detects potential reentrancy vulnerabilities using data flow analysis. \
         Finds state modifications after external calls."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::CallGraph, PassId::ModifierAnalysis]
    }
}

impl BugDetectionPass for ReentrancyDfaDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                match elem {
                    SourceUnitElem::Contract(contract) => {
                        let contract_name = &contract.name.base;
                        for elem in &contract.body {
                            if let ContractElem::Func(func) = elem {
                                self.check_function(func, contract_name, &mut bugs);
                            }
                        }
                    }
                    SourceUnitElem::Func(func) => {
                        self.check_function(func, "global", &mut bugs);
                    }
                    _ => {}
                }
            }
        }

        Ok(bugs)
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

    fn recommendation(&self) -> &'static str {
        "Follow the Checks-Effects-Interactions pattern: perform all state changes \
         before making external calls. Consider using a reentrancy guard \
         (e.g., OpenZeppelin's ReentrancyGuard)."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-107",
            "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reentrancy_detector() {
        let detector = ReentrancyDfaDetector::new();
        assert_eq!(detector.id(), PassId::Reentrancy);
        assert_eq!(detector.risk_level(), RiskLevel::Critical);
        assert_eq!(detector.swc_ids(), vec![107]);
    }
}
