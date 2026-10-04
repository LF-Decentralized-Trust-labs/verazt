//! Reentrancy Detector (BIR control flow)
//!
//! Flags an external call that can run arbitrary code (`may_reenter`) when
//! contract state that may be read before the call may be written after it
//! on some control-flow path, either directly or inside a function of the
//! same contract called later. The read-before condition is what makes the
//! attack work: a re-entrant call observes the stale value.
//!
//! Compared with the syntactic SIR detector, this follows real control flow
//! (branches, loops, `break` / `continue`), sees writes hidden in internal
//! calls, ignores gas-limited `transfer` / `send`, and skips functions
//! protected by a reentrancy guard: either an attribute, or an inlined
//! mutex (a state flag that is checked and set before the call and reset
//! after it).

use super::reentrant_sites::{
    has_guard_attr, ModuleFacts, REENTRANCY_RECOMMENDATION, REENTRANCY_REFERENCES,
};
use crate::context::AnalysisContext;
use crate::detectors::base::traits::DetectorResult;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorId, DetectorMeta, Target};
use crate::frameworks::bir::{FunctionView, StateAccess};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::Pass;
use crate::passes::bir::{DominancePass, FunctionEffectsPass};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use scirs::bir::cfg::Function;
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
    description: "Detects state writes reachable after a re-entrant external call, \
         following BIR control flow and internal calls.",
    id: DetectorId::ReentrancyFlow,
    name: "Reentrancy (control flow)",
    recommendation: REENTRANCY_RECOMMENDATION,
    references: REENTRANCY_REFERENCES,
    risk_level: RiskLevel::Critical,
    swc_ids: &[107],
    target: Target::Evm,
};

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Reentrancy detector over BIR control flow.
#[derive(Debug, Default)]
pub struct ReentrancyFlowDetector;

// ═══════════════════════════════════════════════════════════════════
// Pass and detector metadata
// ═══════════════════════════════════════════════════════════════════

impl Pass for ReentrancyFlowDetector {
    fn name(&self) -> &'static str {
        META.name
    }

    fn description(&self) -> &'static str {
        META.description
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Bir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<DominancePass>(), TypeId::of::<FunctionEffectsPass>()]
    }
}

impl BugDetectionPass for ReentrancyFlowDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();
        for (module, facts) in ModuleFacts::collect(context)? {
            for func in module.functions.iter().filter(|f| f.is_public && !has_guard_attr(f)) {
                bugs.extend(self.check_function(func, &facts));
            }
        }
        Ok(bugs)
    }

    /// The syntactic SIR detectors flag the same write-after-call pattern
    /// with less precision.
    fn supersedes(&self) -> Vec<DetectorId> {
        vec![DetectorId::CeiViolation, DetectorId::Reentrancy]
    }
}

// ═══════════════════════════════════════════════════════════════════
// Detection
// ═══════════════════════════════════════════════════════════════════

impl ReentrancyFlowDetector {
    /// Report each re-entrant call site of `func` after which state read
    /// before it may be written.
    fn check_function(&self, func: &Function, facts: &ModuleFacts) -> Vec<Bug> {
        let view = FunctionView::new(func);
        let mut bugs = Vec::new();
        for site in facts.reentrant_sites(&view) {
            if site.guard_flag.is_some() {
                continue;
            }
            let written: Vec<StateAccess> = site
                .after
                .iter()
                .flat_map(|pos| facts.state_written_by(&view, view.op(*pos)))
                .collect();
            let read: Vec<StateAccess> =
                site.before.iter().filter_map(|pos| StateAccess::read_by(&view, view.op(*pos))).collect();
            let stale: BTreeSet<&str> = written
                .iter()
                .filter(|w| read.iter().any(|r| w.may_alias(r)))
                .map(|w| w.location.as_str())
                .collect();
            if stale.is_empty() {
                continue;
            }
            let description = describe(func, &stale);
            bugs.push(META.bug(Some(&description), view.report_loc_of(site.pos)));
        }
        bugs
    }
}

