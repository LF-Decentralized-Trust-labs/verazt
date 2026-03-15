//! Interval Analysis Pass
//!
//! Abstract interpretation over the integer interval lattice for SSA
//! values.  Per-block abstract states are merged at join points.
//! Widening on back edges to ensure termination.

use crate::context::{AnalysisContext, ArtifactKey};
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use crate::passes::base::{AnalysisPass, Pass, PassResult};
use scirs::air::cfg::{BlockId, Terminator};
use scirs::air::ops::{OpId, OpKind, OpRef};
use scirs::sir::{BinOp, Lit};
use std::any::TypeId;
use std::collections::{HashMap, HashSet, VecDeque};

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
/// Maps `OpId` → `Interval` at the definition point.
pub struct IntervalArtifact;

impl ArtifactKey for IntervalArtifact {
    type Value = HashMap<OpId, Interval>;
    const NAME: &'static str = "interval";
}

// ═══════════════════════════════════════════════════════════════════
// Pass
// ═══════════════════════════════════════════════════════════════════

/// Interval analysis pass.
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
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl AnalysisPass for IntervalPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut result: HashMap<OpId, Interval> = HashMap::new();

        for module in ctx.air_units() {
            for func in &module.functions {
                if func.blocks.is_empty() {
                    continue;
                }

                // Identify back edges (targets of branches that jump
                // backwards — simplified: blocks whose id is ≤ source).
                let back_edge_targets: HashSet<BlockId> = func
                    .blocks
                    .iter()
                    .flat_map(|b| {
                        let succs = match &b.term {
                            Terminator::Jump(t) => vec![*t],
                            Terminator::Branch { then_bb, else_bb, .. } => {
                                vec![*then_bb, *else_bb]
                            }
                            _ => vec![],
                        };
                        succs.into_iter().filter(move |s| s.0 <= b.id.0)
                    })
                    .collect();

                // Worklist-based iteration
                let mut worklist: VecDeque<BlockId> = VecDeque::new();
                worklist.push_back(func.blocks[0].id);
                let mut visited: HashSet<BlockId> = HashSet::new();

                let mut iteration = 0;
                const MAX_ITER: usize = 200;

                while let Some(bid) = worklist.pop_front() {
                    if iteration > MAX_ITER {
                        break;
                    }
                    iteration += 1;
                    visited.insert(bid);

                    let block = match func.blocks.iter().find(|b| b.id == bid) {
                        Some(b) => b,
                        None => continue,
                    };

                    let is_loop_header = back_edge_targets.contains(&bid);

                    for op in &block.ops {
                        let interval = eval_op(&op.kind, &result);
                        let final_interval = if is_loop_header {
                            // Widen at loop headers
                            match result.get(&op.id) {
                                Some(old) => old.widen(&interval),
                                None => interval,
                            }
                        } else {
                            match result.get(&op.id) {
                                Some(old) => old.join(&interval),
                                None => interval,
                            }
                        };
                        result.insert(op.id, final_interval);
                    }

                    // Add successors to worklist
                    let succs = match &block.term {
                        Terminator::Jump(t) => vec![*t],
                        Terminator::Branch { then_bb, else_bb, .. } => {
                            vec![*then_bb, *else_bb]
                        }
                        _ => vec![],
                    };
                    for s in succs {
                        if !visited.contains(&s) || back_edge_targets.contains(&s) {
                            worklist.push_back(s);
                        }
                    }
                }
            }
        }

        ctx.store::<IntervalArtifact>(result);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
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
        OpKind::Phi(args) => {
            let mut result = Interval::Bottom;
            for (_, OpRef(id)) in args {
                let incoming = state.get(id).cloned().unwrap_or(Interval::Top);
                result = result.join(&incoming);
            }
            result
        }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
