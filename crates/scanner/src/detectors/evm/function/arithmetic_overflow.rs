//! Integer Overflow/Underflow Detector
//!
//! Detects arithmetic operations with wrapping semantics (Solidity <0.8
//! without SafeMath) by walking `BinOpExpr` and `AugAssignStmt` nodes.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::exprs::{BinOp, Expr, OverflowSemantics};
use scirs::sir::lits::{Lit, Num};
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{AugAssignStmt, BinOpExpr, ContractDecl, FunctionDecl, Module};

/// Scan detector for integer overflow/underflow.
#[derive(Debug, Default)]
pub struct ArithmeticOverflowDetector;

impl ArithmeticOverflowDetector {
    pub fn new() -> Self {
        Self
    }
}

fn is_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(_))
}

fn is_trivial_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(Lit::Bool(_)) => true,
        Expr::Lit(Lit::Num(n)) => match &n.value {
            Num::Int(int_num) => {
                use num_traits::Zero;
                int_num.value.is_zero() || int_num.value == 1u64.into()
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_arithmetic_op(op: BinOp) -> bool {
    matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow)
}

impl ScanDetector for ArithmeticOverflowDetector {
    fn id(&self) -> &'static str {
        "arithmetic-overflow"
    }

    fn name(&self) -> &'static str {
        "Integer Overflow/Underflow"
    }

    fn description(&self) -> &'static str {
        "Detects arithmetic operations with wrapping overflow semantics (Solidity <0.8)."
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::Arithmetic
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
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
        vec![190, 191]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![101]
    }

    fn recommendation(&self) -> &'static str {
        "Use Solidity ≥0.8.0 which has built-in overflow checks, or use \
         OpenZeppelin's SafeMath library for earlier versions. Avoid \
         unchecked arithmetic blocks unless overflow is intentional."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-101",
            "https://docs.soliditylang.org/en/latest/080-breaking-changes.html",
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
            detector: &'b ArithmeticOverflowDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl Visitor<'_> {
            fn both_literals(&self, lhs: &Expr, rhs: &Expr) -> bool {
                is_literal(lhs) && is_literal(rhs)
            }

            fn has_trivial_operand(&self, lhs: &Expr, rhs: &Expr) -> bool {
                is_trivial_literal(lhs) || is_trivial_literal(rhs)
            }
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_binop_expr(&mut self, expr: &'a BinOpExpr) {
                if is_arithmetic_op(expr.op)
                    && expr.overflow == OverflowSemantics::Wrapping
                    && !self.both_literals(&expr.lhs, &expr.rhs)
                    && !self.has_trivial_operand(&expr.lhs, &expr.rhs)
                {
                    let op_str = match expr.op {
                        BinOp::Add => "addition",
                        BinOp::Sub => "subtraction",
                        BinOp::Mul => "multiplication",
                        BinOp::Pow => "exponentiation",
                        _ => "arithmetic",
                    };
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Potential integer overflow/underflow: unchecked {} in \
                             '{}.{}'. Solidity <0.8 uses wrapping arithmetic.",
                            op_str, self.contract_name, self.func_name
                        )),
                        expr.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }
                visit::default::visit_binop_expr(self, expr);
            }

            fn visit_aug_assign_stmt(&mut self, stmt: &'a AugAssignStmt) {
                if is_arithmetic_op(stmt.op) && !is_trivial_literal(&stmt.rhs) {
                    let op_str = match stmt.op {
                        BinOp::Add => "+=",
                        BinOp::Sub => "-=",
                        BinOp::Mul => "*=",
                        BinOp::Pow => "**=",
                        _ => "op=",
                    };
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Potential integer overflow/underflow: unchecked '{}' \
                             in '{}.{}'.",
                            op_str, self.contract_name, self.func_name
                        )),
                        stmt.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }
                visit::default::visit_aug_assign_stmt(self, stmt);
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
    fn test_arithmetic_overflow_detector() {
        let detector = ArithmeticOverflowDetector::new();
        assert_eq!(detector.id(), "arithmetic-overflow");
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
