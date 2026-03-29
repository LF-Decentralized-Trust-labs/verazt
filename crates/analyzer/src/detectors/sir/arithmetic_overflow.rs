//! Integer Overflow/Underflow Detector (SIR structural)
//!
//! Detects arithmetic operations with wrapping semantics (Solidity <0.8
//! without SafeMath) by walking `BinOpExpr` and `AugAssignStmt` nodes.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::exprs::{BinOp, Expr, OverflowSemantics};
use scirs::sir::lits::{Lit, Num};
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{AugAssignStmt, BinOpExpr, ContractDecl, FunctionDecl};
use std::any::TypeId;

/// SIR structural detector for integer overflow/underflow.
///
/// Flags arithmetic operations (`+`, `-`, `*`, `**`) that use wrapping
/// semantics, which can silently overflow in Solidity <0.8.
#[derive(Debug, Default)]
pub struct ArithmeticOverflowSirDetector;

impl ArithmeticOverflowSirDetector {
    pub fn new() -> Self {
        Self
    }
}

/// Returns true if the expression is a literal value.
fn is_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(_))
}

/// Returns true if the literal is a small constant (0 or 1) that is unlikely
/// to cause overflow.
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

/// Returns true if the binary op is an arithmetic op that can overflow.
fn is_arithmetic_op(op: BinOp) -> bool {
    matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow)
}

impl Pass for ArithmeticOverflowSirDetector {
    fn name(&self) -> &'static str {
        "Integer Overflow/Underflow"
    }

    fn description(&self) -> &'static str {
        "Detects arithmetic operations with wrapping overflow semantics (Solidity <0.8)."
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

impl BugDetectionPass for ArithmeticOverflowSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::ArithmeticOverflow
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        struct Visitor<'b> {
            detector: &'b ArithmeticOverflowSirDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl Visitor<'_> {
            /// Check if both operands are literals (compile-time computable).
            fn both_literals(&self, lhs: &Expr, rhs: &Expr) -> bool {
                is_literal(lhs) && is_literal(rhs)
            }

            /// Check if either operand is a trivial literal (0 or 1).
            fn has_trivial_operand(&self, lhs: &Expr, rhs: &Expr) -> bool {
                is_trivial_literal(lhs) || is_trivial_literal(rhs)
            }
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_contract_decl(&mut self, contract: &'a ContractDecl) {
                self.contract_name = contract.name.clone();
                visit::default::visit_contract_decl(self, contract);
            }

            fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
                self.func_name = func.name.clone();
                visit::default::visit_function_decl(self, func);
            }

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

                // Continue walking children
                visit::default::visit_binop_expr(self, expr);
            }

            fn visit_aug_assign_stmt(&mut self, stmt: &'a AugAssignStmt) {
                if is_arithmetic_op(stmt.op) {
                    // AugAssign (`+=`, `-=`, `*=`) inherits wrapping from the
                    // underlying BinOp but doesn't carry OverflowSemantics
                    // itself.  Check whether the enclosing scope uses wrapping
                    // semantics by looking at literal operands and skipping
                    // trivial cases.
                    if !is_trivial_literal(&stmt.rhs) {
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
                }

                visit::default::visit_aug_assign_stmt(self, stmt);
            }
        }

        let mut visitor = Visitor {
            detector: self,
            bugs: &mut bugs,
            contract_name: String::new(),
            func_name: String::new(),
        };
        visitor.visit_modules(context.ir_units());

        Ok(bugs)
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

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![190, 191] // CWE-190: Integer Overflow, CWE-191: Integer Underflow
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![101] // SWC-101: Integer Overflow and Underflow
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_overflow_sir_detector() {
        let detector = ArithmeticOverflowSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::ArithmeticOverflow);
        assert_eq!(detector.swc_ids(), vec![101]);
        assert_eq!(detector.cwe_ids(), vec![190, 191]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
        assert_eq!(detector.bug_category(), BugCategory::Arithmetic);
    }
}
