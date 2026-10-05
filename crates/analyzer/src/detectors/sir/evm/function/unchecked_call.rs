//! Unchecked Call Return Detector
//!
//! Detects low-level calls whose return values are not checked.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::EvmExprExt;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, DialectExpr, Expr, ExprStmt, FunctionDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::UncheckedLowLevelCalls,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::High,
    cwe_ids: &[252],
    description: "Detects low-level calls whose return values are not checked using SIR tree walking",
    id: DetectorId::UncheckedCall,
    name: "Unchecked Call Return",
    recommendation: "Ensure the return value of the low-level call is checked. \
         Use `require(success)` or handle the failure case explicitly.",
    references: &["https://swcregistry.io/docs/SWC-104"],
    risk_level: RiskLevel::Medium,
    swc_ids: &[104],
    target: Target::Evm,
};

/// Returns true if `expr`, evaluated as a statement, discards the success
/// flag of a low-level call. `transfer` reverts on failure, so discarding
/// its result is safe.
fn discards_call_result(expr: &Expr) -> bool {
    let is_transfer = match expr {
        Expr::FunctionCall(call) => {
            matches!(&*call.callee, Expr::FieldAccess(fa) if fa.field == "transfer")
        }
        Expr::Dialect(DialectExpr::Evm(EvmExpr::Transfer(_))) => true,
        _ => false,
    };
    expr.is_evm_external_call() && !is_transfer
}

/// Scan detector for unchecked call return values.
#[derive(Debug, Default)]
pub struct UncheckedCallDetector;

impl UncheckedCallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for UncheckedCallDetector {
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
            fn visit_expr_stmt(&mut self, stmt: &'a ExprStmt) {
                if discards_call_result(&stmt.expr) {
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "Unchecked call return value in '{}.{}'. \
                             The return value of a low-level call is not checked.",
                            self.contract_name, self.func_name,
                        )),
                        stmt.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    ));
                }
                visit::default::visit_expr_stmt(self, stmt);
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
    fn test_unchecked_call_detector() {
        let detector = UncheckedCallDetector::new();
        assert_eq!(detector.meta().id, DetectorId::UncheckedCall);
        assert_eq!(detector.meta().risk_level, RiskLevel::Medium);
    }
}
