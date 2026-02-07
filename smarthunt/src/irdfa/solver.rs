use crate::irdfa::cfg::{BasicBlock, BasicBlockId, ControlFlowGraph};
use crate::irdfa::lattice::Lattice;
use solidity::ir::Stmt;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

/// Analysis direction
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// Information flows with control flow (entry to exit)
    Forward,
    /// Information flows against control flow (exit to entry)
    Backward,
}

/// Transfer function for statements
pub trait Transfer<L: Lattice>: Send + Sync {
    /// Apply transfer function to a single statement
    fn transfer_stmt(&self, stmt: &Stmt, fact: &L) -> L;

    /// Apply transfer function to a block (default: sequential transfer)
    fn transfer_block(&self, block: &BasicBlock, fact: L, direction: Direction) -> L {
        match direction {
            Direction::Forward => {
                let mut current = fact;
                for stmt in &block.stmts {
                    current = self.transfer_stmt(stmt, &current);
                }
                current
            }
            Direction::Backward => {
                let mut current = fact;
                for stmt in block.stmts.iter().rev() {
                    current = self.transfer_stmt(stmt, &current);
                }
                current
            }
        }
    }
}

/// Result of data flow analysis
#[derive(Debug, Clone)]
pub struct DataFlowResult<L: Lattice> {
    /// Fact at entry of each block
    pub block_entry: HashMap<BasicBlockId, L>,
    /// Fact at exit of each block
    pub block_exit: HashMap<BasicBlockId, L>,
    /// Number of iterations until fixpoint
    pub iterations: usize,
    /// Whether fixpoint was reached
    pub converged: bool,
}

impl<L: Lattice> DataFlowResult<L> {
    /// Get fact at entry of a block
    pub fn entry(&self, block_id: BasicBlockId) -> Option<&L> {
        self.block_entry.get(&block_id)
    }

    /// Get fact at exit of a block
    pub fn exit(&self, block_id: BasicBlockId) -> Option<&L> {
        self.block_exit.get(&block_id)
    }
}

/// Generic worklist-based data flow solver
pub struct DataFlowSolver<L: Lattice, T: Transfer<L>> {
    direction: Direction,
    transfer: T,
    max_iterations: usize,
    _phantom: PhantomData<L>,
}

impl<L: Lattice, T: Transfer<L>> DataFlowSolver<L, T> {
    /// Create a new solver
    pub fn new(direction: Direction, transfer: T) -> Self {
        Self {
            direction,
            transfer,
            max_iterations: 1000,
            _phantom: PhantomData,
        }
    }

    /// Set maximum iterations before giving up
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Solve data flow equations on CFG
    pub fn solve(&self, cfg: &ControlFlowGraph, initial: L) -> DataFlowResult<L> {
        match self.direction {
            Direction::Forward => self.solve_forward(cfg, initial),
            Direction::Backward => self.solve_backward(cfg, initial),
        }
    }

    /// Solve forward data flow analysis
    fn solve_forward(&self, cfg: &ControlFlowGraph, initial: L) -> DataFlowResult<L> {
        let mut block_entry: HashMap<BasicBlockId, L> = HashMap::new();
        let mut block_exit: HashMap<BasicBlockId, L> = HashMap::new();

        // Initialize all blocks to bottom
        for &id in cfg.blocks.keys() {
            block_entry.insert(id, L::bottom());
            block_exit.insert(id, L::bottom());
        }

        // Set entry block to initial value
        block_entry.insert(cfg.entry, initial);

        // Worklist algorithm using reverse postorder for efficiency
        let mut worklist: VecDeque<BasicBlockId> = cfg.reverse_postorder.iter().copied().collect();
        let mut iterations = 0;

        while let Some(block_id) = worklist.pop_front() {
            iterations += 1;

            if iterations > self.max_iterations {
                return DataFlowResult {
                    block_entry,
                    block_exit,
                    iterations,
                    converged: false,
                };
            }

            let block = match cfg.blocks.get(&block_id) {
                Some(b) => b,
                None => continue,
            };

            // Compute entry fact by joining predecessors' exit facts
            let entry = if block_id == cfg.entry {
                block_entry.get(&block_id).unwrap().clone()
            } else {
                let mut result = L::bottom();
                for &pred in &block.predecessors {
                    if let Some(pred_exit) = block_exit.get(&pred) {
                        result = result.join(pred_exit);
                    }
                }
                result
            };

            // Apply transfer function
            let exit = self.transfer.transfer_block(block, entry.clone(), Direction::Forward);

            // Check for changes
            let old_exit = block_exit.get(&block_id).unwrap();
            if &exit != old_exit {
                block_entry.insert(block_id, entry);
                block_exit.insert(block_id, exit);

                // Add successors to worklist if not already present
                for &succ in &block.successors {
                    if !worklist.contains(&succ) {
                        worklist.push_back(succ);
                    }
                }
            }
        }

        DataFlowResult {
            block_entry,
            block_exit,
            iterations,
            converged: true,
        }
    }

