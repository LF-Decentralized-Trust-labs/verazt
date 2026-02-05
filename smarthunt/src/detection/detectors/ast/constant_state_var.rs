//! Constant State Variable Detector (Stub)
//!
//! Detects state variables that could be declared constant or immutable.
//!
//! NOTE: This is currently a stub implementation. Full detection requires
//! data flow analysis to track state mutations.

use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::analysis::context::AnalysisContext;
use solidity::analysis::pass::Pass;
use solidity::analysis::pass_id::PassId;
use solidity::analysis::pass_level::PassLevel;
use solidity::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult};

/// Detector for state variables that could be constant or immutable.
#[derive(Debug, Default)]
pub struct ConstantStateVarDetector;

impl ConstantStateVarDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for ConstantStateVarDetector {
    fn id(&self) -> PassId {
        PassId::ConstantStateVar
    }

    fn name(&self) -> &'static str {
        "Constant State Variable"
    }

    fn description(&self) -> &'static str {
        "Detects state variables that could be constant/immutable (requires data flow analysis)"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        // Will require: StateAccess, DataFlow
        vec![]
    }
}

impl BugDetectionPass for ConstantStateVarDetector {
    fn detect(&self, _context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // TODO: Implement constant state variable detection
        // This requires:
        // 1. Data flow analysis to track state mutations
        // 2. State access analysis
        // 3. Initialization tracking
        Ok(vec![])
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Optimization
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
            "https://docs.soliditylang.org/en/latest/contracts.html#constant-and-immutable-state-variables"
        ]
    }
}
