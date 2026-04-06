//! Low-Level Call Detector
//!
//! Detects usage of low-level calls (`.call`, `.delegatecall`, `.staticcall`).

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, DialectExpr, FieldAccessExpr, FunctionDecl, Module};

/// Scan detector for low-level calls.
#[derive(Debug, Default)]
pub struct LowLevelCallDetector;

impl LowLevelCallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for LowLevelCallDetector {
    fn id(&self) -> &'static str {
        "low-level-call"
    }

    fn name(&self) -> &'static str {
        "Low-Level Calls"
    }

    fn description(&self) -> &'static str {
        "Detects usage of low-level EVM calls on SIR."
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

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Function
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn recommendation(&self) -> &'static str {
        "Avoid low-level `.call()`, `.delegatecall()`, and `.staticcall()` \
         where possible. Use Solidity interfaces or OpenZeppelin's `Address` \
         library for safer external calls. Always check the return value."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://docs.soliditylang.org/en/latest/units-and-global-variables.html#members-of-address-types",
        ]
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        struct Visitor<'b> {
            detector: &'b LowLevelCallDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                let call_info = match d {
                    DialectExpr::Evm(EvmExpr::LowLevelCall(e)) => Some(("call", e.loc.clone())),
                    DialectExpr::Evm(EvmExpr::RawCall(e)) => Some(("raw_call", e.loc.clone())),
                    DialectExpr::Evm(EvmExpr::Send(e)) => Some(("send", e.loc.clone())),
                    DialectExpr::Evm(EvmExpr::Delegatecall(e)) => {
                        Some(("delegatecall", e.loc.clone()))
                    }
                    _ => None,
                };
                if let Some((kind, loc)) = call_info {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Low-level '{}' detected in '{}.{}'. \
                             Consider using higher-level function calls.",
                            kind, self.contract_name, self.func_name
                        )),
                        loc,
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
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
                        fa.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }
                visit::default::visit_field_access_expr(self, fa);
            }
        }

        let mut visitor = Visitor {
            detector: self,
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
    fn test_low_level_call_detector() {
        let detector = LowLevelCallDetector::new();
        assert_eq!(detector.id(), "low-level-call");
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
