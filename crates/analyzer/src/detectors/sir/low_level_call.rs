//! Low-Level Call Detector (SIR structural)
//!
//! Detects usage of low-level calls (`.call`, `.delegatecall`, `.staticcall`)
//! by walking the SIR tree for EvmExpr dialect nodes and FieldAccess patterns.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, DialectExpr, FieldAccessExpr, FunctionDecl};
use std::any::TypeId;

/// SIR structural detector for low-level calls.
#[derive(Debug, Default)]
pub struct LowLevelCallSirDetector;

impl LowLevelCallSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for LowLevelCallSirDetector {
    fn name(&self) -> &'static str {
        "Low-Level Calls"
    }

    fn description(&self) -> &'static str {
        "Detects usage of low-level EVM calls on SIR."
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

impl BugDetectionPass for LowLevelCallSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::LowLevelCall
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        struct Visitor<'b> {
            detector: &'b LowLevelCallSirDetector,
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

            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                let call_kind = match d {
                    DialectExpr::Evm(EvmExpr::LowLevelCall { .. }) => Some("call"),
                    DialectExpr::Evm(EvmExpr::RawCall { .. }) => Some("raw_call"),
                    DialectExpr::Evm(EvmExpr::Send { .. }) => Some("send"),
                    DialectExpr::Evm(EvmExpr::Delegatecall { .. }) => Some("delegatecall"),
                    _ => None,
                };
                if let Some(kind) = call_kind {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Low-level '{}' detected in '{}.{}'. \
                             Consider using higher-level function calls.",
                            kind, self.contract_name, self.func_name
                        )),
                        Loc::new(0, 0, 0, 0),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                    ));
                }
            }

            fn visit_field_access_expr(&mut self, fa: &'a FieldAccessExpr) {
                let field = fa.field.as_str();
                if matches!(field, "call" | "staticcall") {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Low-level '{}' detected in '{}.{}'. \
                             Consider using higher-level function calls.",
                            field, self.contract_name, self.func_name
                        )),
                        Loc::new(0, 0, 0, 0),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                    ));
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
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::UncheckedLowLevelCalls
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn recommendation(&self) -> &'static str {
        "Avoid low-level calls. If necessary, ensure proper checks and handling."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://docs.soliditylang.org/en/latest/units-and-global-variables.html#members-of-address-types",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_level_call_sir_detector() {
        let detector = LowLevelCallSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::LowLevelCall);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
