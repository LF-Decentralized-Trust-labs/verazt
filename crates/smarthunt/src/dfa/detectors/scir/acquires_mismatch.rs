//! SCIR Acquires Mismatch Detector (Move-specific)
//!
//! Detects `move.borrow_global<T>` calls where `T` is not listed in the
//! function's `#move.acquires` attribute.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::scir::structural;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use solidity::ast::Loc;

/// SCIR structural detector for Move acquires mismatch.
#[derive(Debug, Default)]
pub struct ScirAcquiresMismatchDetector;

impl ScirAcquiresMismatchDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for ScirAcquiresMismatchDetector {
    fn id(&self) -> PassId {
        PassId::ScirAcquiresMismatch
    }

    fn name(&self) -> &'static str {
        "SCIR Acquires Mismatch"
    }

    fn description(&self) -> &'static str {
        "Move: borrow_global<T> called on type not in #move.acquires"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::IrGeneration]
    }
}

impl BugDetectionPass for ScirAcquiresMismatchDetector {
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
                            // Get #move.acquires attribute
                            let acquires_attr = func.attrs.iter().find(|a| {
                                a.namespace == "move" && a.key == scir::attrs::move_attrs::ACQUIRES
                            });

                            let declared_acquires: Vec<String> = match acquires_attr {
                                Some(attr) => match &attr.value {
                                    scir::AttrValue::String(s) => {
                                        s.split(',').map(|t| t.trim().to_string()).collect()
                                    }
                                    _ => vec![],
                                },
                                None => vec![],
                            };

                            if let Some(body) = &func.body {
                                // Walk body for borrow_global / borrow_global_mut
                                structural::walk_dialect_exprs(body, &mut |dexpr| {
                                    if let scir::DialectExpr::Move(me) = dexpr {
                                        let borrowed_ty = match me {
                                            scir::dialect::move_lang::MoveExpr::BorrowGlobal { ty, .. } => {
                                                Some(ty.to_string())
                                            }
                                            scir::dialect::move_lang::MoveExpr::BorrowGlobalMut { ty, .. } => {
                                                Some(ty.to_string())
                                            }
                                            _ => None,
                                        };
                                        if let Some(ty_str) = borrowed_ty {
                                            if !declared_acquires.iter().any(|d| *d == ty_str) {
                                                bugs.push(Bug::new(
                                                    "SCIR Acquires Mismatch",
                                                    Some(&format!(
                                                        "Function '{}.{}' borrows global '{}' but it is not in #move.acquires",
                                                        contract.name, func.name, ty_str
                                                    )),
                                                    Loc::new(0, 0, 0, 0),
                                                    BugKind::Vulnerability,
                                                    BugCategory::Other,
                                                    RiskLevel::Medium,
                                                    vec![],
                                                    vec![],
                                                ));
                                            }
                                        }
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
        BugCategory::Other
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }
}
