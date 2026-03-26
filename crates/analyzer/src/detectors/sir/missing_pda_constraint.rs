//! SIR Missing PDA Constraint Detector (Anchor-specific)
//!
//! Detects Anchor accounts loaded without `#anchor.constraint="seeds=[...]"`.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use std::any::TypeId;

/// SIR structural detector for missing PDA constraint (Anchor-specific).
#[derive(Debug, Default)]
pub struct SirMissingPdaConstraintDetector;

impl SirMissingPdaConstraintDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for SirMissingPdaConstraintDetector {
    fn name(&self) -> &'static str {
        "SIR Missing PDA Constraint"
    }

    fn description(&self) -> &'static str {
        "Anchor: account loaded without #anchor.constraint=\"seeds=[...]\""
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

impl BugDetectionPass for SirMissingPdaConstraintDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::SirMissingPdaConstraint
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let scirs::sir::Decl::Contract(contract) = decl {
                    // Find AccountsContext declarations
                    for member in &contract.members {
                        if let scirs::sir::MemberDecl::Dialect(
                            scirs::sir::DialectMemberDecl::Anchor(
                                scirs::sir::dialect::anchor::AnchorMemberDecl::AccountsContext(
                                    scirs::sir::dialect::anchor::AnchorAccountsContext {
                                        name: ctx_name,
                                        accounts,
                                        ..
                                    },
                                ),
                            ),
                        ) = member
                        {
                            for account in accounts {
                                // Check if this is a PDA-type account
                                // (Account<T>) without seeds constraint
                                let is_account_type = matches!(
                                    account.ty,
                                    scirs::sir::dialect::anchor::AnchorType::Account(_)
                                );

                                if is_account_type {
                                    let has_seeds = account
                                        .constraint
                                        .as_ref()
                                        .is_some_and(|c| c.contains("seeds"));

                                    if !has_seeds {
                                        bugs.push(Bug::new(
                                            self.name(),
                                            Some(&format!(
                                                "Account '{}' in context '{}' of '{}' lacks seeds constraint (PDA validation)",
                                                account.name, ctx_name, contract.name
                                            )),
                                            Loc::new(0, 0, 0, 0),
                                            self.bug_kind(),
                                            self.bug_category(),
                                            self.risk_level(),
                                            self.cwe_ids(),
                                            self.swc_ids(),
                                        ));
                                    }
                                }
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
        BugCategory::AccessControl
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
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
        "Ensure PDA constraint is enforced on the account"
    }
}
