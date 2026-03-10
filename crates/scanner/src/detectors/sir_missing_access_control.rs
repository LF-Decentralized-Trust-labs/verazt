//! SIR Missing Access Control Detector
//!
//! Detects public functions that write to storage without any auth guard.

use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::id::PassId;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use mlir::sir::utils::query as structural;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;

/// SIR structural detector for missing access control.
#[derive(Debug, Default)]
pub struct SirMissingAccessControlDetector;

impl SirMissingAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for SirMissingAccessControlDetector {
    fn id(&self) -> PassId {
        PassId::SirMissingAccessControl
    }

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

    fn dependencies(&self) -> Vec<PassId> {
        vec![]
    }
}

impl BugDetectionPass for SirMissingAccessControlDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let mlir::sir::Decl::Contract(contract) = decl {
                    let storage_vars = structural::storage_names(contract);
                    if storage_vars.is_empty() {
                        continue;
                    }

                    for member in &contract.members {
                        if let mlir::sir::MemberDecl::Function(func) = member {
                            if !structural::is_public_function(func) {
                                continue;
                            }

                            if let Some(body) = &func.body {
                                if !structural::has_storage_write(body, &storage_vars) {
                                    continue;
                                }

                                // Check for @requires spec or Assert before write
                                let has_spec_guard =
                                    func.spec.as_ref().is_some_and(|s| !s.requires.is_empty());

                                let has_assert_guard = structural::has_assert_before_storage_write(
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
