//! Variable Shadowing Detector
//!
//! Detects local variable declarations that shadow storage variables.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, FunctionDecl, LocalVarStmt, Module};
use std::collections::HashSet;

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::CodeQuality,
    bug_kind: BugKind::Refactoring,
    confidence: ConfidenceLevel::High,
    cwe_ids: &[],
    description: "Detects variable shadowing that can cause confusion.",
    id: DetectorId::Shadowing,
    name: "Variable Shadowing",
    recommendation: "Rename the local variable to avoid shadowing the inherited state \
         variable. Shadowing can cause unintended reads/writes to the wrong \
         variable, leading to subtle logic bugs.",
    references: &["https://swcregistry.io/docs/SWC-119"],
    risk_level: RiskLevel::Low,
    swc_ids: &[119],
    target: Target::Evm,
};

/// Scan detector for variable shadowing.
#[derive(Debug, Default)]
pub struct ShadowingDetector;

impl ShadowingDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for ShadowingDetector {
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

        let state_vars: HashSet<String> = contract.storage_names().into_iter().collect();
        if state_vars.is_empty() {
            return bugs;
        }

        // Check parameters for shadowing
        for param in &func.params {
            if state_vars.contains(&param.name) {
                bugs.push(META.bug(
                    Some(&format!(
                        "Parameter '{}' in '{}.{}' shadows a state variable.",
                        param.name, contract.name, func.name,
                    )),
                    func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                ));
            }
        }

        // Check local variable declarations
        struct Visitor<'b> {
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
            state_vars: HashSet<String>,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_local_var_stmt(&mut self, stmt: &'a LocalVarStmt) {
                for var in stmt.vars.iter().flatten() {
                    if self.state_vars.contains(&var.name) {
                        self.bugs.push(META.bug(
                            Some(&format!(
                                "Local variable '{}' in '{}.{}' shadows a state variable.",
                                var.name, self.contract_name, self.func_name,
                            )),
                            stmt.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        ));
                    }
                }
                visit::default::visit_local_var_stmt(self, stmt);
            }
        }

        let mut visitor = Visitor {
            bugs: &mut bugs,
            contract_name: contract.name.clone(),
            func_name: func.name.clone(),
            state_vars,
        };
        visitor.visit_function_decl(func);

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadowing_detector() {
        let detector = ShadowingDetector::new();
        assert_eq!(detector.meta().id, DetectorId::Shadowing);
        assert_eq!(detector.meta().risk_level, RiskLevel::Low);
    }
}
