//! Function Effects Pass
//!
//! Closes each BIR function's effects (may re-enter, state read and
//! written) over the module's static call graph, so callers inherit their
//! callees' effects.

use crate::context::{AnalysisContext, ContextKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::bir::cfg::FunctionId;
use scirs::bir::module::Module;
use std::any::TypeId;
use std::collections::{BTreeSet, HashMap};

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for function effects.
///
/// Maps BIR module id → function → its transitive effects.
pub struct FunctionEffectsArtifact;

impl ContextKey for FunctionEffectsArtifact {
    type Value = HashMap<String, HashMap<FunctionId, FunctionEffects>>;
    const NAME: &'static str = "function_effects";
}

/// Effects of a function, including those of its (transitive) internal
/// callees.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunctionEffects {
    /// Whether the function may make an external call that can re-enter.
    pub may_reenter: bool,
    /// State locations (e.g. `@balances`) the function may read.
    pub reads: BTreeSet<String>,
    /// State locations (e.g. `@balances`) the function may write.
    pub writes: BTreeSet<String>,
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Function effects analysis pass.
#[derive(Debug, Default)]
pub struct FunctionEffectsPass;

impl Pass for FunctionEffectsPass {
    fn name(&self) -> &'static str {
        "function-effects"
    }

    fn description(&self) -> &'static str {
        "Close per-function re-entrancy and state-write effects over internal calls"
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

impl AnalysisPass for FunctionEffectsPass {
    type Artifact = FunctionEffectsArtifact;

    fn run(
        &self,
        ctx: &AnalysisContext,
    ) -> PassResult<HashMap<String, HashMap<FunctionId, FunctionEffects>>> {
        Ok(ctx
            .bir_units()
            .iter()
            .map(|module| (module.source_module_id.clone(), module_effects(module)))
            .collect())
    }
}

/// The transitive effects of every function in `module`: each function's
/// own summary and state reads, joined with its callees' effects until a
/// fixpoint.
fn module_effects(module: &Module) -> HashMap<FunctionId, FunctionEffects> {
    let mut effects: HashMap<FunctionId, FunctionEffects> = module
        .summaries
        .iter()
        .map(|summary| {
            let writes = summary.modifies.iter().map(|r| r.base.clone()).collect();
            let own = FunctionEffects {
                may_reenter: !summary.reentrancy_safe,
                reads: BTreeSet::new(),
                writes,
            };
            (summary.func_id.clone(), own)
        })
        .collect();
    for func in &module.functions {
        let reads = func
            .blocks
            .iter()
            .flat_map(|block| &block.ops)
            .filter_map(|op| op.kind.storage_access())
            .filter(|access| !access.is_write)
            .map(|access| access.resource.to_string());
        effects.entry(func.id.clone()).or_default().reads.extend(reads);
    }
    let size = |e: &FunctionEffects| (e.may_reenter, e.reads.len(), e.writes.len());
    let mut changed = true;
    while changed {
        changed = false;
        for (caller, callee) in &module.call_graph.static_edges {
            let Some(callee_effects) = effects.get(callee).cloned() else { continue };
            let caller_effects = effects.entry(caller.clone()).or_default();
            let before = size(caller_effects);
            caller_effects.may_reenter |= callee_effects.may_reenter;
            caller_effects.reads.extend(callee_effects.reads);
            caller_effects.writes.extend(callee_effects.writes);
            changed |= before != size(caller_effects);
        }
    }
    effects
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use scirs::bir::cfg::{BasicBlock, BlockId, Function};
    use scirs::bir::interfaces::StorageRef;
    use scirs::bir::ops::{LoadOp, Op, OpId, OpKind, Resource};
    use scirs::bir::summary::FunctionSummary;

    fn id(name: &str) -> FunctionId {
        FunctionId(name.to_string())
    }

    fn summary(name: &str, reenters: bool, writes: &[&str]) -> FunctionSummary {
        let mut summary = FunctionSummary::new(id(name));
        summary.reentrancy_safe = !reenters;
        summary.modifies =
            writes.iter().map(|w| StorageRef { base: w.to_string(), index_count: 0 }).collect();
        summary
    }

    #[test]
    fn test_effects_propagate_through_call_chain() {
        // a → b → c, where only c writes state and makes a re-entrant call.
        // The edges are listed callee-first so one sweep is not enough.
        let mut module = Module::new("m".to_string());
        module.summaries =
            vec![summary("a", false, &[]), summary("b", false, &[]), summary("c", true, &["@x"])];
        module.call_graph.add_static_edge(id("a"), id("b"));
        module.call_graph.add_static_edge(id("b"), id("c"));
        module.call_graph.static_edges.reverse();

        let effects = module_effects(&module);
        assert!(effects[&id("a")].may_reenter);
        assert!(effects[&id("a")].writes.contains("@x"));
    }

    #[test]
    fn test_reads_propagate_to_callers() {
        // a → b, where only b loads @x.
        let load = LoadOp { resource: Resource::StateVar("x".to_string()), keys: vec![] };
        let mut block = BasicBlock::new(BlockId(0));
        block.ops.push(Op::new(OpId(0), OpKind::Load(load)));
        let mut b = Function::new(id("b"), false);
        b.blocks.push(block);
        let mut module = Module::new("m".to_string());
        module.functions = vec![Function::new(id("a"), true), b];
        module.call_graph.add_static_edge(id("a"), id("b"));

        let effects = module_effects(&module);
        assert!(effects[&id("a")].reads.contains("@x"));
        assert!(effects[&id("a")].writes.is_empty());
    }

    #[test]
    fn test_effects_do_not_flow_to_callees() {
        let mut module = Module::new("m".to_string());
        module.summaries = vec![summary("a", true, &["@x"]), summary("b", false, &[])];
        module.call_graph.add_static_edge(id("a"), id("b"));

        let effects = module_effects(&module);
        assert_eq!(effects[&id("b")], FunctionEffects::default());
    }
}
