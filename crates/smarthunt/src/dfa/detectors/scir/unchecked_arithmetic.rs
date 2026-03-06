//! SCIR Unchecked Arithmetic Detector
//!
//! Detects `BinOp` with `OverflowSemantics::Wrapping` on non-constant operands.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::scir::structural;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use scir::{BinOp, BinOpExpr, Expr, OverflowSemantics};
use solidity::ast::Loc;

/// SCIR structural detector for unchecked arithmetic.
#[derive(Debug, Default)]
pub struct ScirUncheckedArithmeticDetector;

impl ScirUncheckedArithmeticDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for ScirUncheckedArithmeticDetector {
    fn id(&self) -> PassId {
        PassId::ScirUncheckedArithmetic
    }

    fn name(&self) -> &'static str {
        "SCIR Unchecked Arithmetic"
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

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::IrGeneration]
    }
}

fn is_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(_))
}

impl BugDetectionPass for ScirUncheckedArithmeticDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let scir::Decl::Contract(contract) = decl {
                    for member in &contract.members {
                        if let scir::MemberDecl::Function(func) = member {
                            if let Some(body) = &func.body {
                                structural::walk_binops(body, &mut |binop: &BinOpExpr| {
                                    // Only flag arithmetic ops with Wrapping semantics
                                    if binop.overflow != OverflowSemantics::Wrapping {
                                        return;
                                    }
                                    if !matches!(
                                        binop.op,
                                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow
                                    ) {
                                        return;
                                    }
                                    // Skip if both operands are literals
                                    if is_literal(&binop.lhs) && is_literal(&binop.rhs) {
                                        return;
                                    }

                                    bugs.push(Bug::new(
                                        self.name(),
                                        Some(&format!(
                                            "Wrapping arithmetic {:?} in '{}.{}' with non-constant operand",
                                            binop.op, contract.name, func.name
                                        )),
                                        Loc::new(0, 0, 0, 0),
                                        self.bug_kind(),
                                        self.bug_category(),
                                        self.risk_level(),
                                        self.cwe_ids(),
                                        self.swc_ids(),
                                    ));
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
}
