//! SIR Missing Access Control Detector
//!
//! Detects public functions that write to storage without any auth guard.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::ContractDecl;
use scirs::sir::dialect::EvmFunctionExt;
use std::any::TypeId;

/// SIR structural detector for missing access control.
#[derive(Debug, Default)]
pub struct SirMissingAccessControlDetector;

impl SirMissingAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for SirMissingAccessControlDetector {
    fn name(&self) -> &'static str {
        "SIR Missing Access Control"
    }

    fn description(&self) -> &'static str {
        "Public function writes storage without any auth guard (SIR tree pattern)"
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

impl BugDetectionPass for SirMissingAccessControlDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::SirMissingAccessControl
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let scirs::sir::Decl::Contract(contract) = decl {
                    let storage_vars = contract.storage_names();
                    if storage_vars.is_empty() {
                        continue;
                    }

                    for member in &contract.members {
                        if let scirs::sir::MemberDecl::Function(func) = member {
                            if !func.is_public() {
                                continue;
                            }

                            if let Some(body) = &func.body {
                                if !ContractDecl::has_storage_write(body, &storage_vars) {
                                    continue;
                                }

                                // Check for @requires spec or Assert before write
                                let has_spec_guard =
                                    func.spec.as_ref().is_some_and(|s| !s.requires.is_empty());

                                let has_assert_guard =
                                    ContractDecl::has_assert_before_storage_write(
                                        body,
                                        &storage_vars,
                                    );

                                if !has_spec_guard && !has_assert_guard {
                                    bugs.push(Bug::new(
                                        self.name(),
                                        Some(&format!(
                                            "Public function '{}.{}' writes storage without access control",
                                            contract.name, func.name
                                        )),
                                        func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
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
        vec![284]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![105]
    }

    fn recommendation(&self) -> &'static str {
        "Ensure all public state-modifying functions are protected by appropriate access control mechanisms."
    }
}
