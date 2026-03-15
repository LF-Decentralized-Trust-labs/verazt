//! Natural-Loop Detection
//!
//! Identifies natural loops from a dominator tree and the function's
//! CFG.  A **natural loop** is defined by a back-edge `(tail → header)`
//! where `header` dominates `tail`.
//!
//! # Outputs
//!
//! - [`LoopInfo`] — collection of all natural loops in a function
//! - [`NaturalLoop`] — a single loop: header, body blocks, back-edge tails, and
//!   exit blocks
//!
//! # Detector integration
//!
//! - Taint DFA solver can process loop headers correctly and avoid redundant
//!   re-propagation over unreachable back-edges.

use super::domtree::{DomTree, terminator_successors};
use scirs::air::cfg::{BlockId, Function};
use std::collections::{HashMap, HashSet, VecDeque};

// ═══════════════════════════════════════════════════════════════════
// NaturalLoop
// ═══════════════════════════════════════════════════════════════════

/// A natural loop in the CFG.
#[derive(Debug, Clone)]
pub struct NaturalLoop {
    /// The loop header (dominates all body blocks).
    pub header: BlockId,
    /// All blocks in the loop body (including the header).
    pub body: HashSet<BlockId>,
    /// Back-edge tail blocks (source of back-edges to the header).
    pub back_edge_tails: Vec<BlockId>,
    /// Exit blocks (successors of body blocks that are outside the loop).
    pub exits: HashSet<BlockId>,
}

// ═══════════════════════════════════════════════════════════════════
// LoopInfo
// ═══════════════════════════════════════════════════════════════════

/// Loop information for a function's CFG.
#[derive(Debug, Clone)]
pub struct LoopInfo {
    /// All natural loops, keyed by header block.
    loops: HashMap<BlockId, NaturalLoop>,
    /// Mapping from block to the innermost loop header it resides in.
    loop_of: HashMap<BlockId, BlockId>,
}

impl LoopInfo {
    /// Compute loop information for a function from its dominator tree.
    ///
    /// Returns `None` if the dominator tree could not be built (e.g., empty
    /// function).
    pub fn build(func: &Function, dom: &DomTree) -> Self {
        let mut loops: HashMap<BlockId, NaturalLoop> = HashMap::new();

        // Step 1: Identify back-edges.
        // A back-edge is (tail → header) where header dominates tail.
        let mut back_edges: Vec<(BlockId, BlockId)> = Vec::new();
        for block in &func.blocks {
            for succ in terminator_successors(&block.term) {
                if dom.dominates(succ, block.id) {
                    back_edges.push((block.id, succ));
                }
            }
        }

        // Step 2: For each back-edge, compute the natural loop body.
        for &(tail, header) in &back_edges {
            let entry = loops.entry(header).or_insert_with(|| NaturalLoop {
                header,
                body: HashSet::new(),
                back_edge_tails: Vec::new(),
                exits: HashSet::new(),
            });
            entry.back_edge_tails.push(tail);

            // BFS backwards from tail to header to find loop body.
            let mut body = HashSet::new();
            body.insert(header);
            if tail != header {
                let mut worklist = VecDeque::new();
                body.insert(tail);
                worklist.push_back(tail);

                // Build predecessor map.
                let preds = build_predecessor_map(func);

                while let Some(block) = worklist.pop_front() {
                    if let Some(predecessors) = preds.get(&block) {
                        for &pred in predecessors {
                            if body.insert(pred) {
                                worklist.push_back(pred);
                            }
                        }
                    }
                }
            }

            entry.body = &entry.body | &body;
        }

        // Step 3: Compute exit blocks for each loop.
        for lp in loops.values_mut() {
            for block in &func.blocks {
                if !lp.body.contains(&block.id) {
                    continue;
                }
                for succ in terminator_successors(&block.term) {
                    if !lp.body.contains(&succ) {
                        lp.exits.insert(succ);
                    }
                }
            }
        }

        // Step 4: Build block → innermost loop mapping.
        // When a block belongs to multiple loops, pick the one with the
        // smallest body (innermost).
        let mut loop_of: HashMap<BlockId, BlockId> = HashMap::new();
        let mut best_size: HashMap<BlockId, usize> = HashMap::new();
        for lp in loops.values() {
            for &block in &lp.body {
                let size = lp.body.len();
                if !best_size.contains_key(&block) || size < best_size[&block] {
                    best_size.insert(block, size);
                    loop_of.insert(block, lp.header);
                }
            }
        }

        LoopInfo { loops, loop_of }
    }

    /// Get the natural loop for a header block.
    pub fn get_loop(&self, header: BlockId) -> Option<&NaturalLoop> {
        self.loops.get(&header)
    }

    /// Get the innermost loop header for a block, if any.
    pub fn loop_header_of(&self, block: BlockId) -> Option<BlockId> {
        self.loop_of.get(&block).copied()
    }

    /// Check if a block is inside any loop.
    pub fn is_in_loop(&self, block: BlockId) -> bool {
        self.loop_of.contains_key(&block)
    }

    /// Number of detected natural loops.
    pub fn loop_count(&self) -> usize {
        self.loops.len()
    }

    /// Iterate over all loops.
    pub fn iter(&self) -> impl Iterator<Item = &NaturalLoop> {
        self.loops.values()
    }

