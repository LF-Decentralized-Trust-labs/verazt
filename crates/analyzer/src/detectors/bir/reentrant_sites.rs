//! Re-entrant call sites of BIR functions and the contract state that may
//! be accessed around them, shared by the reentrancy detectors.

use crate::context::AnalysisContext;
use crate::detectors::base::traits::{DetectorError, DetectorResult};
use crate::frameworks::bir::{FunctionView, OpPos, StateAccess};
use crate::frameworks::cfa::domtree::DomTree;
use crate::frameworks::dfa::{Direction, OpFacts, PowerSetLattice};
use crate::passes::base::Pass;
use crate::passes::bir::{
    DominanceArtifact, DominancePass, FunctionEffects, FunctionEffectsArtifact, FunctionEffectsPass,
};
use scirs::bir::cfg::{Function, FunctionId};
use scirs::bir::module::Module;
use scirs::bir::ops::{CallTarget, Op, OpKind, Resource};
use scirs::sir::attrs::{evm_attrs, sir_attrs};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════

/// How to fix any reentrancy finding.
pub(crate) const REENTRANCY_RECOMMENDATION: &str =
    "Follow the Checks-Effects-Interactions pattern: perform all state changes \
     before making external calls. Consider using a reentrancy guard \
     (e.g., OpenZeppelin's ReentrancyGuard).";

pub(crate) const REENTRANCY_REFERENCES: &[&str] = &[
    "https://swcregistry.io/docs/SWC-107",
    "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/",
];

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Interprocedural facts about the functions of one module.
pub(crate) struct ModuleFacts<'a> {
    /// Dominator tree of each function.
    doms: &'a HashMap<FunctionId, DomTree>,
    /// Transitive effects of each function.
    effects: &'a HashMap<FunctionId, FunctionEffects>,
}

/// A call that may re-enter the contract, with the state-accessing ops
/// that may execute before and after it.
pub(crate) struct ReentrantSite {
    pub after: Vec<OpPos>,
    pub before: Vec<OpPos>,
    /// The state flag of an inlined mutex protecting the call, if any.
    pub guard_flag: Option<String>,
    pub pos: OpPos,
}

/// Sets of op positions, the facts of the before/after analyses.
type Positions = PowerSetLattice<OpPos>;

// ═══════════════════════════════════════════════════════════════════
// ModuleFacts Implementations
// ═══════════════════════════════════════════════════════════════════

impl<'a> ModuleFacts<'a> {
    /// Each BIR module of `context` with its facts. Modules lacking facts
    /// are skipped.
    pub fn collect(context: &'a AnalysisContext) -> DetectorResult<Vec<(&'a Module, Self)>> {
        let all_effects = context
            .get::<FunctionEffectsArtifact>()
            .ok_or_else(|| DetectorError::MissingAnalysis(FunctionEffectsPass.name().into()))?;
        let all_doms = context
            .get::<DominanceArtifact>()
            .ok_or_else(|| DetectorError::MissingAnalysis(DominancePass.name().into()))?;
        let modules = context.bir_units().iter().filter_map(|module| {
            let id = &module.source_module_id;
            let (effects, doms) = (all_effects.get(id)?, all_doms.get(id)?);
            Some((module, ModuleFacts { doms, effects }))
        });
        Ok(modules.collect())
    }

    pub fn effects_of(&self, func: &FunctionId) -> Option<&FunctionEffects> {
        self.effects.get(func)
    }

    /// The re-entrant call sites of the function of `view`.
    pub fn reentrant_sites(&self, view: &FunctionView) -> Vec<ReentrantSite> {
        let calls: Vec<OpPos> =
            view.positions().filter(|pos| self.is_reentrant_call(view.op(*pos))).collect();
        if calls.is_empty() {
            return vec![];
        }
        let record = |pos: OpPos, op: &Op, seen: &mut Positions| {
            if is_state_effect(op) {
                seen.insert(pos);
            }
        };
        let before = OpFacts::solve(view, Direction::Forward, record);
        let after = OpFacts::solve(view, Direction::Backward, record);
        let dom = self.doms.get(&view.func().id);
        calls
            .into_iter()
            .map(|pos| {
                let before = sorted(before.at(pos));
                let after = sorted(after.at(pos));
                let guard_flag = dom.and_then(|dom| mutex_guard_flag(view, dom, pos, &before, &after));
                ReentrantSite { after, before, guard_flag, pos }
            })
            .collect()
    }

