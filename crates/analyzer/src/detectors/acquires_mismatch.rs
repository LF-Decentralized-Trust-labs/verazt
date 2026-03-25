//! SIR Acquires Mismatch Detector (Move-specific)
//!
//! Detects `move.borrow_global<T>` calls where `T` is not listed in the
//! function's `#move.acquires` attribute.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass::meta::PassLevel;
use crate::analysis::pass::meta::PassRepresentation;
use crate::detectors::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::query as structural;
use std::any::TypeId;

/// SIR structural detector for Move acquires mismatch.
#[derive(Debug, Default)]
pub struct SirAcquiresMismatchDetector;

impl SirAcquiresMismatchDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for SirAcquiresMismatchDetector {
    fn name(&self) -> &'static str {
        "SIR Acquires Mismatch"
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

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for SirAcquiresMismatchDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::SirAcquiresMismatch
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let scirs::sir::Decl::Contract(contract) = decl {
                    for member in &contract.members {
                        if let scirs::sir::MemberDecl::Function(func) = member {
                            // Get #move.acquires attribute
                            let acquires_attr = func.attrs.iter().find(|a| {
                                a.namespace == "move"
                                    && a.key == scirs::sir::attrs::move_attrs::ACQUIRES
                            });

                            let declared_acquires: Vec<String> = match acquires_attr {
                                Some(attr) => match &attr.value {
                                    scirs::sir::AttrValue::String(s) => {
                                        s.split(',').map(|t| t.trim().to_string()).collect()
                                    }
                                    _ => vec![],
                                },
                                None => vec![],
                            };

                            if let Some(body) = &func.body {
                                // Walk body for borrow_global / borrow_global_mut
                                structural::walk_dialect_exprs(body, &mut |dexpr| {
                                    if let scirs::sir::DialectExpr::Move(me) = dexpr {
                                        let borrowed_ty = match me {
                                            scirs::sir::dialect::move_lang::MoveExpr::BorrowGlobal { ty, .. } => {
                                                Some(ty.to_string())
                                            }
                                            scirs::sir::dialect::move_lang::MoveExpr::BorrowGlobalMut { ty, .. } => {
                                                Some(ty.to_string())
                                            }
                                            _ => None,
                                        };
                                        if let Some(ty_str) = borrowed_ty {
                                            if !declared_acquires.iter().any(|d| *d == ty_str) {
                                                bugs.push(Bug::new(
                                                    "SIR Acquires Mismatch",
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

    fn recommendation(&self) -> &'static str {
        "Ensure the @acquires annotation matches the lock actually acquired"
    }
}
