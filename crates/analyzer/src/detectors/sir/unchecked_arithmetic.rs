//! SIR Unchecked Arithmetic Detector
//!
//! Detects `BinOp` with `OverflowSemantics::Wrapping` on non-constant operands.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{BinOp, BinOpExpr, ContractDecl, Expr, FunctionDecl, OverflowSemantics};
use std::any::TypeId;

/// SIR structural detector for unchecked arithmetic.
#[derive(Debug, Default)]
pub struct SirUncheckedArithmeticDetector;

impl SirUncheckedArithmeticDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for SirUncheckedArithmeticDetector {
    fn name(&self) -> &'static str {
        "SIR Unchecked Arithmetic"
    }

    fn description(&self) -> &'static str {
        "BinOp with OverflowSemantics::Wrapping on a non-constant operand"
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

fn is_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(_))
}

impl BugDetectionPass for SirUncheckedArithmeticDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::SirUncheckedArithmetic
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        struct Visitor<'b> {
            detector: &'b SirUncheckedArithmeticDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
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

            fn visit_binop_expr(&mut self, binop: &'a BinOpExpr) {
                if binop.overflow == OverflowSemantics::Wrapping
                    && matches!(binop.op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow)
                    && !(is_literal(&binop.lhs) && is_literal(&binop.rhs))
                {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Wrapping arithmetic {:?} in '{}.{}' with non-constant operand",
                            binop.op, self.contract_name, self.func_name
                        )),
                        binop.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                    ));
                }
                visit::default::visit_binop_expr(self, binop);
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
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![190]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![101]
    }

    fn recommendation(&self) -> &'static str {
        "Use checked arithmetic or SafeMath to prevent overflow/underflow"
    }
}
