//! AIR Generation Pass
//!
//! Runs Pass 2a: SIR → AIRModule.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, Pass, PassError, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;

/// Analysis pass that lowers SIR modules to AIR modules.
pub struct AIRGenerationPass;

impl Pass for AIRGenerationPass {
    fn id(&self) -> PassId {
        PassId::AIRGeneration
    }

    fn name(&self) -> &'static str {
        "AIR-generation"
    }

    fn description(&self) -> &'static str {
        "Lower SIR to AIR (Pass 2a)"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::IrGeneration]
    }
}

impl AnalysisPass for AIRGenerationPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let sir_modules = ctx.ir_units().clone();
        let air_modules = sir_modules
            .iter()
            .map(|m| mlir::air::lower::lower_module(m))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PassError::ExecutionFailed(self.name().to_string(), e.to_string()))?;
        ctx.set_air_units(air_modules);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.has_air()
    }
}
