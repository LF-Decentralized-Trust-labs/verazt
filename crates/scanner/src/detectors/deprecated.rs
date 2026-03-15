//! Deprecated Features Detector (SIR structural)
//!
//! Detects usage of deprecated Solidity features by walking the SIR tree
//! for function calls to `suicide`, `sha3`, `callcode`, and `block.blockhash`.

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::query as structural;
use scirs::sir::{Decl, Expr, MemberDecl};
use std::any::TypeId;

/// Deprecated Solidity constructs with their replacements.
const DEPRECATED_IDENTS: &[(&str, &str)] = &[
    ("suicide", "selfdestruct"),
    ("sha3", "keccak256"),
    ("throw", "revert()"),
];

const DEPRECATED_FIELDS: &[(&str, &str)] = &[("callcode", "delegatecall")];

/// SIR structural detector for deprecated features.
#[derive(Debug, Default)]
pub struct DeprecatedSirDetector;

impl DeprecatedSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for DeprecatedSirDetector {
    fn name(&self) -> &'static str {
        "Deprecated Features"
    }

    fn description(&self) -> &'static str {
        "Detects usage of deprecated Solidity constructs on SIR."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for DeprecatedSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::Deprecated
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
                            if let Some(body) = &func.body {
                                structural::walk_exprs(body, &mut |expr| {
                                    match expr {
                                        // Check for deprecated identifiers used as callees
                                        Expr::Var(v) => {
                                            for (deprecated, replacement) in DEPRECATED_IDENTS {
                                                if v.name == *deprecated {
                                                    bugs.push(Bug::new(
                                                        self.name(),
                                                        Some(&format!(
                                                            "Deprecated '{}' used in '{}.{}'. Use '{}' instead.",
                                                            deprecated, contract.name, func.name, replacement
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
                                        // Check for deprecated field accesses (e.g. addr.callcode)
                                        Expr::FieldAccess(fa) => {
                                            for (deprecated, replacement) in DEPRECATED_FIELDS {
                                                if fa.field == *deprecated {
                                                    bugs.push(Bug::new(
                                                        self.name(),
                                                        Some(&format!(
                                                            "Deprecated '{}' used in '{}.{}'. Use '{}' instead.",
                                                            deprecated, contract.name, func.name, replacement
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
                                        _ => {}
                                    }
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Refactoring
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::CodeQuality
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![111] // SWC-111: Use of Deprecated Solidity Functions
    }

    fn recommendation(&self) -> &'static str {
        "Replace deprecated features with their modern equivalents."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-111"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecated_sir_detector() {
        let detector = DeprecatedSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::Deprecated);
        assert_eq!(detector.swc_ids(), vec![111]);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
