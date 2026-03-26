//! Constant State Variable Detector (SIR structural + WriteSetPass)
//!
//! Detects state variables that could be declared constant or immutable
//! by querying the WriteSetArtifact from the WriteSetPass.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use crate::passes::sir::WriteSetArtifact;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::utils::helpers as structural;
use scirs::sir::{Decl, MemberDecl};
use std::any::TypeId;

/// SIR structural detector for state variables that could be constant.
///
/// Finds state variables that are initialized but never modified,
/// suggesting they could be declared as `constant` or `immutable`.
#[derive(Debug, Default)]
pub struct ConstantStateVarSirDetector;

impl ConstantStateVarSirDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for ConstantStateVarSirDetector {
    fn name(&self) -> &'static str {
        "Constant State Variable"
    }

    fn description(&self) -> &'static str {
        "Detects state variables that could be constant/immutable"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<crate::passes::sir::WriteSetPass>()]
    }
}

impl BugDetectionPass for ConstantStateVarSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::ConstantStateVar
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        // Get the write-set artifact (may be absent if WriteSetPass didn't run)
        let write_sets = context.get::<WriteSetArtifact>();

        for module in context.ir_units() {
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    // Collect all written storage vars across all functions
                    let mut all_written = std::collections::HashSet::new();
                    if let Some(ws) = write_sets {
                        for member in &contract.members {
                            if let MemberDecl::Function(func) = member {
                                let key = (contract.name.clone(), func.name.clone());
                                if let Some(writes) = ws.get(&key) {
                                    all_written.extend(writes.iter().cloned());
                                }
                            }
                        }
                    }

                    for member in &contract.members {
                        if let MemberDecl::Storage(storage) = member {
                            // Skip already constant/immutable
                            if structural::is_constant_storage(storage) {
                                continue;
                            }

                            // Only flag variables with an initializer
                            if storage.init.is_none() {
                                continue;
                            }

                            // If not in any function's write set → effectively constant
                            if !all_written.contains(&storage.name) {
                                bugs.push(Bug::new(
                                    self.name(),
                                    Some(&format!(
                                        "State variable '{}' in '{}' is never modified after \
                                         initialization. Consider declaring it as 'constant' \
                                         or 'immutable' to save gas.",
                                        storage.name, contract.name,
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
        BugKind::Optimization
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
        "Use 'constant' for compile-time constants and 'immutable' for values \
         set once in the constructor. This saves gas by not using storage."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://docs.soliditylang.org/en/latest/contracts.html#constant-and-immutable-state-variables",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_state_var_sir_detector() {
        let detector = ConstantStateVarSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::ConstantStateVar);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
