//! Reentrancy Detector
//!
//! Detects potential reentrancy vulnerabilities by finding storage writes
//! after external calls.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::ContractDecl;
use scirs::sir::dialect::{EvmCallExt, EvmFunctionExt};
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{CallExpr, FunctionDecl, Module, Stmt};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::Reentrancy,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[841],
    description: "Detects potential reentrancy vulnerabilities using SIR tree walking. \
         Finds state modifications after external calls.",
    id: DetectorId::Reentrancy,
    name: "Reentrancy",
    recommendation: "Follow the Checks-Effects-Interactions pattern: perform all state changes \
         before making external calls. Consider using a reentrancy guard \
         (e.g., OpenZeppelin's ReentrancyGuard).",
    references: &[
        "https://swcregistry.io/docs/SWC-107",
        "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/",
    ],
    risk_level: RiskLevel::Critical,
    swc_ids: &[107],
    target: Target::Evm,
};

/// Scan detector for reentrancy vulnerabilities.
#[derive(Debug, Default)]
pub struct ReentrancyDetector;

impl ReentrancyDetector {
    pub fn new() -> Self {
        Self
    }

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
            if !*seen_ext_call && self.stmt_has_external_call(stmt) {
                *seen_ext_call = true;
            }

            if *seen_ext_call && self.stmt_has_storage_write(stmt, storage_vars) {
                bugs.push(META.bug(
                    Some(&format!(
                        "Potential reentrancy in '{}.{}': state modification \
                         after external call.",
                        contract_name, func_name,
                    )),
                    stmt.span().cloned().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                ));
                return;
            }

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

    fn stmt_has_external_call(&self, stmt: &Stmt) -> bool {
        struct CallFinder {
            found: bool,
        }
        impl<'a> Visit<'a> for CallFinder {
            fn visit_call_expr(&mut self, call: &'a CallExpr) {
                if call.is_evm_external_call() {
                    self.found = true;
                }
                if !self.found {
                    visit::default::visit_call_expr(self, call);
                }
            }
        }
        let mut finder = CallFinder { found: false };
        finder.visit_stmt(stmt);
        finder.found
    }

    fn stmt_has_storage_write(&self, stmt: &Stmt, storage_vars: &[String]) -> bool {
        match stmt {
            Stmt::Assign(a) => ContractDecl::expr_references_storage(&a.lhs, storage_vars),
            Stmt::AugAssign(a) => ContractDecl::expr_references_storage(&a.lhs, storage_vars),
            _ => false,
        }
    }
}

impl ScanDetector for ReentrancyDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Function
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        if func.has_reentrancy_guard() {
            return bugs;
        }

        let storage_vars = contract.storage_names();
        if storage_vars.is_empty() {
            return bugs;
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

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reentrancy_detector() {
        let detector = ReentrancyDetector::new();
        assert_eq!(detector.meta().id, DetectorId::Reentrancy);
        assert_eq!(detector.meta().risk_level, RiskLevel::Critical);
    }
}
