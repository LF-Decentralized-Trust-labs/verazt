//! Low-Level Call Detector
//!
//! Detects usage of low-level calls (`.call`, `.delegatecall`, `.staticcall`).

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, DialectExpr, FieldAccessExpr, FunctionDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::UncheckedLowLevelCalls,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[],
    description: "Detects usage of low-level EVM calls on SIR.",
    id: DetectorId::LowLevelCall,
    name: "Low-Level Calls",
    recommendation: "Avoid low-level `.call()`, `.delegatecall()`, and `.staticcall()` \
         where possible. Use Solidity interfaces or OpenZeppelin's `Address` \
         library for safer external calls. Always check the return value.",
    references: &[
        "https://docs.soliditylang.org/en/latest/units-and-global-variables.html#members-of-address-types",
    ],
    risk_level: RiskLevel::Medium,
    swc_ids: &[],
    target: Target::Evm,
};

/// Scan detector for low-level calls.
#[derive(Debug, Default)]
pub struct LowLevelCallDetector;

impl LowLevelCallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for LowLevelCallDetector {
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
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "Low-level '{}' detected in '{}.{}'. \
                             Consider using higher-level function calls.",
                            kind, self.contract_name, self.func_name
                        )),
                        loc,
                    ));
                }
                visit::default::visit_dialect_expr(self, d);
            }

            fn visit_field_access_expr(&mut self, fa: &'a FieldAccessExpr) {
                let field = fa.field.as_str();
                if matches!(field, "call" | "staticcall") {
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "Low-level '{}' detected in '{}.{}'. \
                             Consider using higher-level function calls.",
                            field, self.contract_name, self.func_name
                        )),
                        fa.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    ));
                }
                visit::default::visit_field_access_expr(self, fa);
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
    fn test_low_level_call_detector() {
        let detector = LowLevelCallDetector::new();
        assert_eq!(detector.meta().id, DetectorId::LowLevelCall);
        assert_eq!(detector.meta().risk_level, RiskLevel::Medium);
    }
}
