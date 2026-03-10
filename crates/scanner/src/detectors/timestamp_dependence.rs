//! Timestamp Dependence Detector (SIR structural)
//!
//! Detects usage of `block.timestamp` by walking the SIR tree for
//! `EvmExpr::Timestamp` dialect nodes.

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use mlir::sir::dialect::evm::EvmExpr;
use mlir::sir::utils::query as structural;
use mlir::sir::{Decl, DialectExpr, Expr, MemberDecl};
use std::any::TypeId;

/// SIR structural detector for timestamp dependence.
///
/// Detects usage of `block.timestamp` which can be manipulated by miners.
#[derive(Debug, Default)]
pub struct TimestampDependenceSirDetector;

impl TimestampDependenceSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for TimestampDependenceSirDetector {
    fn name(&self) -> &'static str {
        "Timestamp Dependence"
    }

    fn description(&self) -> &'static str {
        "Detects dangerous reliance on block.timestamp via SIR."
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

impl BugDetectionPass for TimestampDependenceSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::TimestampDependence
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
                                    if matches!(
                                        expr,
                                        Expr::Dialect(DialectExpr::Evm(EvmExpr::Timestamp))
                                    ) {
                                        bugs.push(Bug::new(
                                            self.name(),
                                            Some(&format!(
                                                "Usage of block.timestamp in '{}.{}'. \
                                                 Miners can manipulate this value within \
                                                 a range of ~15 seconds.",
                                                contract.name, func.name
                                            )),
                                            Loc::new(0, 0, 0, 0),
                                            self.bug_kind(),
                                            self.bug_category(),
                                            self.risk_level(),
                                            self.cwe_ids(),
                                            self.swc_ids(),
                                        ));
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
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::TimeManipulation
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![829]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![116] // SWC-116: Block values as a proxy for time
    }

    fn recommendation(&self) -> &'static str {
        "Avoid using block.timestamp for critical logic or randomness."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-116"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_dependence_sir_detector() {
        let detector = TimestampDependenceSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::TimestampDependence);
        assert_eq!(detector.swc_ids(), vec![116]);
    }
}
