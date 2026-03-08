//! ANIR Generation Pass
//!
//! Runs Pass 2a: SIR → AnirModule.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, Pass, PassError, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;

/// Analysis pass that lowers SIR modules to ANIR modules.
pub struct AnirGenerationPass;

impl Pass for AnirGenerationPass {
    fn id(&self) -> PassId {
        PassId::AnirGeneration
    }

    fn name(&self) -> &'static str {
        "anir-generation"
    }

    fn description(&self) -> &'static str {
        "Lower SIR to Anir (Pass 2a)"
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

impl AnalysisPass for AnirGenerationPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let sir_modules = ctx.ir_units().clone();
        let anir_modules = sir_modules
            .iter()
            .map(|m| mlir::air::lower::lower_module(m))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PassError::ExecutionFailed(self.name().to_string(), e.to_string()))?;
        ctx.set_air_units(anir_modules);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.has_air()
    }
}
