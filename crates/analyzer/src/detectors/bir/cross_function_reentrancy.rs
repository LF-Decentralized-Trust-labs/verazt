//! Cross-function Reentrancy Detector (BIR control flow)
//!
//! Flags an external call that may re-enter when contract state written
//! after it is read by another public, state-changing function of the same
//! contract: re-entering that function acts on the stale value. A guard on
//! the calling function alone does not help unless the re-entered function
//! shares it.

use super::reentrancy;
use super::reentrant_sites::{
    ModuleFacts, REENTRANCY_RECOMMENDATION, REENTRANCY_REFERENCES, ReentrantSite, has_guard_attr,
};
use crate::context::AnalysisContext;
use crate::detectors::base::traits::DetectorResult;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorId, DetectorMeta, Target};
use crate::frameworks::bir::{FunctionView, StateAccess};
use crate::passes::base::Pass;
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::bir::{DominancePass, FunctionEffectsPass};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use scirs::bir::cfg::{Function, FunctionId};
use std::any::TypeId;
use std::collections::BTreeSet;

// ═══════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::Reentrancy,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[841],
    description: "Detects state written after a re-entrant external call that another \
         public, state-changing function of the contract reads.",
    id: DetectorId::CrossFunctionReentrancy,
    name: "Cross-function Reentrancy",
    recommendation: REENTRANCY_RECOMMENDATION,
    references: REENTRANCY_REFERENCES,
    risk_level: RiskLevel::High,
    swc_ids: &[107],
    target: Target::Evm,
};

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Cross-function reentrancy detector over BIR control flow.
#[derive(Debug, Default)]
pub struct CrossFunctionReentrancyDetector;

/// A public function that may be re-entered, with the state it reads.
struct EntryPoint<'m> {
    func: &'m Function,
    reads: Vec<StateAccess>,
}

// ═══════════════════════════════════════════════════════════════════
// Pass and detector metadata
// ═══════════════════════════════════════════════════════════════════

impl Pass for CrossFunctionReentrancyDetector {
    fn name(&self) -> &'static str {
        META.name
    }

    fn description(&self) -> &'static str {
        META.description
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Bir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![
            TypeId::of::<DominancePass>(),
            TypeId::of::<FunctionEffectsPass>(),
        ]
    }
}

impl BugDetectionPass for CrossFunctionReentrancyDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();
        for (module, facts) in ModuleFacts::collect(context)? {
            let entries = entry_points(&module.functions, &facts);
            for func in module.functions.iter().filter(|f| f.is_public) {
                bugs.extend(self.check_function(func, &entries, &facts));
            }
        }
        Ok(bugs)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Detection
// ═══════════════════════════════════════════════════════════════════

impl CrossFunctionReentrancyDetector {
    /// Report each re-entrant call site of `func` after which state may be
    /// written that another entry point reads.
    fn check_function(
        &self,
        func: &Function,
        entries: &[EntryPoint],
        facts: &ModuleFacts,
    ) -> Vec<Bug> {
        let view = FunctionView::new(func);
        let mut bugs = Vec::new();
        for site in facts.reentrant_sites(&view) {
            // Re-entering `func` itself already makes the call a reentrancy,
            // which reentrancy-flow reports with the same fix: leave the site
            // to it rather than report it twice.
            if !reentrancy::stale_state(func, &view, &site, facts).is_empty() {
                continue;
            }
            let written: Vec<StateAccess> = site
                .after
                .iter()
                .flat_map(|pos| facts.state_written_by(&view, view.op(*pos)))
                .collect();
            if written.is_empty() {
                continue;
            }
            let readers: Vec<(&FunctionId, BTreeSet<&str>)> = entries
                .iter()
                .filter(|entry| may_reenter_into(func, &site, entry))
                .filter_map(|entry| {
                    let stale = stale_reads(&written, &entry.reads);
                    (!stale.is_empty()).then_some((&entry.func.id, stale))
                })
                .collect();
            if !readers.is_empty() {
                let description = describe(func, &readers);
                bugs.push(META.bug(Some(&description), view.report_loc_of(site.pos)));
            }
        }
        bugs
    }
}

/// The public functions of `functions` that change state: re-entering a
/// read-only function cannot corrupt the contract.
fn entry_points<'m>(functions: &'m [Function], facts: &ModuleFacts) -> Vec<EntryPoint<'m>> {
    functions
        .iter()
        .filter(|func| func.is_public)
        .filter_map(|func| {
            let effects = facts.effects_of(&func.id)?;
            let reads = effects
                .reads
                .iter()
                .map(|r| StateAccess::unkeyed(r.clone()))
                .collect();
            (!effects.writes.is_empty()).then_some(EntryPoint { func, reads })
        })
        .collect()
}

