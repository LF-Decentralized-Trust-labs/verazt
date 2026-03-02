//! Visibility Detector (GREP-based)
//!
//! Detects missing or incorrect visibility specifiers using pattern matching.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{ContractDef, ContractElem, FuncDef, FuncVis, Loc, SourceUnitElem};

/// GREP-based detector for visibility issues.
///
/// Detects functions without explicit visibility specifiers.
#[derive(Debug, Default)]
pub struct VisibilityGrepDetector;

impl VisibilityGrepDetector {
    pub fn new() -> Self {
        Self
    }

    fn check_contract(&self, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.check_function(func, &contract.name.base, bugs);
            }
        }
    }

    fn check_function(&self, func: &FuncDef, contract_name: &str, bugs: &mut Vec<Bug>) {
        // Skip constructors, fallback, and receive functions
        let func_name = func.name.base.as_str();
        if func_name.is_empty()
            || func_name == "constructor"
            || func_name == "fallback"
            || func_name == "receive"
        {
            return;
        }

        // Check if visibility is explicitly specified (None means default/public)
        if func.visibility == FuncVis::None {
            let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
            let bug = create_bug(
                self,
                Some(&format!(
                    "Function '{}' in contract '{}' has no explicit visibility specifier. \
                     Consider adding 'public', 'external', 'internal', or 'private'.",
                    func_name, contract_name,
                )),
                loc,
            );
            bugs.push(bug);
        }

        // Check state variables with default visibility
        // State variables default to internal but explicit is better
    }
}

impl Pass for VisibilityGrepDetector {
    fn id(&self) -> PassId {
        PassId::Visibility
    }

    fn name(&self) -> &'static str {
        "Visibility Issues"
    }

    fn description(&self) -> &'static str {
        "Detects missing or incorrect function visibility specifiers."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }
}

impl BugDetectionPass for VisibilityGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(contract, &mut bugs);
                }
            }
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![710] // CWE-710: Improper Adherence to Coding Standards
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![100, 108]
    }

    fn recommendation(&self) -> &'static str {
        "Explicitly define visibility for all functions and state variables."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-100",
            "https://swcregistry.io/docs/SWC-108",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_grep_detector() {
        let detector = VisibilityGrepDetector::new();
        assert_eq!(detector.id(), PassId::Visibility);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
