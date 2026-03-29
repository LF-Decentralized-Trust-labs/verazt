//! Short Address / Input Validation Detector (SIR structural)
//!
//! Detects ERC-20 `transfer` and `transferFrom` functions that don't
//! validate `msg.data.length`, making them vulnerable to the short
//! address attack (SWC-130).
//!
//! This is largely mitigated by modern Solidity compilers (≥0.5.0)
//! and is reported as informational.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::EvmFunctionExt;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::exprs::Expr;
use scirs::sir::stmts::Stmt;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, DialectExpr, FunctionDecl};
use std::any::TypeId;

/// SIR structural detector for the short address attack.
///
/// Flags ERC-20 `transfer`/`transferFrom` functions that don't
/// check `msg.data.length`.
#[derive(Debug, Default)]
pub struct ShortAddressSirDetector;

impl ShortAddressSirDetector {
    pub fn new() -> Self {
        Self
    }
}

/// Check if a function is an ERC-20 `transfer` or `transferFrom`.
fn is_transfer_function(func: &FunctionDecl) -> bool {
    if !func.is_public() {
        return false;
    }
    match func.name.as_str() {
        "transfer" => func.params.len() == 2,
        "transferFrom" => func.params.len() == 3,
        _ => false,
    }
}

/// Check if a function body references `msg.data` or `msg.data.length`.
fn references_msg_data(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        if stmt_references_msg_data(stmt) {
            return true;
        }
    }
    false
}

fn stmt_references_msg_data(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(es) => expr_references_msg_data(&es.expr),
        Stmt::Assert(a) => {
            expr_references_msg_data(&a.cond)
                || a.message.as_ref().is_some_and(expr_references_msg_data)
        }
        Stmt::If(s) => {
            expr_references_msg_data(&s.cond)
                || references_msg_data(&s.then_body)
                || s.else_body.as_ref().is_some_and(|e| references_msg_data(e))
        }
        Stmt::LocalVar(lv) => lv.init.as_ref().is_some_and(expr_references_msg_data),
        Stmt::Assign(a) => expr_references_msg_data(&a.lhs) || expr_references_msg_data(&a.rhs),
        Stmt::Block(stmts) => references_msg_data(stmts),
        _ => false,
    }
}

fn expr_references_msg_data(expr: &Expr) -> bool {
    match expr {
        Expr::Dialect(DialectExpr::Evm(EvmExpr::MsgData(_))) => true,
        Expr::FieldAccess(fa) => {
            // Check for msg.data.length pattern
            if fa.field == "length" {
                return expr_references_msg_data(&fa.base);
            }
            expr_references_msg_data(&fa.base)
        }
        Expr::BinOp(bin) => {
            expr_references_msg_data(&bin.lhs) || expr_references_msg_data(&bin.rhs)
        }
        Expr::FunctionCall(call) => {
            expr_references_msg_data(&call.callee)
                || call
                    .args
                    .exprs()
                    .iter()
                    .any(|a| expr_references_msg_data(a))
        }
        Expr::UnOp(un) => expr_references_msg_data(&un.operand),
        _ => false,
    }
}

impl Pass for ShortAddressSirDetector {
    fn name(&self) -> &'static str {
        "Short Address Attack"
    }

    fn description(&self) -> &'static str {
        "Detects ERC-20 transfer/transferFrom without msg.data.length check."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for ShortAddressSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::ShortAddress
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        struct Visitor<'b> {
            detector: &'b ShortAddressSirDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_contract_decl(&mut self, contract: &'a ContractDecl) {
                self.contract_name = contract.name.clone();
                visit::default::visit_contract_decl(self, contract);
            }

            fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
                if is_transfer_function(func) {
                    if let Some(body) = &func.body {
                        if !references_msg_data(body) {
                            self.bugs.push(Bug::new(
                                self.detector.name(),
                                Some(&format!(
                                    "ERC-20 '{}' in '{}' does not validate \
                                     msg.data.length. This may be vulnerable \
                                     to a short address attack in older Solidity \
                                     versions (<0.5.0).",
                                    func.name, self.contract_name
                                )),
                                func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
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

                visit::default::visit_function_decl(self, func);
            }
        }

        let mut visitor =
            Visitor { detector: self, bugs: &mut bugs, contract_name: String::new() };
        visitor.visit_modules(context.ir_units());

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::ShortAddresses
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Low
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![20] // CWE-20: Improper Input Validation
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![130] // SWC-130: Short Address/Parameter Attack
    }

    fn recommendation(&self) -> &'static str {
        "Add a check for msg.data.length in transfer/transferFrom functions, \
         e.g., `require(msg.data.length >= 68)`. Better yet, upgrade to \
         Solidity ≥0.5.0 which validates calldata length automatically."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-130",
            "https://blog.golemproject.net/how-to-find-10m-by-just-reading-blockchain/",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_address_sir_detector() {
        let detector = ShortAddressSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::ShortAddress);
        assert_eq!(detector.swc_ids(), vec![130]);
        assert_eq!(detector.cwe_ids(), vec![20]);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
        assert_eq!(detector.bug_category(), BugCategory::ShortAddresses);
    }
}
