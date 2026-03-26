//! Visibility Detector (SIR structural)
//!
//! Detects missing visibility specifiers on SIR `FunctionDecl` nodes by
//! checking for absence of `#sir.visibility` attribute.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::attrs::sir_attrs;
use scirs::sir::{Decl, MemberDecl};
use std::any::TypeId;

/// SIR structural detector for visibility issues.
///
/// Detects functions without explicit visibility specifiers.
#[derive(Debug, Default)]
pub struct VisibilitySirDetector;

impl VisibilitySirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for VisibilitySirDetector {
    fn name(&self) -> &'static str {
        "Visibility Issues"
    }

    fn description(&self) -> &'static str {
        "Detects missing function visibility specifiers on SIR."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for VisibilitySirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::Visibility
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
                        if let MemberDecl::Function(func) = member {
                            // Skip constructors, fallback, receive
                            if func.name.is_empty()
                                || func.name == "constructor"
                                || func.name == "fallback"
                                || func.name == "receive"
                            {
                                continue;
                            }

                            // Check for #sir.visibility attr
                            let has_visibility = func
                                .attrs
                                .iter()
                                .any(|a| a.namespace == "sir" && a.key == sir_attrs::VISIBILITY);

                            if !has_visibility {
                                bugs.push(Bug::new(
                                    self.name(),
                                    Some(&format!(
                                        "Function '{}' in contract '{}' has no explicit \
                                         visibility specifier. Consider adding 'public', \
                                         'external', 'internal', or 'private'.",
                                        func.name, contract.name,
                                    )),
                                    Loc::new(0, 0, 0, 0),
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
        BugCategory::AccessControl
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![710] // CWE-710: Improper Adherence to Coding Standards
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![100, 108]
    }

    fn recommendation(&self) -> &'static str {
        "Explicitly define visibility for all functions and state variables."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-100",
            "https://swcregistry.io/docs/SWC-108",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_sir_detector() {
        let detector = VisibilitySirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::Visibility);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
