//! Short Address / Input Validation Detector
//!
//! Detects ERC-20 `transfer` and `transferFrom` functions that don't
//! validate `msg.data.length`.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::EvmFunctionExt;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::exprs::Expr;
use scirs::sir::stmts::Stmt;
use scirs::sir::{ContractDecl, DialectExpr, FunctionDecl, Module};

/// Scan detector for the short address attack.
#[derive(Debug, Default)]
pub struct ShortAddressDetector;

impl ShortAddressDetector {
    pub fn new() -> Self {
        Self
    }
}

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

fn references_msg_data(stmts: &[Stmt]) -> bool {
    stmts.iter().any(stmt_references_msg_data)
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

impl ScanDetector for ShortAddressDetector {
    fn id(&self) -> &'static str {
        "short-address"
    }

    fn name(&self) -> &'static str {
        "Short Address Attack"
    }

    fn description(&self) -> &'static str {
        "Detects ERC-20 transfer/transferFrom without msg.data.length check."
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

    fn confidence(&self) -> Confidence {
        Confidence::Low
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Function
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![20]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![130]
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

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        if !is_transfer_function(func) {
            return bugs;
        }

        if let Some(body) = &func.body {
            if !references_msg_data(body) {
                bugs.push(Bug::new(
                    self.name(),
                    Some(&format!(
                        "ERC-20 '{}' in '{}' does not validate \
                         msg.data.length. This may be vulnerable \
                         to a short address attack in older Solidity \
                         versions (<0.5.0).",
                        func.name, contract.name
                    )),
                    func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    self.bug_kind(),
                    self.bug_category(),
                    self.risk_level(),
                    self.cwe_ids(),
                    self.swc_ids(),
                    Some(self.recommendation()),
                ));
            }
        }

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_address_detector() {
        let detector = ShortAddressDetector::new();
        assert_eq!(detector.id(), "short-address");
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
