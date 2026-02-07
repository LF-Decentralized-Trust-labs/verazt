//! Def-Use Chain Analysis
//!
//! Tracks variable definitions and their uses.

use crate::analysis::pass::{AnalysisPass, Pass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::analysis::context::AnalysisContext;
use crate::analysis::passes::ir::BasicBlockId;
use solidity::ast::Loc;
use std::collections::HashMap;

/// Variable identifier.
pub type VarId = String;

/// Definition site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Loc,
}

/// Use site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Loc,
}

/// Def-Use chain artifact.
#[derive(Debug, Clone)]
pub struct DefUseChain {
    pub def_uses: HashMap<Definition, Vec<Use>>,
    pub use_defs: HashMap<Use, Vec<Definition>>,
}

/// Def-Use chain analysis pass.
pub struct DefUseChainPass;

impl DefUseChainPass {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for DefUseChainPass {
    fn id(&self) -> PassId {
        PassId::DefUseChain
    }

    fn name(&self) -> &'static str {
        "Def-Use Chain Analysis"
    }

    fn description(&self) -> &'static str {
        "Computes def-use and use-def chains"
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

impl AnalysisPass for DefUseChainPass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // TODO: Implement def-use analysis
        context.store_artifact("def_use_completed", true);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.has_artifact("def_use_completed")
    }
}
