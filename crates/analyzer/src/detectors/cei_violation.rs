//! CEI Violation Detector (SIR structural)
//!
//! Detects violations of the Checks-Effects-Interactions pattern
//! by walking SIR function bodies.
//!
//! The CEI pattern requires that:
//! 1. Checks (conditions, requires) come first
//! 2. Effects (state changes) come second
//! 3. Interactions (external calls) come last

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass::meta::PassLevel;
use crate::analysis::pass::meta::PassRepresentation;
use crate::detectors::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::query as structural;
use scirs::sir::{Decl, MemberDecl, Stmt};
use std::any::TypeId;

/// SIR structural detector for CEI (Checks-Effects-Interactions) pattern
/// violations.
#[derive(Debug, Default)]
pub struct CeiViolationSirDetector;

impl CeiViolationSirDetector {
    pub fn new() -> Self {
        Self
    }

    /// Walk statements sequentially. Track whether we have seen an external
    /// call so far; if a subsequent statement writes to storage, flag a
    /// violation.
    fn check_stmts(
        &self,
        stmts: &[Stmt],
        storage_vars: &[String],
        seen_ext_call: &mut bool,
        bugs: &mut Vec<Bug>,
        contract_name: &str,
        func_name: &str,
    ) {
        for stmt in stmts {
            // First: mark if this statement contains an external call.
            if !*seen_ext_call && self.stmt_has_external_call(stmt) {
                *seen_ext_call = true;
            }

            // Second: if we already saw an external call, check for storage
            // writes.
            if *seen_ext_call && self.stmt_has_storage_write(stmt, storage_vars) {
                bugs.push(Bug::new(
                    self.name(),
                    Some(&format!(
                        "CEI violation in '{}.{}': state update occurs after \
                         an external call. This violates the \
                         Checks-Effects-Interactions pattern.",
                        contract_name, func_name,
                    )),
                    Loc::new(0, 0, 0, 0),
                    self.bug_kind(),
                    self.bug_category(),
                    self.risk_level(),
                    self.cwe_ids(),
                    self.swc_ids(),
                ));
                // Only report once per function.
                return;
            }

            // Recurse into compound statements.
            match stmt {
                Stmt::If(s) => {
                    let mut branch_seen = *seen_ext_call;
                    self.check_stmts(
                        &s.then_body,
                        storage_vars,
                        &mut branch_seen,
                        bugs,
                        contract_name,
                        func_name,
                    );
                    if let Some(else_body) = &s.else_body {
                        let mut else_seen = *seen_ext_call;
                        self.check_stmts(
                            else_body,
                            storage_vars,
                            &mut else_seen,
                            bugs,
                            contract_name,
                            func_name,
                        );
                        branch_seen = branch_seen || else_seen;
                    }
                    *seen_ext_call = branch_seen;
                }
                Stmt::While(s) => {
                    self.check_stmts(
                        &s.body,
                        storage_vars,
                        seen_ext_call,
                        bugs,
                        contract_name,
                        func_name,
                    );
                }
                Stmt::For(s) => {
                    self.check_stmts(
                        &s.body,
                        storage_vars,
                        seen_ext_call,
                        bugs,
                        contract_name,
                        func_name,
                    );
                }
                Stmt::Block(inner) => {
                    self.check_stmts(
                        inner,
                        storage_vars,
                        seen_ext_call,
                        bugs,
                        contract_name,
                        func_name,
                    );
                }
                _ => {}
            }
        }
    }

    /// Check if a single statement contains an external call expression.
    fn stmt_has_external_call(&self, stmt: &Stmt) -> bool {
        let mut found = false;
        structural::walk_function_calls(std::slice::from_ref(stmt), &mut |call| {
            if structural::is_evm_external_call(call) {
                found = true;
            }
        });
        found
    }

    /// Check if a single statement writes to storage.
    fn stmt_has_storage_write(&self, stmt: &Stmt, storage_vars: &[String]) -> bool {
        match stmt {
            Stmt::Assign(a) => structural::expr_references_storage(&a.lhs, storage_vars),
            Stmt::AugAssign(a) => structural::expr_references_storage(&a.lhs, storage_vars),
            _ => false,
        }
    }
}

impl Pass for CeiViolationSirDetector {
    fn name(&self) -> &'static str {
        "CEI Pattern Violation"
    }

    fn description(&self) -> &'static str {
        "Detects violations of the Checks-Effects-Interactions pattern using SIR tree walking"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for CeiViolationSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::CeiViolation
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    let storage_vars = structural::storage_names(contract);
                    if storage_vars.is_empty() {
                        continue;
                    }

                    for member in &contract.members {
                        if let MemberDecl::Function(func) = member {
                            // Skip functions with reentrancy guard
                            if structural::has_reentrancy_guard(func) {
                                continue;
                            }

                            if let Some(body) = &func.body {
                                let mut seen_ext_call = false;
                                self.check_stmts(
                                    body,
                                    &storage_vars,
                                    &mut seen_ext_call,
                                    &mut bugs,
                                    &contract.name,
                                    &func.name,
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::Reentrancy
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
        let detector = CeiViolationSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::CeiViolation);
        assert_eq!(detector.risk_level(), RiskLevel::High);
        assert_eq!(detector.swc_ids(), vec![107]);
    }
}
