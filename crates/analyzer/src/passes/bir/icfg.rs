//! Interprocedural CFG (ICFG) Pass
//!
//! Publishes the per-module ICFGs built by the BIR lowering step (block
//! and op nodes, call/return edges, and external-call re-entry edges) as
//! an analysis artifact.

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::bir::cfg::ICFG;
use std::any::TypeId;

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for the ICFG.
pub struct ICFGArtifact;

impl ContextKey for ICFGArtifact {
    type Value = Vec<ICFG>;
    const NAME: &'static str = "icfg";
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// ICFG construction pass.
#[derive(Debug, Default)]
pub struct ICFGPass;

impl Pass for ICFGPass {
    fn name(&self) -> &'static str {
        "icfg"
    }

    fn description(&self) -> &'static str {
        "Collect interprocedural control flow graphs"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Bir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl AnalysisPass for ICFGPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let icfgs: Vec<ICFG> = ctx.bir_units().iter().map(|module| module.icfg.clone()).collect();
        ctx.store::<ICFGArtifact>(icfgs);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;

    #[test]
    fn test_icfg_pass_empty() {
        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        let pass = ICFGPass;
        pass.run(&mut ctx).unwrap();
        let icfgs = ctx.get::<ICFGArtifact>().unwrap();
        assert!(icfgs.is_empty());
    }
}
