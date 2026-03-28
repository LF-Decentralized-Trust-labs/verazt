//! SIR Missing Modifies Detector
//!
//! Detects public functions that write to storage but lack a `@modifies`
//! annotation.

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

/// SIR structural detector for missing @modifies annotation.
#[derive(Debug, Default)]
pub struct SirMissingModifiesDetector;

impl SirMissingModifiesDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for SirMissingModifiesDetector {
    fn name(&self) -> &'static str {
        "SIR Missing Modifies"
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

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for SirMissingModifiesDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::SirMissingModifies
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
