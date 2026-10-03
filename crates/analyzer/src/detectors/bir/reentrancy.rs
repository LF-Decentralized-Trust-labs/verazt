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

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::base::traits::{ConfidenceLevel, DetectorError, DetectorResult};
use crate::detectors::BugDetectionPass;
use crate::frameworks::cfa::reachability::ReachabilitySet;
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::Pass;
use crate::passes::bir::{FunctionEffects, FunctionEffectsArtifact, FunctionEffectsPass};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::bir::cfg::{BlockId, Function, FunctionId};
use scirs::bir::ops::{CallTarget, Op, OpId, OpKind, OpRef, Resource};
use scirs::sir::attrs::{evm_attrs, sir_attrs};
use scirs::sir::{Lit, Num, NumLit};
use std::any::TypeId;
use std::collections::{BTreeSet, HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Reentrancy detector over BIR control flow.
#[derive(Debug, Default)]
pub struct ReentrancyFlowDetector;

/// Interprocedural facts about the functions of one module.
struct ModuleFacts<'a> {
    /// Transitive effects of each function, from `FunctionEffectsPass`.
    effects: &'a HashMap<FunctionId, FunctionEffects>,
}

/// The position of an op: block index and op index within the block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OpPos {
    block: usize,
    op: usize,
}

/// Ops of one function, addressable by block index.
struct FunctionView<'f> {
    /// The op defining each SSA value.
    defs: HashMap<OpId, &'f Op>,
    func: &'f Function,
    /// Block index of each block ID.
    index: HashMap<BlockId, usize>,
}

/// An access to contract state: a location such as `@balances` and the
/// keys it is indexed by.
struct StateAccess {
    /// Each key's integer value when it is a constant (e.g. `balances[2]`),
    /// `None` otherwise. Empty when the keys are unknown altogether.
    keys: Vec<Option<String>>,
    location: String,
}

// ═══════════════════════════════════════════════════════════════════
// Pass and detector metadata
// ═══════════════════════════════════════════════════════════════════

impl Pass for ReentrancyFlowDetector {
    fn name(&self) -> &'static str {
        "Reentrancy (control flow)"
    }

    fn description(&self) -> &'static str {
        "Detects state writes reachable after a re-entrant external call, \
         following BIR control flow and internal calls."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Bir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<FunctionEffectsPass>()]
    }
}

impl BugDetectionPass for ReentrancyFlowDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::ReentrancyFlow
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let all_effects = context
            .get::<FunctionEffectsArtifact>()
            .ok_or_else(|| DetectorError::MissingAnalysis(FunctionEffectsPass.name().into()))?;
        let mut bugs = Vec::new();
        for module in context.bir_units() {
            let Some(effects) = all_effects.get(&module.source_module_id) else { continue };
            let facts = ModuleFacts { effects };
            for func in module.functions.iter().filter(|f| f.is_public && !has_guard_attr(f)) {
                bugs.extend(self.check_function(func, &facts));
            }
        }
        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::Reentrancy
    }

    /// The syntactic SIR detectors flag the same write-after-call pattern
    /// with less precision.
    fn supersedes(&self) -> Vec<DetectorId> {
        vec![DetectorId::CeiViolation, DetectorId::Reentrancy]
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Critical
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![841]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![107]
    }

    fn recommendation(&self) -> &'static str {
        "Follow the Checks-Effects-Interactions pattern: perform all state changes \
         before making external calls. Consider using a reentrancy guard \
         (e.g., OpenZeppelin's ReentrancyGuard)."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-107",
            "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/",
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════
// Detection
// ═══════════════════════════════════════════════════════════════════

