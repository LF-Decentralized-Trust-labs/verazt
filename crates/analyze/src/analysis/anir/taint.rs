//! ANIR Taint Propagation Pass
//!
//! Iterative forward dataflow over SSA def-use edges to propagate taint labels.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, Pass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use scavir::air::interfaces::TaintLabel;
use scavir::air::ops::OpId;
use std::collections::HashMap;

/// Artifact stored in context after taint propagation.
pub type TaintMap = HashMap<OpId, TaintLabel>;

/// Taint propagation analysis pass.
pub struct AnirTaintPropagationPass;

impl Pass for AnirTaintPropagationPass {
    fn id(&self) -> PassId {
        PassId::AnirTaintPropagation
    }

    fn name(&self) -> &'static str {
        "anir-taint-propagation"
    }

    fn description(&self) -> &'static str {
        "Taint propagation through SSA def-use chains"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::AnirGeneration]
    }
}

impl AnalysisPass for AnirTaintPropagationPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut taint_map: TaintMap = HashMap::new();

        for module in ctx.anir_units() {
            // Initialize from taint seeds
            for seed in &module.taint_graph.seeds {
                let existing = taint_map.entry(seed.op).or_insert(TaintLabel::Clean);
                if seed.label > *existing {
                    *existing = seed.label;
                }
            }

            // Propagate through ICFG — iterate until fixed point
            let mut changed = true;
            let mut iteration = 0;
            const MAX_ITERATIONS: usize = 100;

            while changed && iteration < MAX_ITERATIONS {
                changed = false;
                iteration += 1;

                for func in &module.summaries {
                    // Walk ops via the ICFG nodes — for each StmtNode op,
                    // propagate taint through operands.
                    // Note: in the ANIR representation, ops are stored in
                    // BasicBlocks within AnirFunction.  We iterate over the
                    // icfg nodes to identify ops.
                    let _ = func; // summaries are used for reading
                }

                // For now, do a simple propagation over the existing
                // taint graph propagation edges
                for &(src, dst) in &module.taint_graph.propagation {
                    if let Some(&src_label) = taint_map.get(&src) {
                        let entry = taint_map.entry(dst).or_insert(TaintLabel::Clean);
                        if src_label > *entry {
                            *entry = src_label;
                            changed = true;
                        }
                    }
                }
            }
        }

        ctx.store_artifact("anir.taint_map", taint_map);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}
