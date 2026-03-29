//! Uninitialized Storage Detector (SIR structural)
//!
//! Detects uninitialized storage variables by walking SIR contract
//! declarations.
//!
//! This detector finds:
//! - State variables of mapping/array type that are not explicitly initialized

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::EvmStorageExt;
use scirs::sir::{Decl, MemberDecl};
use std::any::TypeId;

/// SIR structural detector for uninitialized storage variables.
#[derive(Debug, Default)]
pub struct UninitializedSirDetector;

impl UninitializedSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for UninitializedSirDetector {
    fn name(&self) -> &'static str {
        "Uninitialized Storage"
    }

    fn description(&self) -> &'static str {
        "Detects uninitialized storage variables using SIR tree walking"
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

impl BugDetectionPass for UninitializedSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::UninitializedStorage
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    for member in &contract.members {
                        if let MemberDecl::Storage(storage) = member {
                            // Skip constant/immutable storage variables
                            if storage.is_constant_storage() {
                                continue;
                            }

                            // Check if the type is a mapping or array
                            // (these are the most dangerous when
                            // uninitialized)
                            let ty_str = storage.ty.to_string().to_lowercase();
                            let is_complex_type =
                                ty_str.contains("mapping") || ty_str.contains("[]");

                            if is_complex_type && storage.init.is_none() {
                                bugs.push(Bug::new(
                                    self.name(),
                                    Some(&format!(
                                        "State variable '{}' in contract '{}' is not \
                                         initialized. Consider initializing it explicitly.",
                                        storage.name, contract.name,
                                    )),
                                    storage.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                                    self.bug_kind(),
                                    self.bug_category(),
                                    self.risk_level(),
                                    self.cwe_ids(),
                                    self.swc_ids(),
                                ));
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
        BugCategory::Other
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![824] // CWE-824: Access of Uninitialized Pointer
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![109] // SWC-109: Uninitialized Storage Pointer
    }

    fn recommendation(&self) -> &'static str {
        "Initialize all storage variables explicitly. For local variables with storage \
         location, assign a reference to a state variable before use."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-109"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninitialized_detector() {
        let detector = UninitializedSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::UninitializedStorage);
        assert_eq!(detector.swc_ids(), vec![109]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
