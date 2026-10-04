//! Worklist dataflow solver over the ops of a BIR function, with facts
//! queryable at any op.

use super::lattice::Lattice;
use crate::frameworks::bir::{FunctionView, OpPos};
use scirs::bir::ops::Op;
use std::collections::VecDeque;
use std::ops::Range;

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// The direction in which facts flow through a function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// Against control flow, from the exits towards the entry.
    Backward,
    /// With control flow, from the entry towards the exits.
    Forward,
}

/// The fixpoint of a dataflow problem over one function.
///
/// The transfer function updates a fact across one op and must be monotone
/// for the solver to terminate.
pub struct OpFacts<'v, 'f, L, T> {
    /// The fact flowing into each block: at its entry for a forward
    /// analysis, at its exit for a backward one.
    block_in: Vec<L>,
    direction: Direction,
    transfer: T,
    view: &'v FunctionView<'f>,
}

// ═══════════════════════════════════════════════════════════════════
// OpFacts Implementations
// ═══════════════════════════════════════════════════════════════════

impl<'v, 'f, L, T> OpFacts<'v, 'f, L, T>
where
    L: Lattice,
    T: Fn(OpPos, &Op, &mut L),
{
    /// Solve the problem defined by `transfer` over the function of `view`,
    /// starting from bottom at the function boundary.
    pub fn solve(view: &'v FunctionView<'f>, direction: Direction, transfer: T) -> Self {
        let count = view.func().blocks.len();
        let succs: Vec<Vec<usize>> = (0..count).map(|block| view.successors(block)).collect();
        let mut preds = vec![Vec::new(); count];
        for (block, targets) in succs.iter().enumerate() {
            for &succ in targets {
                preds[succ].push(block);
            }
        }
        let (inputs, outputs) = match direction {
            Direction::Forward => (&preds, &succs),
            Direction::Backward => (&succs, &preds),
        };
        let mut facts = OpFacts { block_in: vec![L::bottom(); count], direction, transfer, view };
        let mut block_out = vec![L::bottom(); count];
        let mut worklist: VecDeque<usize> = match direction {
            Direction::Forward => (0..count).collect(),
            Direction::Backward => (0..count).rev().collect(),
        };
        let mut queued = vec![true; count];
        while let Some(block) = worklist.pop_front() {
            queued[block] = false;
            let input = inputs[block].iter().fold(L::bottom(), |acc, b| acc.join(&block_out[*b]));
            let output = facts.transfer_ops(block, input.clone(), facts.all_ops(block));
            facts.block_in[block] = input;
            if output != block_out[block] {
                block_out[block] = output;
                for &next in &outputs[block] {
                    if !queued[next] {
                        queued[next] = true;
                        worklist.push_back(next);
                    }
                }
            }
        }
        facts
    }

    /// The fact at the op at `pos`, excluding the op itself: what holds just
    /// before it for a forward analysis, just after it for a backward one.
    pub fn at(&self, pos: OpPos) -> L {
        let ops = match self.direction {
            Direction::Forward => 0..pos.op,
            Direction::Backward => pos.op + 1..self.view.func().blocks[pos.block].ops.len(),
        };
        self.transfer_ops(pos.block, self.block_in[pos.block].clone(), ops)
    }

    fn all_ops(&self, block: usize) -> Range<usize> {
        0..self.view.func().blocks[block].ops.len()
    }

    /// Apply the transfer function to the ops of `block` in `range`, in the
    /// analysis direction.
    fn transfer_ops(&self, block: usize, mut fact: L, range: Range<usize>) -> L {
        let mut apply = |op: usize| {
            let pos = OpPos { block, op };
            (self.transfer)(pos, self.view.op(pos), &mut fact);
        };
        match self.direction {
            Direction::Forward => range.for_each(&mut apply),
            Direction::Backward => range.rev().for_each(&mut apply),
        }
        fact
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::dfa::lattice::PowerSetLattice;
    use scirs::bir::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};
    use scirs::bir::ops::{OpId, OpKind, OpRef};
    use scirs::sir::{BoolLit, Lit};

    type Seen = PowerSetLattice<OpPos>;

    fn constant(id: usize) -> Op {
        Op::new(OpId(id), OpKind::Const(Lit::Bool(BoolLit::new(true, None))))
    }

    /// `bb0 -> bb1 <-> bb2, bb1 -> bb3`: bb1 heads a loop with body bb2.
    fn looping_function() -> Function {
        let mut func = Function::new(FunctionId("C.f".to_string()), true);
        let cond = OpRef(OpId(0));
        let terms = [
            Terminator::jump(BlockId(1)),
            Terminator::branch(cond, BlockId(2), BlockId(3)),
            Terminator::jump(BlockId(1)),
            Terminator::TxnExit { reverted: false },
        ];
        for (i, term) in terms.into_iter().enumerate() {
            let mut block = BasicBlock::new(BlockId(i));
            block.ops = vec![constant(2 * i), constant(2 * i + 1)];
            block.term = term;
            func.blocks.push(block);
        }
        func
    }

    fn record(pos: OpPos, _op: &Op, seen: &mut Seen) {
        seen.insert(pos);
    }

    fn pos(block: usize, op: usize) -> OpPos {
        OpPos { block, op }
    }

    #[test]
    fn test_forward_facts_include_loop_back_edge() {
        let func = looping_function();
        let view = FunctionView::new(&func);
        let before = OpFacts::solve(&view, Direction::Forward, record);
        let at_body = before.at(pos(2, 1));
        // Earlier ops of the block, the loop header, the entry, and (via the
        // back edge) the body's own later op.
        assert!(at_body.contains(&pos(2, 0)) && at_body.contains(&pos(1, 1)));
        assert!(at_body.contains(&pos(0, 0)) && at_body.contains(&pos(2, 1)));
        assert!(!at_body.contains(&pos(3, 0)));
        assert!(!before.at(pos(0, 1)).contains(&pos(0, 1)));
    }

    #[test]
    fn test_backward_facts_exclude_code_that_cannot_follow() {
        let func = looping_function();
        let view = FunctionView::new(&func);
        let after = OpFacts::solve(&view, Direction::Backward, record);
        let at_exit = after.at(pos(3, 0));
        assert_eq!(at_exit.elements.len(), 1);
        assert!(at_exit.contains(&pos(3, 1)));
        let at_entry = after.at(pos(0, 0));
        assert!(at_entry.contains(&pos(2, 0)) && !at_entry.contains(&pos(0, 0)));
    }
}
