//! Timestamp Dependence Detector
//!
//! Detects usage of `block.timestamp` which can be manipulated by miners.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::Visit;
use scirs::sir::{ContractDecl, DialectExpr, FunctionDecl, Module};

/// Scan detector for timestamp dependence.
#[derive(Debug, Default)]
pub struct TimestampDependenceDetector;

impl TimestampDependenceDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for TimestampDependenceDetector {
    fn id(&self) -> &'static str {
        "timestamp-dependence"
    }

    fn name(&self) -> &'static str {
        "Timestamp Dependence"
    }

    fn description(&self) -> &'static str {
        "Detects dangerous reliance on block.timestamp via SIR."
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::TimeManipulation
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
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
        vec![829]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![116]
    }

    fn recommendation(&self) -> &'static str {
        "Avoid using `block.timestamp` for critical logic. Miners/validators \
         can manipulate it by ~15 seconds. For time-sensitive logic, use \
         block numbers or an oracle. Never use it as a source of randomness."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-116"]
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        struct Visitor<'b> {
            detector: &'b TimestampDependenceDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                if let DialectExpr::Evm(EvmExpr::Timestamp(e)) = d {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Usage of block.timestamp in '{}.{}'. \
                             Miners can manipulate this value within \
                             a range of ~15 seconds.",
                            self.contract_name, self.func_name
                        )),
                        e.loc.clone(),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }
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
    fn test_timestamp_dependence_detector() {
        let detector = TimestampDependenceDetector::new();
        assert_eq!(detector.id(), "timestamp-dependence");
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