    /// Solve backward data flow analysis
    fn solve_backward(&self, cfg: &ControlFlowGraph, initial: L) -> DataFlowResult<L> {
        let mut block_entry: HashMap<BasicBlockId, L> = HashMap::new();
        let mut block_exit: HashMap<BasicBlockId, L> = HashMap::new();

        // Initialize all blocks to bottom
        for &id in cfg.blocks.keys() {
            block_entry.insert(id, L::bottom());
            block_exit.insert(id, L::bottom());
        }

        // Set exit blocks to initial value
        for &exit_id in &cfg.exit_blocks {
            block_exit.insert(exit_id, initial.clone());
        }

        // Worklist algorithm in reverse order
        let mut worklist: VecDeque<BasicBlockId> = cfg.reverse_postorder.iter().rev().copied().collect();
        let mut iterations = 0;

        while let Some(block_id) = worklist.pop_front() {
            iterations += 1;

            if iterations > self.max_iterations {
                return DataFlowResult {
                    block_entry,
                    block_exit,
                    iterations,
                    converged: false,
                };
            }

            let block = match cfg.blocks.get(&block_id) {
                Some(b) => b,
                None => continue,
            };

            // Compute exit fact by joining successors' entry facts
            let exit = if cfg.exit_blocks.contains(&block_id) {
                block_exit.get(&block_id).unwrap().clone()
            } else {
                let mut result = L::bottom();
                for &succ in &block.successors {
                    if let Some(succ_entry) = block_entry.get(&succ) {
                        result = result.join(succ_entry);
                    }
                }
                result
            };

            // Apply transfer function in reverse
            let entry = self.transfer.transfer_block(block, exit.clone(), Direction::Backward);

            // Check for changes
            let old_entry = block_entry.get(&block_id).unwrap();
            if &entry != old_entry {
                block_entry.insert(block_id, entry);
                block_exit.insert(block_id, exit);

                // Add predecessors to worklist if not already present
                for &pred in &block.predecessors {
                    if !worklist.contains(&pred) {
                        worklist.push_back(pred);
                    }
                }
            }
        }

        DataFlowResult {
            block_entry,
            block_exit,
            iterations,
            converged: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::irdfa::cfg::Terminator;
    use crate::irdfa::lattice::PowerSetLattice;

    struct TestTransfer;

    impl Transfer<PowerSetLattice<i32>> for TestTransfer {
        fn transfer_stmt(&self, _stmt: &Stmt, fact: &PowerSetLattice<i32>) -> PowerSetLattice<i32> {
            // Simple test: just pass through
            fact.clone()
        }
    }

    #[test]
    fn test_forward_analysis() {
        let mut cfg = ControlFlowGraph::new("test".to_string(), BasicBlockId(0));

        let mut block0 = BasicBlock::new(BasicBlockId(0), Terminator::Jump(BasicBlockId(1)));
        block0.compute_successors();

        let block1 = BasicBlock::new(BasicBlockId(1), Terminator::Return);

        cfg.add_block(block0);
        cfg.add_block(block1);
        cfg.compute_metadata();

        let solver = DataFlowSolver::new(Direction::Forward, TestTransfer);
        let mut initial = PowerSetLattice::new();
        initial.insert(42);

        let result = solver.solve(&cfg, initial);
        assert!(result.converged);
        assert!(result.iterations > 0);
    }
}
