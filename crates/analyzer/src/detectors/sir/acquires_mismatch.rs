//! SIR Acquires Mismatch Detector (Move-specific)
//!
//! Detects `move.borrow_global<T>` calls where `T` is not listed in the
//! function's `#move.acquires` attribute.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::visit::{self, Visit};
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

        struct Visitor<'b> {
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
            declared_acquires: Vec<String>,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_contract_decl(&mut self, contract: &'a scirs::sir::ContractDecl) {
                self.contract_name = contract.name.clone();
                visit::default::visit_contract_decl(self, contract);
            }

            fn visit_function_decl(&mut self, func: &'a scirs::sir::FunctionDecl) {
                self.func_name = func.name.clone();

                // Get #move.acquires attribute
                let acquires_attr = func.attrs.iter().find(|a| {
                    a.namespace == "move" && a.key == scirs::sir::attrs::move_attrs::ACQUIRES
                });

                self.declared_acquires = match acquires_attr {
                    Some(attr) => match &attr.value {
                        scirs::sir::AttrValue::String(s) => {
                            s.split(',').map(|t| t.trim().to_string()).collect()
                        }
                        _ => vec![],
                    },
                    None => vec![],
                };

                visit::default::visit_function_decl(self, func);
            }

            fn visit_dialect_expr(&mut self, d: &'a scirs::sir::DialectExpr) {
                if let scirs::sir::DialectExpr::Move(me) = d {
                    let borrowed_ty = match me {
                        scirs::sir::dialect::move_lang::MoveExpr::BorrowGlobal(e) => {
                            Some(e.ty.to_string())
                        }
                        scirs::sir::dialect::move_lang::MoveExpr::BorrowGlobalMut(e) => {
                            Some(e.ty.to_string())
                        }
                        _ => None,
                    };
                    if let Some(ty_str) = borrowed_ty {
                        if !self.declared_acquires.iter().any(|d| *d == ty_str) {
                            self.bugs.push(Bug::new(
                                "SIR Acquires Mismatch",
                                Some(&format!(
                                    "Function '{}.{}' borrows global '{}' but it is not in #move.acquires",
                                    self.contract_name, self.func_name, ty_str
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
            }
        }

        let mut visitor = Visitor {
            bugs: &mut bugs,
            contract_name: String::new(),
            func_name: String::new(),
            declared_acquires: vec![],
        };
        visitor.visit_modules(context.ir_units());

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
