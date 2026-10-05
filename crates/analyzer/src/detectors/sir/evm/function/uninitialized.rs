//! Uninitialized Storage Detector
//!
//! Detects local storage references declared without an initializer. Before
//! Solidity 0.5, such a local struct or array points at storage slot 0, so
//! writing to it overwrites the first state variables.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, FunctionDecl, LocalVarStmt, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::Other,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[824],
    description: "Detects local storage references declared without an initializer, \
         which point at storage slot 0",
    id: DetectorId::UninitializedStorage,
    name: "Uninitialized Storage",
    recommendation: "Initialize all storage variables explicitly. For local variables with storage \
         location, assign a reference to a state variable before use.",
    references: &["https://swcregistry.io/docs/SWC-109"],
    risk_level: RiskLevel::High,
    swc_ids: &[109],
    target: Target::Evm,
};

/// Scan detector for uninitialized local storage references.
#[derive(Debug, Default)]
pub struct UninitializedDetector;

impl UninitializedDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for UninitializedDetector {
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

        struct Visitor<'b> {
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a> Visit<'a> for Visitor<'_> {
            fn visit_local_var_stmt(&mut self, stmt: &'a LocalVarStmt) {
                if stmt.init.is_none() {
                    for decl in stmt.vars.iter().flatten().filter(|d| d.is_storage_ref) {
                        self.bugs.push(META.bug(
                            Some(&format!(
                                "Local storage variable '{}' in '{}.{}' is not \
                                 initialized, so it points at storage slot 0.",
                                decl.name, self.contract_name, self.func_name,
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
        };
        visitor.visit_function_decl(func);

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninitialized_detector() {
        let detector = UninitializedDetector::new();
        assert_eq!(detector.meta().id, DetectorId::UninitializedStorage);
        assert_eq!(detector.meta().risk_level, RiskLevel::High);
    }
}
