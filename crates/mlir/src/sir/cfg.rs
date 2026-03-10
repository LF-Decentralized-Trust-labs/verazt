//! Control Flow Graph types for SIR.
//!
//! These types represent a function-level control flow graph built from
//! SIR statements.  They mirror the AIR CFG types in `crate::air::cfg`
//! but operate at the SIR (source-level IR) layer.

use crate::sir::{Expr, Stmt};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BasicBlockId(pub usize);

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// Terminator instruction for a basic block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    /// Unconditional jump to a block.
    Jump(BasicBlockId),

    /// Conditional branch: if condition then true_block else false_block.
    Branch {
        condition: Expr,
        true_block: BasicBlockId,
        false_block: BasicBlockId,
    },

    /// Return from function.
    Return,

    /// Revert/throw.
    Revert,

    /// Unreachable code.
    Unreachable,
}

/// A basic block in the control flow graph.
///
/// A basic block is a sequence of instructions with:
/// - Single entry point (first instruction)
/// - Single exit point (terminator)
/// - No branches except at the end
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier for this block.
    pub id: BasicBlockId,

    /// Statements in this block (excluding terminator).
    pub stmts: Vec<Stmt>,

    /// Terminator instruction.
    pub terminator: Terminator,

    /// Predecessor blocks.
    pub predecessors: Vec<BasicBlockId>,

    /// Successor blocks (derived from terminator).
    pub successors: Vec<BasicBlockId>,
}

impl BasicBlock {
    /// Create a new basic block.
    pub fn new(id: BasicBlockId, terminator: Terminator) -> Self {
        Self {
            id,
            stmts: Vec::new(),
            terminator,
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    /// Add a statement to this block.
    pub fn add_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    /// Compute successors from terminator.
    pub fn compute_successors(&mut self) {
        self.successors = match &self.terminator {
            Terminator::Jump(target) => vec![*target],
            Terminator::Branch { true_block, false_block, .. } => {
                vec![*true_block, *false_block]
            }
            Terminator::Return | Terminator::Revert | Terminator::Unreachable => {
                vec![]
            }
        };
    }
}

/// Control flow graph for a function.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Function this CFG represents.
    pub function_name: String,

    /// All basic blocks indexed by ID.
    pub blocks: HashMap<BasicBlockId, BasicBlock>,

    /// Entry block ID.
    pub entry: BasicBlockId,

    /// Exit block ID (for functions with explicit returns).
    pub exit: Option<BasicBlockId>,
}

impl ControlFlowGraph {
    /// Create a new empty CFG.
    pub fn new(function_name: String, entry: BasicBlockId) -> Self {
        Self { function_name, blocks: HashMap::new(), entry, exit: None }
    }

    /// Add a basic block to the CFG.
    pub fn add_block(&mut self, block: BasicBlock) {
        self.blocks.insert(block.id, block);
    }

    /// Get a basic block by ID.
    pub fn get_block(&self, id: BasicBlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    /// Get a mutable basic block by ID.
    pub fn get_block_mut(&mut self, id: BasicBlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }

    /// Compute all predecessor relationships.
    pub fn compute_predecessors(&mut self) {
        // Clear existing predecessors
        for block in self.blocks.values_mut() {
            block.predecessors.clear();
        }

        // Recompute from successors
        let edges: Vec<(BasicBlockId, BasicBlockId)> = self
            .blocks
            .values()
            .flat_map(|block| block.successors.iter().map(move |&succ| (block.id, succ)))
            .collect();

        for (pred, succ) in edges {
            if let Some(block) = self.blocks.get_mut(&succ) {
                if !block.predecessors.contains(&pred) {
                    block.predecessors.push(pred);
                }
            }
        }
    }

    /// Get all block IDs in the CFG.
    pub fn block_ids(&self) -> Vec<BasicBlockId> {
        let mut ids: Vec<_> = self.blocks.keys().copied().collect();
        ids.sort();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sir::{BoolLit, Lit};

    #[test]
    fn test_basic_block_creation() {
        let id = BasicBlockId(0);
        let terminator = Terminator::Return;
        let block = BasicBlock::new(id, terminator);

        assert_eq!(block.id, id);
        assert_eq!(block.stmts.len(), 0);
        assert_eq!(block.successors.len(), 0);
    }

    #[test]
    fn test_basic_block_successors() {
        let mut block = BasicBlock::new(BasicBlockId(0), Terminator::Jump(BasicBlockId(1)));

        block.compute_successors();
        assert_eq!(block.successors, vec![BasicBlockId(1)]);
    }

    #[test]
    fn test_cfg_creation() {
        let entry = BasicBlockId(0);
        let cfg = ControlFlowGraph::new("test_function".to_string(), entry);

        assert_eq!(cfg.function_name, "test_function");
        assert_eq!(cfg.entry, entry);
        assert_eq!(cfg.blocks.len(), 0);
    }

    #[test]
    fn test_cfg_predecessors() {
        let mut cfg = ControlFlowGraph::new("test".to_string(), BasicBlockId(0));

        // Create blocks: 0 -> 1, 0 -> 2
        let mut block0 = BasicBlock::new(
            BasicBlockId(0),
            Terminator::Branch {
                condition: Expr::Lit(Lit::Bool(BoolLit { value: true, span: None })),
                true_block: BasicBlockId(1),
                false_block: BasicBlockId(2),
            },
        );
        block0.compute_successors();

        let mut block1 = BasicBlock::new(BasicBlockId(1), Terminator::Return);
        block1.compute_successors();

        let mut block2 = BasicBlock::new(BasicBlockId(2), Terminator::Return);
        block2.compute_successors();

        cfg.add_block(block0);
        cfg.add_block(block1);
        cfg.add_block(block2);
        cfg.compute_predecessors();

        // Check predecessors
        let block1_preds = &cfg.get_block(BasicBlockId(1)).unwrap().predecessors;
        assert_eq!(block1_preds.len(), 1);
        assert!(block1_preds.contains(&BasicBlockId(0)));

        let block2_preds = &cfg.get_block(BasicBlockId(2)).unwrap().predecessors;
        assert_eq!(block2_preds.len(), 1);
        assert!(block2_preds.contains(&BasicBlockId(0)));
    }
}
