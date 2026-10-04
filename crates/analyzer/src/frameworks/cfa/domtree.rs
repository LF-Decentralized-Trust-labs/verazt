//! Dominator Trees
//!
//! Computes dominator trees over BIR per-function CFGs using the iterative
//! algorithm of Cooper, Harvey and Kennedy ("A Simple, Fast Dominance
//! Algorithm"), as implemented by `petgraph::algo::dominators::simple_fast`.
//!
//! # Outputs
//!
//! - [`DomTree`]: maps each block to its immediate dominator

use petgraph::algo::dominators;
use petgraph::graph::{DiGraph, NodeIndex};
use scirs::bir::cfg::{BlockId, Function};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// DomTree
// ═══════════════════════════════════════════════════════════════════

/// Immediate-dominator tree for a single function's CFG.
///
/// `idom[block]` is the immediate dominator of `block`.
/// The entry block has no immediate dominator (it is the tree root).
#[derive(Debug, Clone)]
pub struct DomTree {
    /// Immediate dominator map: block → idom.
    idom: HashMap<BlockId, BlockId>,
    /// The root (entry) block.
    root: BlockId,
}

impl DomTree {
    /// Compute the dominator tree for a function's CFG.
    ///
    /// Returns `None` if the function has no basic blocks.
    pub fn build(func: &Function) -> Option<Self> {
        if func.blocks.is_empty() {
            return None;
        }

        let (graph, block_to_node, node_to_block) = build_forward_graph(func);
        let entry_block = func.blocks[0].id;
        let entry_node = block_to_node[&entry_block];

        let doms = dominators::simple_fast(&graph, entry_node);

        let mut idom = HashMap::new();
        for block in &func.blocks {
            if block.id == entry_block {
                continue;
            }
            let node = block_to_node[&block.id];
            if let Some(dom_node) = doms.immediate_dominator(node) {
                idom.insert(block.id, node_to_block[&dom_node]);
            }
        }

        Some(DomTree { idom, root: entry_block })
    }

    /// Get the immediate dominator of a block.
    pub fn idom(&self, block: BlockId) -> Option<BlockId> {
        self.idom.get(&block).copied()
    }

    /// The root (entry) block.
    pub fn root(&self) -> BlockId {
        self.root
    }

    /// Check whether `a` dominates `b` (i.e., every path from entry to
    /// `b` must pass through `a`).
    ///
    /// A block trivially dominates itself.
    pub fn dominates(&self, a: BlockId, b: BlockId) -> bool {
        if a == b {
            return true;
        }
        let mut cur = b;
        while let Some(parent) = self.idom.get(&cur) {
            if *parent == a {
                return true;
            }
            cur = *parent;
        }
        false
    }

}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

/// Build a petgraph DiGraph from a function's blocks (forward edges).
fn build_forward_graph(
    func: &Function,
) -> (DiGraph<BlockId, ()>, HashMap<BlockId, NodeIndex>, HashMap<NodeIndex, BlockId>) {
    let mut graph = DiGraph::<BlockId, ()>::new();
    let mut block_to_node: HashMap<BlockId, NodeIndex> = HashMap::new();
    let mut node_to_block: HashMap<NodeIndex, BlockId> = HashMap::new();

    for block in &func.blocks {
        let node = graph.add_node(block.id);
        block_to_node.insert(block.id, node);
        node_to_block.insert(node, block.id);
    }

    for block in &func.blocks {
        let from = block_to_node[&block.id];
        for succ_id in block.term.successors() {
            if let Some(&to) = block_to_node.get(&succ_id) {
                graph.add_edge(from, to, ());
            }
        }
    }

    (graph, block_to_node, node_to_block)
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use scirs::bir::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};

    /// Build a diamond CFG:
    ///
    /// ```text
    ///     bb0
    ///    /   \
    ///  bb1   bb2
    ///    \   /
    ///     bb3 (exit)
    /// ```
    fn diamond_function() -> Function {
        let mut func = Function::new(FunctionId("diamond".into()), true);

        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.term = Terminator::branch(
            scirs::bir::ops::OpRef(scirs::bir::ops::OpId(0)),
            BlockId(1),
            BlockId(2),
        );

        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term = Terminator::jump(BlockId(3));

        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::jump(BlockId(3));

        let mut bb3 = BasicBlock::new(BlockId(3));
        bb3.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2, bb3];
        func
    }

    #[test]
    fn test_domtree_diamond() {
        let func = diamond_function();
        let dom = DomTree::build(&func).unwrap();

        // bb0 is root, dominates everything.
        assert_eq!(dom.root(), BlockId(0));
        assert!(dom.dominates(BlockId(0), BlockId(1)));
        assert!(dom.dominates(BlockId(0), BlockId(2)));
        assert!(dom.dominates(BlockId(0), BlockId(3)));

        // bb1/bb2 do NOT dominate bb3 (there are two paths).
        assert!(!dom.dominates(BlockId(1), BlockId(3)));
        assert!(!dom.dominates(BlockId(2), BlockId(3)));

        // Immediate dominators.
        assert_eq!(dom.idom(BlockId(1)), Some(BlockId(0)));
        assert_eq!(dom.idom(BlockId(2)), Some(BlockId(0)));
        assert_eq!(dom.idom(BlockId(3)), Some(BlockId(0)));
    }

    #[test]
    fn test_empty_function() {
        let func = Function::new(FunctionId("empty".into()), false);
        assert!(DomTree::build(&func).is_none());
    }

    #[test]
    fn test_linear_chain() {
        let mut func = Function::new(FunctionId("chain".into()), true);

        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.term = Terminator::jump(BlockId(1));

        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term = Terminator::jump(BlockId(2));

        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2];

        let dom = DomTree::build(&func).unwrap();
        assert!(dom.dominates(BlockId(0), BlockId(2)));
        assert!(dom.dominates(BlockId(1), BlockId(2)));
        assert!(!dom.dominates(BlockId(2), BlockId(0)));
    }
}
