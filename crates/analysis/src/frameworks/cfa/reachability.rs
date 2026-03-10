//! Reachability Queries over AIR Per-Function CFGs
//!
//! Provides BFS/DFS-based reachability queries:
//!
//! - Can block A reach block B along CFG edges?
//! - Compute the set of all blocks reachable from a starting block.
//!
//! # Detector integration
//!
//! - **Reentrancy**: check whether an external call can *reach* a storage write
//!   on any path — a direct reachability query, no full DFA pass required.

use mlir::air::cfg::{AIRFunction, BlockId, Terminator};
use std::collections::{HashMap, HashSet, VecDeque};

// ═══════════════════════════════════════════════════════════════════
// ReachabilitySet
// ═══════════════════════════════════════════════════════════════════

/// The set of blocks reachable from a given starting block.
#[derive(Debug, Clone)]
pub struct ReachabilitySet {
    /// Blocks reachable from the start (inclusive).
    reachable: HashSet<BlockId>,
    /// The start block.
    start: BlockId,
}

impl ReachabilitySet {
    /// Compute all blocks reachable from `start` in `func`'s CFG using BFS.
    pub fn forward(func: &AIRFunction, start: BlockId) -> Self {
        let succ_map = build_successor_map(func);
        let reachable = bfs(&succ_map, start);
        ReachabilitySet { reachable, start }
    }

    /// Compute all blocks that can *reach* `target` by backwards traversal.
    pub fn backward(func: &AIRFunction, target: BlockId) -> Self {
        let pred_map = build_predecessor_map(func);
        let reachable = bfs(&pred_map, target);
        ReachabilitySet { reachable, start: target }
    }

    /// Check if `block` is reachable from the start.
    pub fn contains(&self, block: BlockId) -> bool {
        self.reachable.contains(&block)
    }

    /// The starting block.
    pub fn start(&self) -> BlockId {
        self.start
    }

    /// Number of reachable blocks.
    pub fn len(&self) -> usize {
        self.reachable.len()
    }

    /// Check if the set is empty (never happens — start is always reachable).
    pub fn is_empty(&self) -> bool {
        self.reachable.is_empty()
    }

    /// Iterate over all reachable block IDs.
    pub fn iter(&self) -> impl Iterator<Item = BlockId> + '_ {
        self.reachable.iter().copied()
    }

    /// Get the underlying set.
    pub fn blocks(&self) -> &HashSet<BlockId> {
        &self.reachable
    }
}

// ═══════════════════════════════════════════════════════════════════
// Free functions
// ═══════════════════════════════════════════════════════════════════

/// Quick point-to-point reachability check: can `from` reach `to`?
///
/// Terminates early as soon as `to` is found. More efficient than
/// building the full `ReachabilitySet` when you only need a yes/no answer.
pub fn can_reach(func: &AIRFunction, from: BlockId, to: BlockId) -> bool {
    if from == to {
        return true;
    }
    let succ_map = build_successor_map(func);
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(from);
    queue.push_back(from);

    while let Some(block) = queue.pop_front() {
        if let Some(succs) = succ_map.get(&block) {
            for &succ in succs {
                if succ == to {
                    return true;
                }
                if visited.insert(succ) {
                    queue.push_back(succ);
                }
            }
        }
    }
    false
}

/// Check if `from` can reach `to` going backwards through predecessors.
pub fn can_reach_backward(func: &AIRFunction, from: BlockId, to: BlockId) -> bool {
    if from == to {
        return true;
    }
    let pred_map = build_predecessor_map(func);
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(from);
    queue.push_back(from);

    while let Some(block) = queue.pop_front() {
        if let Some(preds) = pred_map.get(&block) {
            for &pred in preds {
                if pred == to {
                    return true;
                }
                if visited.insert(pred) {
                    queue.push_back(pred);
                }
            }
        }
    }
    false
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn terminator_successors(term: &Terminator) -> Vec<BlockId> {
    match term {
        Terminator::Jump(bb) => vec![*bb],
        Terminator::Branch { then_bb, else_bb, .. } => vec![*then_bb, *else_bb],
        Terminator::TxnExit { .. } | Terminator::Unreachable => vec![],
    }
}

fn build_successor_map(func: &AIRFunction) -> HashMap<BlockId, Vec<BlockId>> {
    let mut map: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for block in &func.blocks {
        map.insert(block.id, terminator_successors(&block.term));
    }
    map
}

fn build_predecessor_map(func: &AIRFunction) -> HashMap<BlockId, Vec<BlockId>> {
    let mut preds: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for block in &func.blocks {
        for succ in terminator_successors(&block.term) {
            preds.entry(succ).or_default().push(block.id);
        }
    }
    preds
}

fn bfs(adj: &HashMap<BlockId, Vec<BlockId>>, start: BlockId) -> HashSet<BlockId> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);

    while let Some(block) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&block) {
            for &next in neighbors {
                if visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
    }
    visited
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlir::air::cfg::{AIRFunction, BasicBlock, BlockId, FunctionId, Terminator};
    use mlir::air::ops::{OpId, OpRef};

    /// Diamond CFG: bb0 → {bb1, bb2} → bb3 (exit)
    fn diamond_function() -> AIRFunction {
        let mut func = AIRFunction::new(FunctionId("diamond".into()), true);

        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.term =
            Terminator::Branch { cond: OpRef(OpId(0)), then_bb: BlockId(1), else_bb: BlockId(2) };
        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term = Terminator::Jump(BlockId(3));
        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::Jump(BlockId(3));
        let mut bb3 = BasicBlock::new(BlockId(3));
        bb3.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2, bb3];
        func
    }

    #[test]
    fn test_forward_reachability() {
        let func = diamond_function();
        let reach = ReachabilitySet::forward(&func, BlockId(0));

        assert!(reach.contains(BlockId(0)));
        assert!(reach.contains(BlockId(1)));
        assert!(reach.contains(BlockId(2)));
        assert!(reach.contains(BlockId(3)));
        assert_eq!(reach.len(), 4);
    }

    #[test]
    fn test_forward_from_branch() {
        let func = diamond_function();
        let reach = ReachabilitySet::forward(&func, BlockId(1));

        assert!(reach.contains(BlockId(1)));
        assert!(reach.contains(BlockId(3)));
        assert!(!reach.contains(BlockId(0)));
        assert!(!reach.contains(BlockId(2)));
    }

    #[test]
    fn test_backward_reachability() {
        let func = diamond_function();
        let reach = ReachabilitySet::backward(&func, BlockId(3));

        assert!(reach.contains(BlockId(3)));
        assert!(reach.contains(BlockId(1)));
        assert!(reach.contains(BlockId(2)));
        assert!(reach.contains(BlockId(0)));
    }

    #[test]
    fn test_can_reach() {
        let func = diamond_function();

        assert!(can_reach(&func, BlockId(0), BlockId(3)));
        assert!(can_reach(&func, BlockId(1), BlockId(3)));
        assert!(!can_reach(&func, BlockId(3), BlockId(0)));
        assert!(!can_reach(&func, BlockId(1), BlockId(2)));
    }

    #[test]
    fn test_can_reach_backward() {
        let func = diamond_function();

        assert!(can_reach_backward(&func, BlockId(3), BlockId(0)));
        assert!(!can_reach_backward(&func, BlockId(0), BlockId(3)));
    }

    #[test]
    fn test_self_reachability() {
        let func = diamond_function();
        assert!(can_reach(&func, BlockId(0), BlockId(0)));
    }
}
