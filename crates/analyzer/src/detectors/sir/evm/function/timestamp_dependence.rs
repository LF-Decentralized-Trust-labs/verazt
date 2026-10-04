//! Timestamp Dependence Detector
//!
//! Detects usage of `block.timestamp` which can be manipulated by miners.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::Visit;
use scirs::sir::{ContractDecl, DialectExpr, FunctionDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::TimeManipulation,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[829],
    description: "Detects dangerous reliance on block.timestamp via SIR.",
    id: DetectorId::TimestampDependence,
    name: "Timestamp Dependence",
    recommendation: "Avoid using `block.timestamp` for critical logic. Miners/validators \
         can manipulate it by ~15 seconds. For time-sensitive logic, use \
         block numbers or an oracle. Never use it as a source of randomness.",
    references: &["https://swcregistry.io/docs/SWC-116"],
    risk_level: RiskLevel::Low,
    swc_ids: &[116],
    target: Target::Evm,
};

/// Scan detector for timestamp dependence.
#[derive(Debug, Default)]
pub struct TimestampDependenceDetector;

impl TimestampDependenceDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for TimestampDependenceDetector {
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
                if let DialectExpr::Evm(EvmExpr::Timestamp(e)) = d {
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "Usage of block.timestamp in '{}.{}'. \
                             Miners can manipulate this value within \
                             a range of ~15 seconds.",
                            self.contract_name, self.func_name
                        )),
                        e.loc.clone(),
                    ));
                }
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
    fn test_timestamp_dependence_detector() {
        let detector = TimestampDependenceDetector::new();
        assert_eq!(detector.meta().id, DetectorId::TimestampDependence);
        assert_eq!(detector.meta().risk_level, RiskLevel::Low);
    }
}
