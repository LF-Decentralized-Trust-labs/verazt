//! SCIR Missing Modifies Detector
//!
//! Detects public functions that write to storage but lack a `@modifies`
//! annotation.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::scir::structural;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use solidity::ast::Loc;

/// SCIR structural detector for missing @modifies annotation.
#[derive(Debug, Default)]
pub struct ScirMissingModifiesDetector;

impl ScirMissingModifiesDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for ScirMissingModifiesDetector {
    fn id(&self) -> PassId {
        PassId::ScirMissingModifies
    }

    fn name(&self) -> &'static str {
        "SCIR Missing Modifies"
    }

    fn description(&self) -> &'static str {
        "Public storage-writing function has no @modifies annotation"
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

impl BugDetectionPass for ScirMissingModifiesDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            for decl in &module.decls {
                if let scavir::sir::Decl::Contract(contract) = decl {
                    let storage_vars = structural::storage_names(contract);
                    if storage_vars.is_empty() {
                        continue;
                    }

                    for member in &contract.members {
                        if let scavir::sir::MemberDecl::Function(func) = member {
                            if !structural::is_public_function(func) {
                                continue;
                            }

                            if let Some(body) = &func.body {
                                if !structural::has_storage_write(body, &storage_vars) {
                                    continue;
                                }

                                // Check for @modifies annotation
                                let has_modifies =
                                    func.spec.as_ref().is_some_and(|s| !s.modifies.is_empty());

                                if !has_modifies {
                                    bugs.push(Bug::new(
                                        self.name(),
                                        Some(&format!(
                                            "Public function '{}.{}' writes storage but lacks @modifies annotation",
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
        BugKind::Refactoring
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::CodeQuality
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
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
        "Add @modifies annotation to indicate storage writes"
    }
}