/// Returns `true` if the call at `site` in `caller` may re-enter `entry`:
/// another function of the same contract, not protected by the same guard.
fn may_reenter_into(caller: &Function, site: &ReentrantSite, entry: &EntryPoint) -> bool {
    let callee = entry.func;
    if callee.id == caller.id || callee.id.contract() != caller.id.contract() {
        return false;
    }
    let caller_guarded = has_guard_attr(caller) || site.guard_flag.is_some();
    let shares_mutex = site
        .guard_flag
        .as_ref()
        .is_some_and(|flag| entry.reads.iter().any(|read| read.location == *flag));
    !(caller_guarded && (has_guard_attr(callee) || shares_mutex))
}

/// The locations in `written` that some access in `reads` may observe.
fn stale_reads<'w>(written: &'w [StateAccess], reads: &[StateAccess]) -> BTreeSet<&'w str> {
    written
        .iter()
        .filter(|w| reads.iter().any(|r| w.may_alias(r)))
        .map(|w| w.location.as_str())
        .collect()
}

fn describe(func: &Function, readers: &[(&FunctionId, BTreeSet<&str>)]) -> String {
    let readers = readers
        .iter()
        .map(|(id, stale)| {
            let names = stale.iter().copied().collect::<Vec<_>>().join(", ");
            format!("'{}' ({names})", id.0)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Potential cross-function reentrancy in '{}': state written after an external \
         call that may re-enter is read by {readers}, which can run during the call \
         and act on the stale value.",
        func.id.0
    )
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::bir::test_fixtures::*;
    use scirs::sir::FunctionDecl;

    fn detect(functions: Vec<FunctionDecl>) -> Vec<Bug> {
        detect_with(&CrossFunctionReentrancyDetector, functions)
    }

    /// `transfer` reads and updates balances: re-entering it moves funds
    /// that `withdraw` has not yet debited.
    fn transfer() -> FunctionDecl {
        function("transfer", true, vec![read_balance(), write_balance()])
    }

    #[test]
    fn test_flags_state_read_by_another_entry_point() {
        let withdraw = function("withdraw", true, vec![call_out(), write_balance()]);
        let bugs = detect(vec![withdraw, transfer()]);
        assert_eq!(bugs.len(), 1);
        let description = bugs[0].description.as_deref().unwrap();
        assert!(description.contains("'C.transfer' (@balances)"));
    }

    #[test]
    fn test_leaves_same_function_reentrancy_to_reentrancy_flow() {
        // `withdraw` reads the balance before the call and writes it after,
        // so reentrancy-flow reports the call; `transfer` reading it too adds
        // nothing to that finding.
        let withdraw =
            function("withdraw", true, vec![read_balance(), call_out(), write_balance()]);
        assert!(detect(vec![withdraw, transfer()]).is_empty());
    }

    #[test]
    fn test_ignores_read_only_entry_point() {
        let withdraw = function("withdraw", true, vec![call_out(), write_balance()]);
        let balance_of = function("balanceOf", true, vec![read_balance()]);
        assert!(detect(vec![withdraw, balance_of]).is_empty());
    }

    #[test]
    fn test_ignores_internal_reader() {
        let withdraw = function("withdraw", true, vec![call_out(), write_balance()]);
        let settle = function("settle", false, vec![read_balance(), write_balance()]);
        assert!(detect(vec![withdraw, settle]).is_empty());
    }

    #[test]
    fn test_ignores_entry_points_sharing_a_guard() {
        let withdraw = guarded(function("withdraw", true, vec![call_out(), write_balance()]));
        assert!(detect(vec![withdraw, guarded(transfer())]).is_empty());
    }

    #[test]
    fn test_flags_when_only_the_caller_is_guarded() {
        // A guard on `withdraw` does not stop re-entry into `transfer`.
        let withdraw = guarded(function("withdraw", true, vec![call_out(), write_balance()]));
        assert_eq!(detect(vec![withdraw, transfer()]).len(), 1);
    }

    #[test]
    fn test_ignores_entry_point_checking_the_same_mutex() {
        let withdraw = function(
            "withdraw",
            true,
            vec![
                check_locked(),
                set_locked(true),
                call_out(),
                write_balance(),
                set_locked(false),
            ],
        );
        let transfer =
            function("transfer", true, vec![check_locked(), read_balance(), write_balance()]);
        assert!(detect(vec![withdraw, transfer]).is_empty());
    }

    #[test]
    fn test_ignores_gas_limited_transfer() {
        let withdraw = function("withdraw", true, vec![transfer_out(), write_balance()]);
        assert!(detect(vec![withdraw, transfer()]).is_empty());
    }
}
