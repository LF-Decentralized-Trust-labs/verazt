//! Visibility detector.
//!
//! Detects missing or incorrect visibility specifiers.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{ContractDef, ContractElem, FuncKind, FuncVis, Loc, VarVis};
use solidity::ast::utils::Visit;

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
        let mut visitor = VisibilityVisitor::new(self);
        for source_unit in &context.source_units {
            visitor.visit_source_unit(source_unit);
        }
        visitor.bugs
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
    fn is_called_internally(&self, _func_name: &solidity::ast::Name, _contract: &ContractDef) -> bool {
        // Simplified check - in a real implementation, we would analyze the call graph
        // For now, we assume all public functions could be called internally
        // This avoids false positives
        true
    }
}

/// Visitor to collect visibility issues.
struct VisibilityVisitor<'a, 'b> {
    detector: &'a VisibilityDetector,
    bugs: Vec<Bug>,
    current_contract: Option<String>,
    _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> VisibilityVisitor<'a, 'b> {
    fn new(detector: &'a VisibilityDetector) -> Self {
        Self {
            detector,
            bugs: Vec::new(),
            current_contract: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b> Visit<'b> for VisibilityVisitor<'a, 'b> {
    fn visit_contract_def(&mut self, contract: &'b ContractDef) {
        self.current_contract = Some(contract.name.base.clone());
        solidity::ast::utils::visit::default::visit_contract_def(self, contract);
        self.current_contract = None;
    }

    fn visit_contract_elem(&mut self, elem: &'b ContractElem) {
        let contract_name = self.current_contract.as_deref().unwrap_or("<unknown>");

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
                        self.detector,
                        Some(&format!(
                            "Function '{}' in contract '{}' has no explicit visibility. \
                             It defaults to public.",
                            func_name, contract_name
                        )),
                        loc,
                    );
                    self.bugs.push(bug);
                }
            }
            ContractElem::Var(state_var) => {
                // Check if state variable has explicit visibility (not None)
                let has_visibility = !matches!(state_var.visibility, VarVis::None);

                if !has_visibility {
                    let var_name = state_var.name.base.as_str();
                    let loc = state_var.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                    let bug = create_bug(
                        self.detector,
                        Some(&format!(
                            "State variable '{}' in contract '{}' has no explicit visibility. \
                             It defaults to internal.",
                            var_name, contract_name
                        )),
                        loc,
                    );
                    self.bugs.push(bug);
                }
            }
            _ => {}
        }
        solidity::ast::utils::visit::default::visit_contract_elem(self, elem);
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