fn describe(func: &Function, stale: &BTreeSet<&str>) -> String {
    let names = stale.iter().copied().collect::<Vec<_>>().join(", ");
    format!(
        "Potential reentrancy in '{}': state ({names}) is read before an external call \
         that may re-enter, and written after it.",
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
    use scirs::sir::{AssignStmt, FunctionDecl, IfStmt, Stmt, WhileStmt};

    fn detect(functions: Vec<FunctionDecl>) -> Vec<Bug> {
        detect_with(&ReentrancyFlowDetector, functions)
    }

    #[test]
    fn test_flags_stale_read_written_after_reentrant_call() {
        let body = vec![read_balance(), call_out(), write_balance()];
        let bugs = detect(vec![function("withdraw", true, body)]);
        assert_eq!(bugs.len(), 1);
        assert!(bugs[0].description.as_deref().unwrap().contains("@balances"));
    }

    #[test]
    fn test_ignores_write_not_read_before_call() {
        // Without a prior read, a re-entrant call observes no stale state.
        let bugs = detect(vec![function("withdraw", true, vec![call_out(), write_balance()])]);
        assert!(bugs.is_empty());
    }

    #[test]
    fn test_ignores_write_before_call() {
        // Checks-Effects-Interactions: the state is updated first.
        let body = vec![read_balance(), write_balance(), call_out()];
        let bugs = detect(vec![function("withdraw", true, body)]);
        assert!(bugs.is_empty());
    }

    #[test]
    fn test_flags_write_before_call_inside_loop() {
        // while (c) { read; write; call(); } writes again after the call on
        // the next iteration.
        let body = vec![read_balance(), write_balance(), call_out()];
        let loop_ = Stmt::While(WhileStmt { cond: var("c"), body, invariant: None, span: None });
        let bugs = detect(vec![function("withdraw", true, vec![loop_])]);
        assert_eq!(bugs.len(), 1);
    }

    #[test]
    fn test_follows_write_into_internal_helper() {
        let body = vec![read_balance(), call_out(), call_internal("settle")];
        let withdraw = function("withdraw", true, body);
        let settle = function("settle", false, vec![write_balance()]);
        let bugs = detect(vec![withdraw, settle]);
        assert_eq!(bugs.len(), 1);
        assert!(bugs[0].description.as_deref().unwrap().contains("@balances"));
    }

    #[test]
    fn test_ignores_gas_limited_transfer() {
        let body = vec![read_balance(), transfer_out(), write_balance()];
        let bugs = detect(vec![function("withdraw", true, body)]);
        assert!(bugs.is_empty());
    }

    #[test]
    fn test_skips_inlined_mutex_guard() {
        // if (locked) revert(); locked = true; call(); write; locked = false;
        let body = vec![
            check_locked(),
            set_locked(true),
            read_balance(),
            call_out(),
            write_balance(),
            set_locked(false),
        ];
        let bugs = detect(vec![function("withdraw", true, body)]);
        assert!(bugs.is_empty());
    }

    #[test]
    fn test_flags_mutex_set_on_one_path_only() {
        // if (c) { locked = true; } call(); ...: the call is unguarded when
        // `c` is false.
        let lock_if = Stmt::If(IfStmt {
            cond: var("c"),
            then_body: vec![set_locked(true)],
            else_body: None,
            span: None,
        });
        let body = vec![
            check_locked(),
            lock_if,
            read_balance(),
            call_out(),
            write_balance(),
            set_locked(false),
        ];
        let bugs = detect(vec![function("withdraw", true, body)]);
        assert_eq!(bugs.len(), 1);
    }

    #[test]
    fn test_skips_guard_set_to_computed_value() {
        // mark = locked; locked = amount; call(); write; locked = mark;
        let set_locked_to = |value: &str| Stmt::Assign(AssignStmt {
            lhs: var("locked"),
            rhs: var(value),
            span: None,
        });
        let body = vec![
            check_locked(),
            set_locked_to("amount"),
            read_balance(),
            call_out(),
            write_balance(),
            set_locked_to("mark"),
        ];
        let bugs = detect(vec![function("withdraw", true, body)]);
        assert!(bugs.is_empty());
    }

    #[test]
    fn test_ignores_disjoint_constant_keys() {
        // balances[0] is read before the call; only balances[1] is written.
        let body = vec![read_balance_at(int(false)), call_out(), write_balance_at(int(true))];
        let bugs = detect(vec![function("withdraw", true, body)]);
        assert!(bugs.is_empty());
    }

    #[test]
    fn test_flags_same_constant_key() {
        let body = vec![read_balance_at(int(true)), call_out(), write_balance_at(int(true))];
        let bugs = detect(vec![function("withdraw", true, body)]);
        assert_eq!(bugs.len(), 1);
    }
}
