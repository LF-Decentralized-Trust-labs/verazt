//! SCIR Missing Access Control Detector
//!
//! Detects public functions that write to storage without any auth guard.

use crate::analysis::context::AnalysisContext;
use crate::analysis::scir::structural;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use solidity::ast::Loc;

/// SCIR structural detector for missing access control.
#[derive(Debug, Default)]
pub struct ScirMissingAccessControlDetector;

impl ScirMissingAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for ScirMissingAccessControlDetector {
    fn id(&self) -> PassId {
        PassId::ScirMissingAccessControl
    }

    fn name(&self) -> &'static str {
        "SCIR Missing Access Control"
    }

    fn description(&self) -> &'static str {
        "Public function writes storage without any auth guard (SCIR tree pattern)"
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

impl BugDetectionPass for ScirMissingAccessControlDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let scir::Decl::Contract(contract) = decl {
                    let storage_vars = structural::storage_names(contract);
                    if storage_vars.is_empty() {
                        continue;
                    }

                    for member in &contract.members {
                        if let scir::MemberDecl::Function(func) = member {
                            if !structural::is_public_function(func) {
                                continue;
                            }

                            if let Some(body) = &func.body {
                                if !structural::has_storage_write(body, &storage_vars) {
                                    continue;
                                }

                                // Check for @requires spec or Assert before write
                                let has_spec_guard = func
                                    .spec
                                    .as_ref()
                                    .is_some_and(|s| !s.requires.is_empty());

                                let has_assert_guard = structural::has_assert_before_storage_write(body, &storage_vars);

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
}