impl ReentrancyFlowDetector {
    /// Report each re-entrant call site of `func` after which state may be
    /// written.
    fn check_function(&self, func: &Function, facts: &ModuleFacts) -> Vec<Bug> {
        let view = FunctionView::new(func);
        let mut bugs = Vec::new();
        for site in view.positions().filter(|pos| facts.is_reentrant_call(view.op(*pos))) {
            let before = view.positions_before(site);
            let after = view.positions_after(site);
            if has_mutex_guard(&view, &before, &after) {
                continue;
            }
            let written: Vec<StateAccess> =
                after.iter().flat_map(|pos| facts.state_written_by(&view, view.op(*pos))).collect();
            let read: Vec<StateAccess> =
                before.iter().filter_map(|pos| state_read_by(&view, view.op(*pos))).collect();
            let stale: BTreeSet<&str> = written
                .iter()
                .filter(|w| read.iter().any(|r| w.may_alias(r)))
                .map(|w| w.location.as_str())
                .collect();
            if stale.is_empty() {
                continue;
            }
            let description = describe(func, &stale);
            bugs.push(Bug::new(
                self.name(),
                Some(&description),
                view.report_loc_of(site),
                self.bug_kind(),
                self.bug_category(),
                self.risk_level(),
                self.cwe_ids(),
                self.swc_ids(),
                Some(self.recommendation()),
            ));
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

impl StateAccess {
    /// Returns `true` if the two accesses may touch the same state: their
    /// locations overlap and no key position holds two different constants.
    fn may_alias(&self, other: &StateAccess) -> bool {
        let keys_may_match = self.keys.iter().zip(&other.keys).all(|keys| match keys {
            (Some(a), Some(b)) => a == b,
            _ => true,
        });
        overlaps(&self.location, &other.location) && keys_may_match
    }
}

/// Returns `true` if two state locations may overlap: they are equal, or
/// one is a field path inside the other (`@accounts` and `@accounts.balance`).
fn overlaps(a: &str, b: &str) -> bool {
    let inside = |inner: &str, outer: &str| {
        inner.strip_prefix(outer).is_some_and(|rest| rest.starts_with('.'))
    };
    a == b || inside(a, b) || inside(b, a)
}

/// The contract state read by `op`, if any.
fn state_read_by(view: &FunctionView, op: &Op) -> Option<StateAccess> {
    let access = op.kind.storage_access()?;
    (!access.is_write).then(|| view.state_access(&access.resource, &access.keys))
}

/// Returns `true` if the function carries a reentrancy-guard attribute.
fn has_guard_attr(func: &Function) -> bool {
    func.attrs.iter().any(|a| {
        (a.namespace == "sir" && a.key == sir_attrs::REENTRANCY_GUARD)
            || (a.namespace == "evm" && a.key == evm_attrs::NONREENTRANT)
    })
}

/// Returns `true` if the call site is protected by an inlined mutex: some
/// state flag is read and set to a constant before the call, and set to a
/// constant again after it (e.g. an inlined `nonReentrant` modifier).
fn has_mutex_guard(view: &FunctionView, before: &[OpPos], after: &[OpPos]) -> bool {
    let set_to_const = |positions: &[OpPos]| -> HashSet<String> {
        positions.iter().filter_map(|pos| constant_store(view, view.op(*pos))).collect()
    };
    let set_before = set_to_const(before);
    let set_after = set_to_const(after);
    set_before.intersection(&set_after).any(|flag| {
        before.iter().any(|pos| reads_state(view.op(*pos), flag))
    })
}

/// The plain (unindexed) state variable that `op` sets to a constant, as
/// its resource name (e.g. `@locked`), if any. Mutex flags are scalars;
/// indexed entries such as `balances[a] = 0` are data, not locks.
fn constant_store(view: &FunctionView, op: &Op) -> Option<String> {
    let OpKind::Store(store) = &op.kind else { return None };
    if !matches!(store.resource, Resource::StateVar(_)) || !store.keys.is_empty() {
        return None;
    }
    let value = store.value?;
    view.constant(value).map(|_| store.resource.to_string())
}

/// Returns `true` if `op` reads the state resource named `resource` (e.g.
/// `@balances`).
fn reads_state(op: &Op, resource: &str) -> bool {
    matches!(op.kind.storage_access(), Some(access)
        if !access.is_write && access.resource.to_string() == resource)
}

// ═══════════════════════════════════════════════════════════════════
// Module facts
// ═══════════════════════════════════════════════════════════════════

impl ModuleFacts<'_> {
    /// Returns `true` if `op` is a re-entrant external call, or an internal
    /// call to a function that makes one.
    fn is_reentrant_call(&self, op: &Op) -> bool {
        let OpKind::Call(call) = &op.kind else { return false };
        match &call.target {
            CallTarget::External(_) => call.may_reenter(),
            CallTarget::Internal(callee) => self.effects_of(callee).is_some_and(|e| e.may_reenter),
        }
    }

    /// The contract state written by `op`, directly or through an internal
    /// call (whose keys are unknown here).
    fn state_written_by(&self, view: &FunctionView, op: &Op) -> Vec<StateAccess> {
        if let Some(access) = op.kind.storage_access() {
            return access
                .is_write
                .then(|| view.state_access(&access.resource, &access.keys))
                .into_iter()
                .collect();
        }
        match &op.kind {
            OpKind::Call(call) => match &call.target {
                CallTarget::Internal(callee) => self
                    .effects_of(callee)
                    .into_iter()
                    .flat_map(|e| &e.writes)
                    .map(|location| StateAccess { location: location.clone(), keys: vec![] })
                    .collect(),
                CallTarget::External(_) => vec![],
            },
            _ => vec![],
        }
    }

    fn effects_of(&self, func: &FunctionId) -> Option<&FunctionEffects> {
        self.effects.get(func)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Function view
// ═══════════════════════════════════════════════════════════════════

impl<'f> FunctionView<'f> {
    fn new(func: &'f Function) -> Self {
        let index = func.blocks.iter().enumerate().map(|(i, b)| (b.id, i)).collect();
        let defs = func.blocks.iter().flat_map(|b| &b.ops).map(|op| (op.id, op)).collect();
        FunctionView { defs, func, index }
    }

    /// The literal `value` is defined as, if it is a constant.
    fn constant(&self, value: OpRef) -> Option<&'f Lit> {
        match &self.defs.get(&value.0)?.kind {
            OpKind::Const(lit) => Some(lit),
            _ => None,
        }
    }

    /// The access to `resource` indexed by `keys`, with constant integer keys
    /// resolved.
    fn state_access(&self, resource: &Resource, keys: &[OpRef]) -> StateAccess {
        let int_key = |key: &OpRef| match self.constant(*key)? {
            Lit::Num(NumLit { value: Num::Int(n), .. }) => Some(n.value.to_string()),
            _ => None,
        };
        StateAccess { keys: keys.iter().map(int_key).collect(), location: resource.to_string() }
    }

    fn op(&self, pos: OpPos) -> &'f Op {
        &self.func.blocks[pos.block].ops[pos.op]
    }

    fn positions(&self) -> impl Iterator<Item = OpPos> + '_ {
        self.func.blocks.iter().enumerate().flat_map(|(block, b)| {
            (0..b.ops.len()).map(move |op| OpPos { block, op })
        })
    }

    /// Ops that may execute after `site`: later ops of its block and all
    /// ops of blocks reachable from it (including the site's own block
    /// again when it is in a loop).
    fn positions_after(&self, site: OpPos) -> Vec<OpPos> {
        let later = (site.op + 1..self.func.blocks[site.block].ops.len())
            .map(|op| OpPos { block: site.block, op });
        let site_block = self.func.blocks[site.block].id;
        let reachable = ReachabilitySet::forward_strict(self.func, site_block);
        later.chain(self.all_ops_of(&reachable)).collect()
    }

    /// Ops that may execute before `site`: earlier ops of its block and all
    /// ops of blocks that can reach it.
    fn positions_before(&self, site: OpPos) -> Vec<OpPos> {
        let earlier = (0..site.op).map(|op| OpPos { block: site.block, op });
        let site_block = self.func.blocks[site.block].id;
        let reaching = ReachabilitySet::backward_strict(self.func, site_block);
        earlier.chain(self.all_ops_of(&reaching)).collect()
    }

    fn all_ops_of<'a>(&'a self, blocks: &'a ReachabilitySet) -> impl Iterator<Item = OpPos> + 'a {
        blocks.iter().filter_map(|id| self.index.get(&id).copied()).flat_map(move |block| {
            (0..self.func.blocks[block].ops.len()).map(move |op| OpPos { block, op })
        })
    }

    /// The source location of the op at `pos`. Ops lowered from nodes
    /// without a location fall back to the nearest located op of the same
    /// block (usually the statement that consumes the value).
    fn loc_of(&self, pos: OpPos) -> Loc {
        let ops = &self.func.blocks[pos.block].ops;
        let located = |op: &Op| op.span.clone().filter(|span| *span != Loc::default());
        located(&ops[pos.op])
            .or_else(|| ops[pos.op + 1..].iter().find_map(located))
            .or_else(|| ops[..pos.op].iter().rev().find_map(located))
            .unwrap_or_default()
    }

    /// The location to report for the op at `pos`: its own, unless it lies
    /// outside the function (code inlined from a modifier), in which case
    /// the function that applies the modifier is reported.
    fn report_loc_of(&self, pos: OpPos) -> Loc {
        let loc = self.loc_of(pos);
        match &self.func.span {
            Some(span) if !is_within(&loc, span) => span.clone(),
            _ => loc,
        }
    }
}

/// Returns `true` if `inner` lies within the lines of `outer` (and in the
/// same file, when both files are known).
fn is_within(inner: &Loc, outer: &Loc) -> bool {
    let same_file = match (&inner.file, &outer.file) {
        (Some(a), Some(b)) => a == b,
        _ => true,
    };
    same_file && outer.start_line <= inner.start_line && inner.end_line <= outer.end_line
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use crate::passes::base::AnalysisPass;
    use scirs::sir::attrs::sir_attrs;
    use scirs::sir::evm::{EvmExpr, EvmLowLevelCall, EvmMsgSender, EvmTransfer};
    use num_traits::Zero;
    use scirs::sir::{
        AssignStmt, Attr, AttrValue, BoolLit, CallArgs, CallExpr, ContractDecl, Decl, DialectExpr,
        Expr, ExprStmt, FunctionDecl, IfStmt, IndexAccessExpr, IntNum, Lit, MemberDecl, Num,
        NumLit, Param, RevertStmt, StorageDecl, Stmt, StringLit, Type, VarExpr, WhileStmt,
    };

    fn var(name: &str) -> Expr {
        Expr::Var(VarExpr::new(name.to_string(), Type::I256, None))
    }

    fn lit(value: bool) -> Expr {
        Expr::Lit(Lit::Bool(BoolLit::new(value, None)))
    }

    fn sender() -> Expr {
        Expr::Dialect(DialectExpr::Evm(EvmExpr::MsgSender(EvmMsgSender { loc: Loc::default() })))
    }

    fn stmt(expr: Expr) -> Stmt {
        Stmt::Expr(ExprStmt { expr, span: None })
    }

    /// `msg.sender.call{value: amount}("")`
    fn call_out() -> Stmt {
        stmt(Expr::Dialect(DialectExpr::Evm(EvmExpr::LowLevelCall(EvmLowLevelCall {
            target: Box::new(sender()),
            data: Box::new(Expr::Lit(Lit::String(StringLit::new(String::new(), None)))),
            value: Some(Box::new(var("amount"))),
            gas: None,
            loc: Loc::default(),
        }))))
    }

    /// `msg.sender.transfer(amount)`
    fn transfer_out() -> Stmt {
        stmt(Expr::Dialect(DialectExpr::Evm(EvmExpr::Transfer(EvmTransfer {
            target: Box::new(sender()),
            amount: Box::new(var("amount")),
            loc: Loc::default(),
        }))))
    }

    /// `balances[msg.sender] = false`
    fn write_balance() -> Stmt {
        write_balance_at(sender())
    }

    /// `if (balances[msg.sender]) {}`: reads the balance before the call.
    fn read_balance() -> Stmt {
        read_balance_at(sender())
    }

    /// `balances[key]`
    fn balance_at(key: Expr) -> Expr {
        Expr::IndexAccess(IndexAccessExpr {
            base: Box::new(var("balances")),
            index: Some(Box::new(key)),
            ty: Type::I256,
            span: None,
        })
    }

    /// `balances[key] = false`
    fn write_balance_at(key: Expr) -> Stmt {
        Stmt::Assign(AssignStmt { lhs: balance_at(key), rhs: lit(false), span: None })
    }

    /// `if (balances[key]) {}`
    fn read_balance_at(key: Expr) -> Stmt {
        Stmt::If(IfStmt { cond: balance_at(key), then_body: vec![], else_body: None, span: None })
    }

    /// The integer literal `0` or `1`.
    fn int(one: bool) -> Expr {
        let value = if one { IntNum::one() } else { IntNum::new(Zero::zero(), Type::I256) };
        Expr::Lit(Lit::Num(NumLit::new(Num::Int(value), None)))
    }

    fn set_locked(value: bool) -> Stmt {
        Stmt::Assign(AssignStmt { lhs: var("locked"), rhs: lit(value), span: None })
    }

    fn call_internal(name: &str) -> Stmt {
        stmt(Expr::FunctionCall(CallExpr {
            callee: Box::new(var(name)),
            args: CallArgs::Positional(vec![]),
            ty: Type::None,
            span: None,
        }))
    }

    fn function(name: &str, public: bool, body: Vec<Stmt>) -> FunctionDecl {
        let params = vec![Param::new("amount".to_string(), Type::I256)];
        let mut func = FunctionDecl::new(name.to_string(), params, vec![], Some(body), None);
        if public {
            let visibility = AttrValue::String("public".to_string());
            func.attrs.push(Attr::new("sir", sir_attrs::VISIBILITY, visibility));
        }
        func
    }

    /// Run the detector on contract `C { balances; locked; functions }`.
    fn detect(functions: Vec<FunctionDecl>) -> Vec<Bug> {
        let mut members: Vec<MemberDecl> = ["balances", "locked"]
            .iter()
            .map(|n| MemberDecl::Storage(StorageDecl::new(n.to_string(), Type::I256, None, None)))
            .collect();
        members.extend(functions.into_iter().map(MemberDecl::Function));
        let contract = ContractDecl::new("C".to_string(), members, None);
        let module = scirs::sir::Module::new("test", vec![Decl::Contract(contract)]);
        let mut context = AnalysisContext::new(vec![module], AnalysisConfig::default());
        FunctionEffectsPass.run(&mut context).unwrap();
        ReentrancyFlowDetector.detect(&context).unwrap()
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
        let revert = Stmt::Revert(RevertStmt { error: None, args: vec![], span: None });
        let check = Stmt::If(IfStmt {
            cond: var("locked"),
            then_body: vec![revert],
            else_body: None,
            span: None,
        });
        let body = vec![
            check,
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

    #[test]
    fn test_is_within_rejects_lines_outside_function() {
        // A function on lines 15-17; its modifier's code sits on line 21.
        let function = Loc::new(15, 3, 17, 4);
        assert!(is_within(&Loc::new(16, 5, 16, 30), &function));
        assert!(!is_within(&Loc::new(21, 5, 21, 80), &function));
        let elsewhere = Loc::new(16, 5, 16, 30).with_file("Base.sol".to_string());
        let function = function.with_file("Main.sol".to_string());
        assert!(!is_within(&elsewhere, &function));
    }
}
