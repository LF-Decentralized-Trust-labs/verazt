//! CEI Violation Detector (DFA-based)
//!
//! Detects violations of the Checks-Effects-Interactions pattern
//! using data flow analysis.
//!
//! The CEI pattern requires that:
//! 1. Checks (conditions, requires) come first
//! 2. Effects (state changes) come second
//! 3. Interactions (external calls) come last

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{Block, CallArgs, ContractElem, Expr, FuncDef, Loc, SourceUnitElem, Stmt};

/// DFA-based detector for CEI (Checks-Effects-Interactions) pattern violations.
#[derive(Debug, Default)]
pub struct CeiViolationDfaDetector;

impl CeiViolationDfaDetector {
    pub fn new() -> Self {
        Self
    }

    fn check_function(&self, func: &FuncDef, contract_name: &str, bugs: &mut Vec<Bug>) {
        // Skip if function has nonReentrant modifier
        for modifier in &func.modifier_invocs {
            if let Expr::Ident(ident) = modifier.callee.as_ref() {
                let name = ident.name.base.as_str().to_lowercase();
                if name == "nonreentrant" {
                    return;
                }
            }
        }

        if let Some(body) = &func.body {
            let mut analyzer = CeiAnalyzer::new();
            analyzer.analyze_block(body);

            let func_name = func.name.base.as_str();
            for issue in analyzer.violations {
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "CEI violation in '{}.{}': state update at line {} occurs after \
                         external call at line {}. This violates the \
                         Checks-Effects-Interactions pattern.",
                        contract_name,
                        func_name,
                        issue.state_update_line,
                        issue.external_call_line,
                    )),
                    issue.loc,
                );
                bugs.push(bug);
            }
        }
    }
}

/// CEI violation detail.
struct CeiViolation {
    loc: Loc,
    external_call_line: usize,
    state_update_line: usize,
}

/// Analyzer for CEI pattern violations.
struct CeiAnalyzer {
    /// Whether we've seen an external call.
    seen_external_call: bool,
    /// Location of the first external call.
    external_call_loc: Option<Loc>,
    /// Detected violations.
    violations: Vec<CeiViolation>,
}

impl CeiAnalyzer {
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
                // Check for external calls first
                if let Some(call_loc) = self.find_external_call(&expr_stmt.expr) {
                    if !self.seen_external_call {
                        self.seen_external_call = true;
                        self.external_call_loc = Some(call_loc);
                    }
                }

                // Check for state updates after external call
                if self.seen_external_call {
                    if let Expr::Assign(assign) = &expr_stmt.expr {
                        if self.is_state_write(&assign.left) {
                            if let Some(call_loc) = self.external_call_loc {
                                let update_loc = assign.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                                self.violations.push(CeiViolation {
                                    loc: update_loc,
                                    external_call_line: call_loc.start_line,
                                    state_update_line: update_loc.start_line,
                                });
                            }
                        }
                    }
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

                // Analyze branches
                self.analyze_stmt(&if_stmt.true_branch);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.analyze_stmt(false_br);
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

    fn find_external_call(&self, expr: &Expr) -> Option<Loc> {
        match expr {
            Expr::Call(call) => {
                if self.is_external_call(&call.callee) {
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
            _ => None,
        }
    }

    fn is_external_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Member(member) => {
                let method = member.member.base.as_str();
                matches!(method, "call" | "delegatecall" | "staticcall" | "transfer" | "send")
            }
            _ => false,
        }
    }

    fn is_state_write(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(_) => true,
            Expr::Member(m) => self.is_state_write(&m.base),
            Expr::Index(i) => self.is_state_write(&i.base_expr),
            _ => false,
        }
    }
}

impl Pass for CeiViolationDfaDetector {
    fn id(&self) -> PassId {
        PassId::CeiViolation
    }

    fn name(&self) -> &'static str {
        "CEI Pattern Violation (DFA)"
    }

    fn description(&self) -> &'static str {
        "Detects violations of the Checks-Effects-Interactions pattern using data flow analysis"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::CallGraph]
    }
}

impl BugDetectionPass for CeiViolationDfaDetector {
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
        RiskLevel::High
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
        "Follow the Checks-Effects-Interactions pattern: perform all checks first, \
         then make state changes, and finally interact with external contracts. \
         Consider using OpenZeppelin's ReentrancyGuard."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-107",
            "https://fravoll.github.io/solidity-patterns/checks_effects_interactions.html",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cei_violation_detector() {
        let detector = CeiViolationDfaDetector::new();
        assert_eq!(detector.id(), PassId::CeiViolation);
        assert_eq!(detector.risk_level(), RiskLevel::High);
        assert_eq!(detector.swc_ids(), vec![107]);
    }
}
