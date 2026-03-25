//! Low-Level Call Detector (SIR structural)
//!
//! Detects usage of low-level calls (`.call`, `.delegatecall`, `.staticcall`)
//! by walking the SIR tree for EvmExpr dialect nodes and FieldAccess patterns.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass::meta::PassLevel;
use crate::analysis::pass::meta::PassRepresentation;
use crate::detectors::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::query as structural;
use scirs::sir::{Decl, DialectExpr, Expr, MemberDecl};
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

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    for member in &contract.members {
                        if let MemberDecl::Function(func) = member {
                            if let Some(body) = &func.body {
                                structural::walk_exprs(body, &mut |expr| {
                                    let call_kind = match expr {
                                        // EVM dialect low-level call nodes
                                        Expr::Dialect(DialectExpr::Evm(
                                            EvmExpr::LowLevelCall { .. },
                                        )) => Some("call"),
                                        Expr::Dialect(DialectExpr::Evm(EvmExpr::RawCall {
                                            ..
                                        })) => Some("raw_call"),
                                        Expr::Dialect(DialectExpr::Evm(EvmExpr::Send {
                                            ..
                                        })) => Some("send"),
                                        Expr::Dialect(DialectExpr::Evm(
                                            EvmExpr::Delegatecall { .. },
                                        )) => Some("delegatecall"),
                                        // Field-access fallback: addr.call, addr.staticcall
                                        Expr::FieldAccess(fa) => {
                                            let field = fa.field.as_str();
                                            if matches!(field, "call" | "staticcall") {
                                                Some(field)
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    };
                                    if let Some(kind) = call_kind {
                                        bugs.push(Bug::new(
                                            self.name(),
                                            Some(&format!(
                                                "Low-level '{}' detected in '{}.{}'. \
                                                 Consider using higher-level function calls.",
                                                kind, contract.name, func.name
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
