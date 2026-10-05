//! Interval Analysis Pass
//!
//! Abstract interpretation over the integer interval lattice for SSA
//! values. A worklist re-evaluates every block that uses a value whose
//! interval grew; block parameters join the arguments of all incoming
//! edges and are widened at loop headers (targets of back edges, found by
//! dominance) to ensure termination.

use crate::context::{AnalysisContext, ContextKey};
use crate::frameworks::cfa::domtree::DomTree;
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::bir::cfg::{BlockId, Function, FunctionId};
use scirs::bir::ops::{OpId, OpKind, OpRef};
use scirs::sir::{BinOp, Lit};
use std::any::TypeId;
use std::collections::{HashMap, HashSet, VecDeque};

// ═══════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════

/// Block visits allowed per block before the analysis of a function gives
/// up. Widening bounds the visits of reducible loops well below this;
/// irreducible ones have no widening point and may never stabilize.
const MAX_VISITS_PER_BLOCK: usize = 64;

// ═══════════════════════════════════════════════════════════════════
// Interval type
// ═══════════════════════════════════════════════════════════════════

/// Abstract interval for integer values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interval {
    /// Unreachable / empty set.
    Bottom,
    /// Concrete range `[lo, hi]` (signed, using i128 for simplicity).
    Range { lo: i128, hi: i128 },
    /// Unknown / unbounded.
    Top,
}

impl Interval {
    /// Join (least upper bound) of two intervals.
    pub fn join(&self, other: &Interval) -> Interval {
        match (self, other) {
            (Interval::Bottom, x) | (x, Interval::Bottom) => x.clone(),
            (Interval::Top, _) | (_, Interval::Top) => Interval::Top,
            (Interval::Range { lo: l1, hi: h1 }, Interval::Range { lo: l2, hi: h2 }) => {
                Interval::Range { lo: (*l1).min(*l2), hi: (*h1).max(*h2) }
            }
        }
    }

    /// Widen: if the new interval extends beyond old, push to ±∞.
    pub fn widen(&self, new: &Interval) -> Interval {
        match (self, new) {
            (Interval::Bottom, x) => x.clone(),
            (_, Interval::Bottom) => self.clone(),
            (Interval::Top, _) | (_, Interval::Top) => Interval::Top,
            (Interval::Range { lo: l1, hi: h1 }, Interval::Range { lo: l2, hi: h2 }) => {
                let lo = if *l2 < *l1 { i128::MIN } else { *l1 };
                let hi = if *h2 > *h1 { i128::MAX } else { *h1 };
                if lo == i128::MIN && hi == i128::MAX {
                    Interval::Top
                } else {
                    Interval::Range { lo, hi }
                }
            }
        }
    }

    /// Check if the interval can potentially overflow a given bit-width.
    pub fn can_overflow(&self, type_max: i128, type_min: i128) -> bool {
        match self {
            Interval::Top => true,
            Interval::Bottom => false,
            Interval::Range { lo, hi } => *hi > type_max || *lo < type_min,
        }
    }

    /// Add two intervals.
    pub fn add(&self, other: &Interval) -> Interval {
        match (self, other) {
            (Interval::Bottom, _) | (_, Interval::Bottom) => Interval::Bottom,
            (Interval::Top, _) | (_, Interval::Top) => Interval::Top,
            (Interval::Range { lo: l1, hi: h1 }, Interval::Range { lo: l2, hi: h2 }) => {
                let lo = l1.checked_add(*l2).unwrap_or(i128::MIN);
                let hi = h1.checked_add(*h2).unwrap_or(i128::MAX);
                if lo == i128::MIN && hi == i128::MAX {
                    Interval::Top
                } else {
                    Interval::Range { lo, hi }
                }
            }
        }
    }

    /// Subtract two intervals.
    pub fn sub(&self, other: &Interval) -> Interval {
        match (self, other) {
            (Interval::Bottom, _) | (_, Interval::Bottom) => Interval::Bottom,
            (Interval::Top, _) | (_, Interval::Top) => Interval::Top,
            (Interval::Range { lo: l1, hi: h1 }, Interval::Range { lo: l2, hi: h2 }) => {
                let lo = l1.checked_sub(*h2).unwrap_or(i128::MIN);
                let hi = h1.checked_sub(*l2).unwrap_or(i128::MAX);
                if lo == i128::MIN && hi == i128::MAX {
                    Interval::Top
                } else {
                    Interval::Range { lo, hi }
                }
            }
        }
    }

