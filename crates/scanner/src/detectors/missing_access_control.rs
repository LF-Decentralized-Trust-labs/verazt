//! Missing Access Control Detector
//!
//! This module previously contained a GREP-based detector for missing access
//! control.  The functionality has been merged into the SIR structural
//! detector in `sir_missing_access_control.rs`.
//!
//! A thin SIR wrapper is kept here so that the `MissingAccessControl`
//! detector ID can still be registered independently of the
//! `SirMissingAccessControl` ID if needed.

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use analysis::passes::sir::WriteSetArtifact;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::query as structural;
use scirs::sir::{Decl, MemberDecl};
use std::any::TypeId;

/// SIR structural detector for missing access control on public
/// state-modifying functions.
#[derive(Debug, Default)]
pub struct MissingAccessControlSirDetector;

impl MissingAccessControlSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for MissingAccessControlSirDetector {
    fn name(&self) -> &'static str {
        "Missing Access Control"
    }

    fn description(&self) -> &'static str {
        "Detects public functions that modify state without access control"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<analysis::passes::sir::WriteSetPass>()]
    }
}

impl BugDetectionPass for MissingAccessControlSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::MissingAccessControl
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        let write_sets = context.get::<WriteSetArtifact>();

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    let storage_vars = structural::storage_names(contract);
                    if storage_vars.is_empty() {
                        continue;
                    }

                    for member in &contract.members {
                        if let MemberDecl::Function(func) = member {
                            // Only check public/external functions.
                            if !structural::is_public_function(func) {
                                continue;
                            }

                            // Skip constructors / fallback / receive.
                            let name_lower = func.name.to_lowercase();
                            if name_lower.is_empty()
                                || name_lower == "constructor"
                                || name_lower == "fallback"
                                || name_lower == "receive"
                            {
                                continue;
                            }

                            // Check for spec-based or assert-based guard.
                            let has_spec_guard =
                                func.spec.as_ref().is_some_and(|s| !s.requires.is_empty());

                            let has_assert_guard = func.body.as_ref().map_or(false, |body| {
                                structural::has_assert_before_storage_write(body, &storage_vars)
                            });

                            if has_spec_guard || has_assert_guard {
                                continue;
                            }

                            // Check if function modifies state via WriteSet
                            // or structural walk.
                            let has_writes_ws = write_sets
                                .and_then(|ws| ws.get(&(contract.name.clone(), func.name.clone())))
                                .map_or(false, |w| !w.is_empty());

                            let has_writes_structural = func.body.as_ref().map_or(false, |body| {
                                structural::has_storage_write(body, &storage_vars)
                            });

                            if has_writes_ws || has_writes_structural {
                                bugs.push(Bug::new(
                                    self.name(),
                                    Some(&format!(
                                        "Function '{}' in '{}' performs state \
                                         modifications without access control.",
                                        func.name, contract.name,
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
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![284] // CWE-284: Improper Access Control
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![105, 106]
    }

    fn recommendation(&self) -> &'static str {
        "Add access control modifiers (e.g., onlyOwner, onlyRole) to functions \
         that modify sensitive state or perform privileged operations."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-105",
            "https://swcregistry.io/docs/SWC-106",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_access_control_sir_detector() {
        let detector = MissingAccessControlSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::MissingAccessControl);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
