//! Positional view of a BIR function: ops addressed by block and index,
//! with the control-flow and source-location queries detectors need.

use crate::frameworks::cfa::domtree::DomTree;
use common::loc::Loc;
use scirs::bir::cfg::{BlockId, Function};
use scirs::bir::ops::{Op, OpId, OpKind, OpRef};
use scirs::sir::Lit;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Ops of one function, addressable by block index.
pub struct FunctionView<'f> {
    /// The op defining each SSA value.
    defs: HashMap<OpId, &'f Op>,
    func: &'f Function,
    /// Block index of each block ID.
    index: HashMap<BlockId, usize>,
}

/// The position of an op: block index and op index within the block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OpPos {
    pub block: usize,
    pub op: usize,
}

// ═══════════════════════════════════════════════════════════════════
// FunctionView Implementations
// ═══════════════════════════════════════════════════════════════════

impl<'f> FunctionView<'f> {
    /// Index the blocks and SSA definitions of `func`.
    pub fn new(func: &'f Function) -> Self {
        let index = func.blocks.iter().enumerate().map(|(i, b)| (b.id, i)).collect();
        let defs = func.blocks.iter().flat_map(|b| &b.ops).map(|op| (op.id, op)).collect();
        FunctionView { defs, func, index }
    }

    //-----------------------------------------------------------
    // Ops
    //-----------------------------------------------------------

    /// The viewed function.
    pub fn func(&self) -> &'f Function {
        self.func
    }

    /// The op at `pos`, which must be a position of this function.
    pub fn op(&self, pos: OpPos) -> &'f Op {
        &self.func.blocks[pos.block].ops[pos.op]
    }

    /// The literal `value` is defined as, if it is a constant.
    pub fn constant(&self, value: OpRef) -> Option<&'f Lit> {
        match &self.defs.get(&value.0)?.kind {
            OpKind::Const(lit) => Some(lit),
            _ => None,
        }
    }

    /// Every op position, block by block in layout order.
    pub fn positions(&self) -> impl Iterator<Item = OpPos> + '_ {
        self.func.blocks.iter().enumerate().flat_map(|(block, b)| {
            (0..b.ops.len()).map(move |op| OpPos { block, op })
        })
    }

    //-----------------------------------------------------------
    // Control flow
    //-----------------------------------------------------------

    /// Block indices of the successors of the block at index `block`.
    pub fn successors(&self, block: usize) -> Vec<usize> {
        let targets = self.func.blocks[block].term.successors();
        targets.iter().filter_map(|id| self.index.get(id).copied()).collect()
    }

    /// Returns `true` if every path that reaches `later` executes `earlier`
    /// first.
    pub fn always_precedes(&self, dom: &DomTree, earlier: OpPos, later: OpPos) -> bool {
        if earlier.block == later.block {
            return earlier.op < later.op;
        }
        let block_id = |pos: OpPos| self.func.blocks[pos.block].id;
        dom.dominates(block_id(earlier), block_id(later))
    }

    //-----------------------------------------------------------
    // Source locations
    //-----------------------------------------------------------

    /// The location to report for the op at `pos`: its own, unless it lies
    /// outside the function (code inlined from a modifier), in which case
    /// the function that applies the modifier is reported.
    pub fn report_loc_of(&self, pos: OpPos) -> Loc {
        let loc = self.loc_of(pos);
        match &self.func.span {
            Some(span) if !is_within(&loc, span) => span.clone(),
            _ => loc,
        }
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
