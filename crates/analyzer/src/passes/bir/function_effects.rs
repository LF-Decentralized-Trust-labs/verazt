//! Function Effects Pass
//!
//! Closes each BIR function's summary (may re-enter, state written) over
//! the module's static call graph, so callers inherit their callees' effects.

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
    /// State locations (e.g. `@balances`) the function may write.
    pub writes: BTreeSet<String>,
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Function effects analysis pass.
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
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let effects = ctx
            .bir_units()
            .iter()
            .map(|module| (module.source_module_id.clone(), module_effects(module)))
            .collect();
        ctx.store::<FunctionEffectsArtifact>(effects);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}

/// The transitive effects of every function in `module`: each function's
/// own summary, joined with its callees' effects until a fixpoint.
fn module_effects(module: &Module) -> HashMap<FunctionId, FunctionEffects> {
    let mut effects: HashMap<FunctionId, FunctionEffects> = module
        .summaries
        .iter()
        .map(|summary| {
            let writes = summary.modifies.iter().map(|r| r.base.clone()).collect();
            let own = FunctionEffects { may_reenter: !summary.reentrancy_safe, writes };
            (summary.func_id.clone(), own)
        })
        .collect();
    let mut changed = true;
    while changed {
        changed = false;
        for (caller, callee) in &module.call_graph.static_edges {
            let Some(callee_effects) = effects.get(callee).cloned() else { continue };
            let caller_effects = effects.entry(caller.clone()).or_default();
            let before = (caller_effects.may_reenter, caller_effects.writes.len());
            caller_effects.may_reenter |= callee_effects.may_reenter;
            caller_effects.writes.extend(callee_effects.writes);
            changed |= before != (caller_effects.may_reenter, caller_effects.writes.len());
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
    use scirs::bir::interfaces::StorageRef;
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
    fn test_effects_do_not_flow_to_callees() {
        let mut module = Module::new("m".to_string());
        module.summaries = vec![summary("a", true, &["@x"]), summary("b", false, &[])];
        module.call_graph.add_static_edge(id("a"), id("b"));

        let effects = module_effects(&module);
        assert_eq!(effects[&id("b")], FunctionEffects::default());
    }
}
