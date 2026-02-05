//! Visibility detector.
//!
//! Detects missing or incorrect visibility specifiers.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{ContractDef, ContractElem, FuncKind, FuncVis, Loc, VarVis};

/// Detector for visibility issues.
pub struct VisibilityDetector;

impl VisibilityDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VisibilityDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for VisibilityDetector {
    fn id(&self) -> &'static str {
        "visibility"
    }

    fn name(&self) -> &'static str {
        "Visibility Issues"
    }

    fn description(&self) -> &'static str {
        "Functions without explicit visibility specifiers default to public, \
         which may not be the intended visibility."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![]
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
        vec![100, 108] // SWC-100: Function Default Visibility, SWC-108: State Variable Default Visibility
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();
        
        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let solidity::ast::SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(&contract.name.base, &contract, &mut bugs);
                }
            }
        }
        
        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Always explicitly specify visibility for functions and state variables. \
         Use 'private' for internal implementation details, 'internal' for functions \
         that should be callable by derived contracts, and 'public'/'external' for \
         functions that should be callable from outside."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-100",
            "https://swcregistry.io/docs/SWC-108",
        ]
    }
}

impl VisibilityDetector {
    fn check_contract(&self, contract_name: &str, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            match elem {
                ContractElem::Func(func) => {
                    // Check if function has explicit visibility (not None)
                    let has_visibility = !matches!(func.visibility, FuncVis::None);
                    
                    // Skip constructors, fallback, and receive functions
                    let is_special = matches!(func.kind, FuncKind::Constructor | FuncKind::Fallback | FuncKind::Receive);
                    
                    if !has_visibility && !is_special {
                        let func_name = func.name.base.as_str();
                        let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Function '{}' in contract '{}' has no explicit visibility. \
                                 It defaults to public.",
                                func_name, contract_name
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                    
                    // Check for public functions that could be external
                    let is_public = matches!(func.visibility, FuncVis::Public);
                    
                    if is_public && !self.is_called_internally(&func.name, contract) {
                        let func_name = func.name.base.as_str();
                        let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Function '{}' in contract '{}' is public but could be external \
                                 (not called internally). Consider using 'external' for gas optimization.",
                                func_name, contract_name
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
                ContractElem::Var(state_var) => {
                    // Check if state variable has explicit visibility (not None)
                    let has_visibility = !matches!(state_var.visibility, VarVis::None);
                    
                    if !has_visibility {
                        let var_name = state_var.name.base.as_str();
                        let loc = state_var.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "State variable '{}' in contract '{}' has no explicit visibility. \
                                 It defaults to internal.",
                                var_name, contract_name
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
                _ => {}
            }
        }
    }

    fn is_called_internally(&self, _func_name: &solidity::ast::Name, _contract: &ContractDef) -> bool {
        // Simplified check - in a real implementation, we would analyze the call graph
        // For now, we assume all public functions could be called internally
        // This avoids false positives
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_detector() {
        let detector = VisibilityDetector::new();
        assert_eq!(detector.id(), "visibility");
        assert_eq!(detector.swc_ids(), vec![100, 108]);
    }
}