    /// Get all loop headers.
    pub fn headers(&self) -> impl Iterator<Item = BlockId> + '_ {
        self.loops.keys().copied()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

/// Build a predecessor map from the function's blocks.
fn build_predecessor_map(func: &Function) -> HashMap<BlockId, Vec<BlockId>> {
    let mut preds: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for block in &func.blocks {
        for succ in terminator_successors(&block.term) {
            preds.entry(succ).or_default().push(block.id);
        }
    }
    preds
}

#[cfg(test)]
mod tests {
    use super::*;
    use scirs::air::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};
    use scirs::air::ops::{OpId, OpRef};

    /// Build a while-loop CFG:
    ///
    /// ```text
    ///   bb0 (entry)
    ///    |
    ///   bb1 (header) <─┐  back-edge
    ///   / \            │
    ///  bb2  bb3 (exit) │
    ///   │              │
    ///   └──────────────┘
    /// ```
    fn while_loop_function() -> Function {
        let mut func = Function::new(FunctionId("while_loop".into()), true);

        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.term = Terminator::Jump(BlockId(1));

        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term =
            Terminator::Branch { cond: OpRef(OpId(0)), then_bb: BlockId(2), else_bb: BlockId(3) };

        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::Jump(BlockId(1)); // back-edge

        let mut bb3 = BasicBlock::new(BlockId(3));
        bb3.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2, bb3];
        func
    }

    #[test]
    fn test_loop_detection() {
        let func = while_loop_function();
        let dom = DomTree::build(&func).unwrap();
        let info = LoopInfo::build(&func, &dom);

        assert_eq!(info.loop_count(), 1);

        let lp = info.get_loop(BlockId(1)).unwrap();
        assert_eq!(lp.header, BlockId(1));
        assert!(lp.body.contains(&BlockId(1)));
        assert!(lp.body.contains(&BlockId(2)));
        assert!(!lp.body.contains(&BlockId(0)));
        assert!(!lp.body.contains(&BlockId(3)));
        assert!(lp.exits.contains(&BlockId(3)));
        assert_eq!(lp.back_edge_tails, vec![BlockId(2)]);
    }

    #[test]
    fn test_is_in_loop() {
        let func = while_loop_function();
        let dom = DomTree::build(&func).unwrap();
        let info = LoopInfo::build(&func, &dom);

        assert!(info.is_in_loop(BlockId(1)));
        assert!(info.is_in_loop(BlockId(2)));
        assert!(!info.is_in_loop(BlockId(0)));
        assert!(!info.is_in_loop(BlockId(3)));
    }

    #[test]
    fn test_no_loops() {
        // Linear CFG: bb0 → bb1 → bb2 (exit). No back-edges.
        let mut func = Function::new(FunctionId("no_loop".into()), true);

        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.term = Terminator::Jump(BlockId(1));
        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term = Terminator::Jump(BlockId(2));
        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2];

        let dom = DomTree::build(&func).unwrap();
        let info = LoopInfo::build(&func, &dom);
        assert_eq!(info.loop_count(), 0);
    }

    #[test]
    fn test_nested_loops() {
        // Outer loop: bb1 → bb2 → bb3 → bb1
        // Inner loop: bb2 → bb3 → bb2
        //
        // ```text
        //   bb0
        //    │
        //   bb1 ◄──────────┐ (outer back-edge)
        //    │              │
        //   bb2 ◄─────┐    │ (inner back-edge)
        //    │         │    │
        //   bb3 ───┬───┘   │
        //    │     │        │
        //   bb4 ───┘────────┘
        //    │   (bb4→bb1 outer back, bb3→bb2 inner back)
        //   bb5 (exit)
        // ```
        //
        // We'll use: bb0→bb1, bb1→bb2 (branch with exit bb5),
        // bb2→bb3, bb3→bb2(back) or bb4, bb4→bb1(back) or bb5
        let mut func = Function::new(FunctionId("nested".into()), true);

        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.term = Terminator::Jump(BlockId(1));

        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term = Terminator::Jump(BlockId(2));

        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::Jump(BlockId(3));

        let mut bb3 = BasicBlock::new(BlockId(3));
        bb3.term = Terminator::Branch {
            cond: OpRef(OpId(0)),
            then_bb: BlockId(2), // inner back-edge
            else_bb: BlockId(4),
        };

        let mut bb4 = BasicBlock::new(BlockId(4));
        bb4.term = Terminator::Branch {
            cond: OpRef(OpId(1)),
            then_bb: BlockId(1), // outer back-edge
            else_bb: BlockId(5),
        };

        let mut bb5 = BasicBlock::new(BlockId(5));
        bb5.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2, bb3, bb4, bb5];

        let dom = DomTree::build(&func).unwrap();
        let info = LoopInfo::build(&func, &dom);

        assert_eq!(info.loop_count(), 2);

        // Inner loop: header = bb2, body = {bb2, bb3}.
        let inner = info.get_loop(BlockId(2)).unwrap();
        assert!(inner.body.contains(&BlockId(2)));
        assert!(inner.body.contains(&BlockId(3)));
        assert!(!inner.body.contains(&BlockId(4)));

        // Outer loop: header = bb1, body includes bb1, bb2, bb3, bb4.
        let outer = info.get_loop(BlockId(1)).unwrap();
        assert!(outer.body.contains(&BlockId(1)));
        assert!(outer.body.contains(&BlockId(2)));
        assert!(outer.body.contains(&BlockId(3)));
        assert!(outer.body.contains(&BlockId(4)));

        // bb3 is in the inner loop (smallest body).
        assert_eq!(info.loop_header_of(BlockId(3)), Some(BlockId(2)));
        // bb4 is in the outer loop only.
        assert_eq!(info.loop_header_of(BlockId(4)), Some(BlockId(1)));
    }
}
