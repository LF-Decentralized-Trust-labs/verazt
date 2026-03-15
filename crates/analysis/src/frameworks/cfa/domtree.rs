//! Dominator and Post-Dominator Trees
//!
//! Computes dominator trees over AIR per-function CFGs using the
//! Lengauer-Tarjan algorithm provided by `petgraph`.
//!
//! # Outputs
//!
//! - [`DomTree`] — maps each block to its immediate dominator
//! - [`PostDomTree`] — maps each block to its immediate post-dominator
//!
//! # Detector integration
//!
//! - **Access control**: verify that a signer guard *dominates* every storage
//!   write (eliminates false negatives from guards inside conditional
//!   branches).
//! - **Taint precision**: dominance-aware def-use annotation in `dfa/`.

use scirs::air::cfg::{BlockId, Function, Terminator};
use petgraph::algo::dominators;
use petgraph::graph::{DiGraph, NodeIndex};
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

    /// Iterate over all blocks in the tree with their immediate domonators.
    pub fn iter(&self) -> impl Iterator<Item = (BlockId, BlockId)> + '_ {
        self.idom.iter().map(|(&block, &dom)| (block, dom))
    }
}

// ═══════════════════════════════════════════════════════════════════
// PostDomTree
// ═══════════════════════════════════════════════════════════════════

/// Immediate post-dominator tree for a single function's CFG.
///
/// `ipdom[block]` is the immediate post-dominator of `block`.
/// Exit blocks (or a virtual exit) serve as the tree root.
#[derive(Debug, Clone)]
pub struct PostDomTree {
    /// Immediate post-dominator map: block → ipdom.
    ipdom: HashMap<BlockId, BlockId>,
    /// The virtual exit block id (if created).
    virtual_exit: BlockId,
}

impl PostDomTree {
    /// Compute the post-dominator tree.
    ///
    /// Adds a virtual exit node that all exit blocks (TxnExit,
    /// Unreachable) flow to, then computes dominators on the reversed
    /// graph from that virtual exit.
    ///
    /// Returns `None` if the function has no basic blocks.
    pub fn build(func: &Function) -> Option<Self> {
        if func.blocks.is_empty() {
            return None;
        }

        let virtual_exit_id = BlockId(func.blocks.iter().map(|b| b.id.0).max().unwrap_or(0) + 1);

        // Build the *reverse* graph: edges go from successor to predecessor.
        let mut graph = DiGraph::<BlockId, ()>::new();
        let mut block_to_node: HashMap<BlockId, NodeIndex> = HashMap::new();
        let mut node_to_block: HashMap<NodeIndex, BlockId> = HashMap::new();

        // Add real blocks.
        for block in &func.blocks {
            let node = graph.add_node(block.id);
            block_to_node.insert(block.id, node);
            node_to_block.insert(node, block.id);
        }

        // Add virtual exit.
        let exit_node = graph.add_node(virtual_exit_id);
        block_to_node.insert(virtual_exit_id, exit_node);
        node_to_block.insert(exit_node, virtual_exit_id);

        // Connect exit blocks to virtual exit, then reverse all edges.
        for block in &func.blocks {
            let successors = terminator_successors(&block.term);
            if successors.is_empty() {
                // This is an exit block → connect virtual exit → this block (reversed).
                graph.add_edge(exit_node, block_to_node[&block.id], ());
            }
            for succ_id in successors {
                if let Some(&succ_node) = block_to_node.get(&succ_id) {
                    // Add reversed edge: succ → pred.
                    graph.add_edge(succ_node, block_to_node[&block.id], ());
                }
            }
        }

        let doms = dominators::simple_fast(&graph, exit_node);

        let mut ipdom = HashMap::new();
        for block in &func.blocks {
            let node = block_to_node[&block.id];
            if let Some(dom_node) = doms.immediate_dominator(node) {
                ipdom.insert(block.id, node_to_block[&dom_node]);
            }
        }

        Some(PostDomTree { ipdom, virtual_exit: virtual_exit_id })
    }

    /// Get the immediate post-dominator of a block.
    pub fn ipdom(&self, block: BlockId) -> Option<BlockId> {
        self.ipdom.get(&block).copied()
    }

    /// The virtual exit block id.
    pub fn virtual_exit(&self) -> BlockId {
        self.virtual_exit
    }

    /// Check whether `a` post-dominates `b` (every path from `b` to exit
    /// passes through `a`).
    pub fn post_dominates(&self, a: BlockId, b: BlockId) -> bool {
        if a == b {
            return true;
        }
        let mut cur = b;
        while let Some(parent) = self.ipdom.get(&cur) {
            if *parent == a {
                return true;
            }
            if *parent == self.virtual_exit {
                return false;
            }
            cur = *parent;
        }
        false
    }

    /// Iterate over all blocks with their immediate post-dominators.
    pub fn iter(&self) -> impl Iterator<Item = (BlockId, BlockId)> + '_ {
        self.ipdom.iter().map(|(&block, &dom)| (block, dom))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

/// Extract successor block IDs from a terminator.
pub(super) fn terminator_successors(term: &Terminator) -> Vec<BlockId> {
    match term {
        Terminator::Jump(bb) => vec![*bb],
        Terminator::Branch { then_bb, else_bb, .. } => vec![*then_bb, *else_bb],
        Terminator::TxnExit { .. } | Terminator::Unreachable => vec![],
    }
}

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
        for succ_id in terminator_successors(&block.term) {
            if let Some(&to) = block_to_node.get(&succ_id) {
                graph.add_edge(from, to, ());
            }
        }
    }

    (graph, block_to_node, node_to_block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scirs::air::cfg::{BasicBlock, BlockId, Function, FunctionId, Terminator};

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
        bb0.term = Terminator::Branch {
            cond: scirs::air::ops::OpRef(scirs::air::ops::OpId(0)),
            then_bb: BlockId(1),
            else_bb: BlockId(2),
        };

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
    fn test_postdomtree_diamond() {
        let func = diamond_function();
        let pdom = PostDomTree::build(&func).unwrap();

        // bb3 post-dominates bb1 and bb2 (both must reach bb3).
        assert!(pdom.post_dominates(BlockId(3), BlockId(1)));
        assert!(pdom.post_dominates(BlockId(3), BlockId(2)));
        // bb3 post-dominates bb0 as well (all paths reach bb3).
        assert!(pdom.post_dominates(BlockId(3), BlockId(0)));

        // bb1 does NOT post-dominate bb0.
        assert!(!pdom.post_dominates(BlockId(1), BlockId(0)));
    }

    #[test]
    fn test_empty_function() {
        let func = Function::new(FunctionId("empty".into()), false);
        assert!(DomTree::build(&func).is_none());
        assert!(PostDomTree::build(&func).is_none());
    }

    #[test]
    fn test_linear_chain() {
        let mut func = Function::new(FunctionId("chain".into()), true);

        let mut bb0 = BasicBlock::new(BlockId(0));
        bb0.term = Terminator::Jump(BlockId(1));

        let mut bb1 = BasicBlock::new(BlockId(1));
        bb1.term = Terminator::Jump(BlockId(2));

        let mut bb2 = BasicBlock::new(BlockId(2));
        bb2.term = Terminator::TxnExit { reverted: false };

        func.blocks = vec![bb0, bb1, bb2];

        let dom = DomTree::build(&func).unwrap();
        assert!(dom.dominates(BlockId(0), BlockId(2)));
        assert!(dom.dominates(BlockId(1), BlockId(2)));
        assert!(!dom.dominates(BlockId(2), BlockId(0)));
    }
}
