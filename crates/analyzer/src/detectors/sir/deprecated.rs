//! Deprecated Features Detector (SIR structural)
//!
//! Detects usage of deprecated Solidity features by walking the SIR tree
//! for function calls to `suicide`, `sha3`, `callcode`, and `block.blockhash`.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, FieldAccessExpr, FunctionDecl, VarExpr};
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

        struct Visitor<'b> {
            detector: &'b DeprecatedSirDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_contract_decl(&mut self, contract: &'a ContractDecl) {
                self.contract_name = contract.name.clone();
                visit::default::visit_contract_decl(self, contract);
            }

            fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
                self.func_name = func.name.clone();
                visit::default::visit_function_decl(self, func);
            }

            fn visit_var_expr(&mut self, v: &'a VarExpr) {
                for (deprecated, replacement) in DEPRECATED_IDENTS {
                    if v.name == *deprecated {
                        self.bugs.push(Bug::new(
                            self.detector.name(),
                            Some(&format!(
                                "Deprecated '{}' used in '{}.{}'. Use '{}' instead.",
                                deprecated, self.contract_name, self.func_name, replacement
                            )),
                            v.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                            self.detector.bug_kind(),
                            self.detector.bug_category(),
                            self.detector.risk_level(),
                            self.detector.cwe_ids(),
                            self.detector.swc_ids(),
                        ));
                    }
                }
            }

            fn visit_field_access_expr(&mut self, fa: &'a FieldAccessExpr) {
                for (deprecated, replacement) in DEPRECATED_FIELDS {
                    if fa.field == *deprecated {
                        self.bugs.push(Bug::new(
                            self.detector.name(),
                            Some(&format!(
                                "Deprecated '{}' used in '{}.{}'. Use '{}' instead.",
                                deprecated, self.contract_name, self.func_name, replacement
                            )),
                            fa.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                            self.detector.bug_kind(),
                            self.detector.bug_category(),
                            self.detector.risk_level(),
                            self.detector.cwe_ids(),
                            self.detector.swc_ids(),
                        ));
                    }
                }
                visit::default::visit_field_access_expr(self, fa);
            }
        }

        let mut visitor = Visitor {
            detector: self,
            bugs: &mut bugs,
            contract_name: String::new(),
            func_name: String::new(),
        };
        visitor.visit_modules(context.ir_units());

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
