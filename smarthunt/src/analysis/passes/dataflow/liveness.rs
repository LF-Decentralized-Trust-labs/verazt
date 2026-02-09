//! Liveness Analysis
//!
//! Computes live variables at each program point.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, Pass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use std::collections::HashSet;

/// Variable identifier.
pub type VarId = String;

/// Live set (set of live variables).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveSet {
    pub live_in: HashSet<VarId>,
    pub live_out: HashSet<VarId>,
}

/// Liveness analysis pass.
pub struct LivenessAnalysisPass;

impl LivenessAnalysisPass {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for LivenessAnalysisPass {
    fn id(&self) -> PassId {
        PassId::LivenessAnalysis
    }

    fn name(&self) -> &'static str {
        "Liveness Analysis"
    }

    fn description(&self) -> &'static str {
        "Computes live variables (backward analysis)"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::IrGeneration, PassId::IrCfg]
    }
}

impl AnalysisPass for LivenessAnalysisPass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // TODO: Implement liveness analysis
        context.store_artifact("liveness_completed", true);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.has_artifact("liveness_completed")
    }
}