    /// Multiply two intervals.
    pub fn mul(&self, other: &Interval) -> Interval {
        match (self, other) {
            (Interval::Bottom, _) | (_, Interval::Bottom) => Interval::Bottom,
            (Interval::Top, _) | (_, Interval::Top) => Interval::Top,
            (Interval::Range { lo: l1, hi: h1 }, Interval::Range { lo: l2, hi: h2 }) => {
                let products = [
                    l1.checked_mul(*l2),
                    l1.checked_mul(*h2),
                    h1.checked_mul(*l2),
                    h1.checked_mul(*h2),
                ];
                let mut lo = i128::MAX;
                let mut hi = i128::MIN;
                for p in &products {
                    match p {
                        Some(v) => {
                            lo = lo.min(*v);
                            hi = hi.max(*v);
                        }
                        None => return Interval::Top,
                    }
                }
                Interval::Range { lo, hi }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Artifact
// ═══════════════════════════════════════════════════════════════════

/// Artifact key for interval analysis.
///
/// Maps BIR module id → function → its intervals. `OpId`s are only unique
/// within a function.
pub struct IntervalArtifact;

impl ContextKey for IntervalArtifact {
    type Value = HashMap<String, HashMap<FunctionId, FunctionIntervals>>;
    const NAME: &'static str = "interval";
}

/// The intervals of the SSA values of one function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionIntervals {
    /// Whether the analysis reached a fixpoint. If it did not, every value
    /// is `Top`.
    pub converged: bool,
    /// The interval of each reachable SSA value (block parameter or op
    /// result) at its definition point.
    pub values: HashMap<OpId, Interval>,
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Interval analysis pass.
#[derive(Debug, Default)]
pub struct IntervalPass;

impl Pass for IntervalPass {
    fn name(&self) -> &'static str {
        "interval"
    }

    fn description(&self) -> &'static str {
        "Abstract interpretation over integer intervals"
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

impl AnalysisPass for IntervalPass {
    type Artifact = IntervalArtifact;

    fn run(
        &self,
        ctx: &AnalysisContext,
    ) -> PassResult<HashMap<String, HashMap<FunctionId, FunctionIntervals>>> {
        let result = ctx
            .bir_units()
            .iter()
            .map(|module| {
                let funcs = module
                    .functions
                    .iter()
                    .map(|func| (func.id.clone(), function_intervals(func)))
                    .collect();
                (module.source_module_id.clone(), funcs)
            })
            .collect();

        Ok(result)
    }
}

/// The intervals of the SSA values of `func`, solved with a worklist of
/// block indices starting at the entry block.
fn function_intervals(func: &Function) -> FunctionIntervals {
    let mut values: HashMap<OpId, Interval> = HashMap::new();
    let Some(dom) = DomTree::build(func) else {
        return FunctionIntervals { converged: true, values };
    };
    let index: HashMap<BlockId, usize> = func
        .blocks
        .iter()
        .enumerate()
        .map(|(i, block)| (block.id, i))
        .collect();
    let headers = loop_headers(func, &dom, &index);
    let users = value_users(func);

    // The entry block's parameters are bound by the caller.
    for param in &func.blocks[0].params {
        values.insert(param.id, Interval::Top);
    }
    let count = func.blocks.len();
    let mut worklist = VecDeque::from([0]);
    let mut queued = vec![false; count];
    queued[0] = true;
    let mut visited = vec![false; count];
    let mut budget = MAX_VISITS_PER_BLOCK * count;

    while let Some(current) = worklist.pop_front() {
        if budget == 0 {
            return unconverged_intervals(func);
        }
        budget -= 1;
        queued[current] = false;
        visited[current] = true;
        let block = &func.blocks[current];
        let mut dirty: Vec<usize> = Vec::new();
        let mark_users = |id: OpId, dirty: &mut Vec<usize>| {
            dirty.extend(users.get(&id).into_iter().flatten().copied())
        };

        for op in &block.ops {
            let interval = eval_op(&op.kind, &values);
            if update_interval(&mut values, op.id, &interval, false) {
                mark_users(op.id, &mut dirty);
            }
        }

        // Bind successor block parameters to the incoming arguments
        for call in block.term.block_calls() {
            let Some(&target) = index.get(&call.block) else {
                continue;
            };
            let widen = headers.contains(&target);
            let mut changed = !visited[target];
            for (param, OpRef(arg)) in func.blocks[target].params.iter().zip(&call.args) {
                let incoming = values.get(arg).cloned().unwrap_or(Interval::Top);
                if update_interval(&mut values, param.id, &incoming, widen) {
                    changed = true;
                    mark_users(param.id, &mut dirty);
                }
            }
            if changed {
                dirty.push(target);
            }
        }

        for next in dirty {
            if !queued[next] {
                queued[next] = true;
                worklist.push_back(next);
            }
        }
    }

    FunctionIntervals { converged: true, values }
}

/// Indices of the loop headers of `func`: the targets of back edges, i.e.
/// edges whose target dominates their source.
fn loop_headers(
    func: &Function,
    dom: &DomTree,
    index: &HashMap<BlockId, usize>,
) -> HashSet<usize> {
    func.blocks
        .iter()
        .flat_map(|block| {
            let succs = block.term.successors().into_iter();
            succs.filter(move |succ| dom.dominates(*succ, block.id))
        })
        .filter_map(|succ| index.get(&succ).copied())
        .collect()
}

/// Indices of the blocks that use each SSA value, in an op or in the
/// terminator.
fn value_users(func: &Function) -> HashMap<OpId, Vec<usize>> {
    let mut users: HashMap<OpId, Vec<usize>> = HashMap::new();
    for (i, block) in func.blocks.iter().enumerate() {
        let op_uses = block.ops.iter().flat_map(|op| op.kind.operands());
        for OpRef(id) in op_uses.chain(block.term.operands()) {
            users.entry(id).or_default().push(i);
        }
    }
    users
}

/// Join `incoming` into the interval of `id`, widening it if `widen` is
/// set; returns `true` if the interval changed.
fn update_interval(
    values: &mut HashMap<OpId, Interval>,
    id: OpId,
    incoming: &Interval,
    widen: bool,
) -> bool {
    let old = values.get(&id).cloned().unwrap_or(Interval::Bottom);
    let joined = old.join(incoming);
    let new = if widen { old.widen(&joined) } else { joined };
    if values.get(&id) == Some(&new) {
        return false;
    }
    values.insert(id, new);
    true
}

/// The conservative result for a function whose analysis did not converge:
/// every SSA value is `Top`.
fn unconverged_intervals(func: &Function) -> FunctionIntervals {
    let params = func
        .blocks
        .iter()
        .flat_map(|block| block.params.iter().map(|param| param.id));
    let ops = func
        .blocks
        .iter()
        .flat_map(|block| block.ops.iter().map(|op| op.id));
    let values = params.chain(ops).map(|id| (id, Interval::Top)).collect();
    FunctionIntervals { converged: false, values }
}

/// Evaluate the interval for a single Op.
fn eval_op(kind: &OpKind, state: &HashMap<OpId, Interval>) -> Interval {
    match kind {
        OpKind::Const(lit) => {
            if let Some(val) = lit_to_i128(lit) {
                Interval::Range { lo: val, hi: val }
            } else {
                Interval::Top
            }
        }
        OpKind::BinOp { op, lhs: OpRef(l), rhs: OpRef(r), .. } => {
            let left = state.get(l).cloned().unwrap_or(Interval::Top);
            let right = state.get(r).cloned().unwrap_or(Interval::Top);
            match op {
                BinOp::Add => left.add(&right),
                BinOp::Sub => left.sub(&right),
                BinOp::Mul => left.mul(&right),
                _ => Interval::Top,
            }
        }
        OpKind::Param { .. } => Interval::Top,
        _ => Interval::Top,
    }
}

/// Try to convert a Lit to an i128.
fn lit_to_i128(lit: &Lit) -> Option<i128> {
    match lit {
        Lit::Num(n) => {
            use scirs::sir::lits::Num;
            match &n.value {
                Num::Int(i) => {
                    use num_traits::ToPrimitive;
                    i.value.to_i128()
                }
                Num::Hex(h) => i128::from_str_radix(h.value.trim_start_matches("0x"), 16).ok(),
                _ => None,
            }
        }
        Lit::Bool(b) => Some(if b.value { 1 } else { 0 }),
        _ => None,
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::AnalysisConfig;
    use scirs::bir::cfg::{BasicBlock, BlockCall, BlockParam, Terminator};
    use scirs::bir::ops::{Op, SsaName};
    use scirs::sir::{IntNum, Num, NumLit, OverflowSemantics, Type};

    /// `%id = const value`.
    fn constant(id: usize, value: i64) -> Op {
        let num = Num::Int(IntNum::new(value.into(), Type::I256));
        Op::new(OpId(id), OpKind::Const(Lit::Num(NumLit::new(num, None))))
    }

    /// `%id = binop op %lhs, %rhs`.
    fn binop(id: usize, op: BinOp, lhs: usize, rhs: usize) -> Op {
        let (lhs, rhs) = (OpRef(OpId(lhs)), OpRef(OpId(rhs)));
        let overflow = OverflowSemantics::Checked;
        Op::new(OpId(id), OpKind::BinOp { op, lhs, rhs, overflow })
    }

    /// Block `id` with parameters `params`, `ops`, and terminator `term`.
    fn block(id: usize, params: &[usize], ops: Vec<Op>, term: Terminator) -> BasicBlock {
        let mut block = BasicBlock::new(BlockId(id));
        block.params = params
            .iter()
            .map(|&p| BlockParam { id: OpId(p), name: SsaName::new("p", 0), ty: Type::I256 })
            .collect();
        block.ops = ops;
        block.term = term;
        block
    }

    /// A jump to `target` passing `args`.
    fn jump(target: usize, args: &[usize]) -> Terminator {
        let args = args.iter().map(|&a| OpRef(OpId(a))).collect();
        Terminator::Jump(BlockCall { block: BlockId(target), args })
    }

    fn exit() -> Terminator {
        Terminator::TxnExit { reverted: false }
    }

    /// The intervals of a single function made of `blocks`.
    fn analyze(blocks: Vec<BasicBlock>) -> FunctionIntervals {
        let mut func = Function::new(FunctionId("f".into()), true);
        func.blocks = blocks;
        function_intervals(&func)
    }

    fn range(lo: i128, hi: i128) -> Interval {
        Interval::Range { lo, hi }
    }

    #[test]
    fn test_counting_loop_widens_induction_variable() {
        // bb0: jump bb1(0)
        // bb1(%2): %3 = %2 < 10; branch %3, bb2, bb3
        // bb2: %5 = %2 + 1; jump bb1(%5)
        let blocks = vec![
            block(0, &[], vec![constant(0, 0), constant(1, 10)], jump(1, &[0])),
            block(
                1,
                &[2],
                vec![binop(3, BinOp::Lt, 2, 1)],
                Terminator::branch(OpRef(OpId(3)), BlockId(2), BlockId(3)),
            ),
            block(2, &[], vec![constant(4, 1), binop(5, BinOp::Add, 2, 4)], jump(1, &[5])),
            block(3, &[], vec![], exit()),
        ];
        let result = analyze(blocks);
        assert!(result.converged);
        assert_eq!(result.values[&OpId(2)], range(0, i128::MAX));
        assert_eq!(result.values[&OpId(5)], range(1, i128::MAX));
    }

    #[test]
    fn test_straight_line_code_is_precise() {
        let ops = vec![
            constant(0, 3),
            constant(1, 4),
            binop(2, BinOp::Mul, 0, 1),
            binop(3, BinOp::Sub, 2, 0),
        ];
        let result = analyze(vec![block(0, &[], ops, exit())]);
        assert!(result.converged);
        assert_eq!(result.values[&OpId(2)], range(12, 12));
        assert_eq!(result.values[&OpId(3)], range(9, 9));
    }

    #[test]
    fn test_forward_jump_to_lower_numbered_block_is_not_widened() {
        // bb0 branches to bb2 and bb3, which both jump forward to bb1,
        // passing 5 and 7: bb1 is no loop header despite its lower number.
        let blocks = vec![
            block(
                0,
                &[],
                vec![constant(0, 5), constant(1, 1)],
                Terminator::branch(OpRef(OpId(1)), BlockId(2), BlockId(3)),
            ),
            block(1, &[3], vec![], exit()),
            block(2, &[], vec![], jump(1, &[0])),
            block(3, &[], vec![constant(2, 7)], jump(1, &[2])),
        ];
        let result = analyze(blocks);
        assert!(result.converged);
        assert_eq!(result.values[&OpId(3)], range(5, 7));
    }

    #[test]
    fn test_irreducible_loop_is_marked_unconverged() {
        // bb1 and bb2 form a cycle entered at both blocks, so neither
        // dominates the other and no edge is widened.
        let blocks = vec![
            block(
                0,
                &[],
                vec![constant(0, 0), constant(1, 1)],
                Terminator::Branch {
                    cond: OpRef(OpId(1)),
                    then_dest: BlockCall { block: BlockId(1), args: vec![OpRef(OpId(0))] },
                    else_dest: BlockCall { block: BlockId(2), args: vec![OpRef(OpId(0))] },
                },
            ),
            block(1, &[2], vec![binop(3, BinOp::Add, 2, 1)], jump(2, &[3])),
            block(2, &[4], vec![binop(5, BinOp::Add, 4, 1)], jump(1, &[5])),
        ];
        let result = analyze(blocks);
        assert!(!result.converged);
        assert_eq!(result.values[&OpId(0)], Interval::Top);
        assert_eq!(result.values[&OpId(5)], Interval::Top);
    }

    #[test]
    fn test_interval_arithmetic() {
        let a = Interval::Range { lo: 0, hi: 100 };
        let b = Interval::Range { lo: 1, hi: 50 };

        let sum = a.add(&b);
        assert_eq!(sum, Interval::Range { lo: 1, hi: 150 });

        let diff = a.sub(&b);
        assert_eq!(diff, Interval::Range { lo: -50, hi: 99 });

        let prod = a.mul(&b);
        assert_eq!(prod, Interval::Range { lo: 0, hi: 5000 });
    }

    #[test]
    fn test_interval_join() {
        let a = Interval::Range { lo: 0, hi: 10 };
        let b = Interval::Range { lo: 5, hi: 20 };
        let joined = a.join(&b);
        assert_eq!(joined, Interval::Range { lo: 0, hi: 20 });

        assert_eq!(Interval::Bottom.join(&a), a);
        assert_eq!(a.join(&Interval::Top), Interval::Top);
    }

    #[test]
    fn test_interval_can_overflow() {
        let safe = Interval::Range { lo: 0, hi: 100 };
        let uint256_max: i128 = i128::MAX; // simplified
        assert!(!safe.can_overflow(uint256_max, 0));

        let risky = Interval::Top;
        assert!(risky.can_overflow(uint256_max, 0));
    }

    #[test]
    fn test_interval_widen() {
        let old = Interval::Range { lo: 0, hi: 10 };
        let new = Interval::Range { lo: 0, hi: 20 };
        let widened = old.widen(&new);
        // hi extended → push to i128::MAX
        assert_eq!(widened, Interval::Range { lo: 0, hi: i128::MAX });
    }

    #[test]
    fn test_intervals_do_not_leak_between_functions() {
        // `f` and `g` both define `%0`, as constants 1 and 2.
        let mut module = scirs::bir::Module::new("m".into());
        for (name, value) in [("f", 1), ("g", 2)] {
            let mut func = Function::new(FunctionId(name.into()), true);
            let mut block = BasicBlock::new(BlockId(0));
            block.ops = vec![constant(0, value)];
            block.term = Terminator::TxnExit { reverted: false };
            func.blocks = vec![block];
            module.functions.push(func);
        }
        let mut ctx = AnalysisContext::new(vec![], AnalysisConfig::default());
        ctx.set_bir_units(vec![module]);

        let intervals = IntervalPass.run(&ctx).unwrap();
        let of = |name: &str| intervals["m"][&FunctionId(name.into())].values[&OpId(0)].clone();
        assert_eq!(of("f"), Interval::Range { lo: 1, hi: 1 });
        assert_eq!(of("g"), Interval::Range { lo: 2, hi: 2 });
    }
}
