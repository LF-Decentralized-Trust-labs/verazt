//! CEI Violation detector.
//!
//! Detects violations of the Checks-Effects-Interactions pattern.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, Loc, SourceUnit, SourceUnitElem, Stmt,
};

/// Detector for CEI (Checks-Effects-Interactions) pattern violations.
pub struct CeiViolationDetector;

impl CeiViolationDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CeiViolationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for CeiViolationDetector {
    fn id(&self) -> &'static str {
        "cei-violation"
    }

    fn name(&self) -> &'static str {
        "CEI Pattern Violation"
    }

    fn description(&self) -> &'static str {
        "The Checks-Effects-Interactions pattern requires that state changes \
         (effects) happen before external calls (interactions). Violating this \
         pattern can lead to reentrancy vulnerabilities."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::StateMutation]
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

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, context, &mut bugs);
        }

        bugs
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

impl CeiViolationDetector {
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
        // Skip if function has nonReentrant modifier
        for modifier in &func.modifier_invocs {
            if let Expr::Ident(ident) = modifier.callee.as_ref() {
                if ident.name.base.as_str() == "nonReentrant" {
                    return;
                }
            }
        }

        if let Some(body) = &func.body {
            let mut analyzer = CeiAnalyzer::new();
            analyzer.analyze_block(body);

            for issue in analyzer.violations {
                let bug = create_bug(
                    self,
                    Some(&format!(
                        "State update at line {} occurs after external call at line {}.",
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

/// CEI violation issue
struct CeiViolation {
    loc: Loc,
    external_call_line: usize,
    state_update_line: usize,
}

/// Analyzer for CEI pattern violations
struct CeiAnalyzer {
    /// Whether we've seen an external call
    seen_external_call: bool,
    /// Location of the first external call
    external_call_loc: Option<Loc>,
    /// Detected violations
    violations: Vec<CeiViolation>,
}

impl CeiAnalyzer {
    fn new() -> Self {
        Self {
            seen_external_call: false,
            external_call_loc: None,
            violations: Vec::new(),
        }
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
            }
            Expr::CallOpts(call_opts) => {
                if let Expr::Member(member) = call_opts.callee.as_ref() {
                    let method = member.member.base.as_str();
                    if matches!(method, "call" | "delegatecall" | "staticcall" | "transfer" | "send") {
                        return call_opts.loc;
                    }
                }
            }
            Expr::Member(member) => {
                return self.find_external_call(&member.base);
            }
            Expr::Binary(binary) => {
                if let Some(loc) = self.find_external_call(&binary.left) {
                    return Some(loc);
                }
                return self.find_external_call(&binary.right);
            }
            Expr::Unary(unary) => {
                return self.find_external_call(&unary.body);
            }
            _ => {}
        }
        None
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
            Expr::Ident(_) => true, // Simplified: assume all identifiers are state vars
            Expr::Member(member) => {
                // Check for state variable access like this.x or contract.x
                if let Expr::Ident(ident) = member.base.as_ref() {
                    ident.name.base.as_str() != "msg" && ident.name.base.as_str() != "block"
                } else {
                    true
                }
            }
            Expr::Index(index) => self.is_state_write(&index.base_expr),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cei_violation_detector_new() {
        let detector = CeiViolationDetector::new();
        assert_eq!(detector.id(), "cei-violation");
    }
}