    /// The contract state written by `op`, directly or through an internal
    /// call (whose keys are unknown here).
    pub fn state_written_by(&self, view: &FunctionView, op: &Op) -> Vec<StateAccess> {
        if op.kind.storage_access().is_some() {
            return StateAccess::written_by(view, op).into_iter().collect();
        }
        match &op.kind {
            OpKind::Call(call) => match &call.target {
                CallTarget::Internal(callee) => self
                    .effects_of(callee)
                    .into_iter()
                    .flat_map(|e| &e.writes)
                    .map(|location| StateAccess::unkeyed(location.clone()))
                    .collect(),
                CallTarget::External(_) => vec![],
            },
            _ => vec![],
        }
    }

    /// Returns `true` if `op` is a re-entrant external call, or an internal
    /// call to a function that makes one.
    fn is_reentrant_call(&self, op: &Op) -> bool {
        let OpKind::Call(call) = &op.kind else { return false };
        match &call.target {
            CallTarget::External(_) => call.may_reenter(),
            CallTarget::Internal(callee) => self.effects_of(callee).is_some_and(|e| e.may_reenter),
        }
    }
}

/// Returns `true` if `op` may access contract state: a load or store, or
/// an internal call that may write it.
fn is_state_effect(op: &Op) -> bool {
    op.kind.storage_access().is_some()
        || matches!(&op.kind, OpKind::Call(call) if matches!(call.target, CallTarget::Internal(_)))
}

fn sorted(positions: Positions) -> Vec<OpPos> {
    let mut positions: Vec<OpPos> = positions.elements.into_iter().collect();
    positions.sort();
    positions
}

// ═══════════════════════════════════════════════════════════════════
// Reentrancy guards
// ═══════════════════════════════════════════════════════════════════

/// Returns `true` if the function carries a reentrancy-guard attribute.
pub(crate) fn has_guard_attr(func: &Function) -> bool {
    func.attrs.iter().any(|a| {
        (a.namespace == "sir" && a.key == sir_attrs::REENTRANCY_GUARD)
            || (a.namespace == "evm" && a.key == evm_attrs::NONREENTRANT)
    })
}

/// The flag of an inlined mutex protecting the call at `site`, if any: a
/// state flag that is read before the call, set on every path to it, and
/// set again by another store after it (e.g. an inlined `nonReentrant`
/// modifier).
fn mutex_guard_flag(
    view: &FunctionView,
    dom: &DomTree,
    site: OpPos,
    before: &[OpPos],
    after: &[OpPos],
) -> Option<String> {
    let flag_stores = |positions: &[OpPos]| -> Vec<(OpPos, String)> {
        positions.iter().filter_map(|pos| Some((*pos, flag_store(view.op(*pos))?))).collect()
    };
    let resets = flag_stores(after);
    flag_stores(before)
        .into_iter()
        .filter(|(set, _)| view.always_precedes(dom, *set, site))
        .find(|(set, flag)| {
            resets.iter().any(|(reset, reset_flag)| reset != set && reset_flag == flag)
                && before.iter().any(|pos| reads_state(view.op(*pos), flag))
        })
        .map(|(_, flag)| flag)
}

/// The plain (unindexed) state variable that `op` sets, as its resource name
/// (e.g. `@locked`), if any. Mutex flags are scalars; indexed entries such as
/// `balances[a] = 0` are data, not locks.
fn flag_store(op: &Op) -> Option<String> {
    let OpKind::Store(store) = &op.kind else { return None };
    let is_scalar = matches!(store.resource, Resource::StateVar(_)) && store.keys.is_empty();
    is_scalar.then(|| store.resource.to_string())
}

/// Returns `true` if `op` reads the state resource named `resource` (e.g.
/// `@balances`).
fn reads_state(op: &Op, resource: &str) -> bool {
    matches!(op.kind.storage_access(), Some(access)
        if !access.is_write && access.resource.to_string() == resource)
}
