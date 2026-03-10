//! Taint Propagation Pass
//!
//! Iterative forward dataflow over SSA def-use edges to propagate taint labels.

use crate::context::AnalysisContext;
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use mlir::air::interfaces::TaintLabel;
use mlir::air::ops::OpId;
use std::any::TypeId;
use std::collections::HashMap;

/// Artifact stored in context after taint propagation.
pub type TaintMap = HashMap<OpId, TaintLabel>;

/// Taint propagation analysis pass.
pub struct TaintPropagationPass;

impl Pass for TaintPropagationPass {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn name(&self) -> &'static str {
        "taint-propagation"
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

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl AnalysisPass for TaintPropagationPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut taint_map: TaintMap = HashMap::new();

        for module in ctx.air_units() {
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
                    // Note: in the AIR representation, ops are stored in
                    // BasicBlocks within AIRFunction.  We iterate over the
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

        #[allow(deprecated)]
        ctx.store_artifact("AIR.taint_map", taint_map);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}
