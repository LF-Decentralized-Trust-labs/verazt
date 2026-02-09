//! Core Data Flow Analysis Framework
//!
//! This module provides the generic infrastructure for implementing data flow
//! analyses using the worklist algorithm with lattice-based abstract
//! interpretation.

use crate::analysis::passes::ir::{BasicBlockId, ControlFlowGraph};
use solidity::ir::Stmt;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

/// Error type for data flow analysis.
#[derive(Debug, Error)]
pub enum DataFlowError {
    #[error("Data flow analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Fixpoint not reached after {0} iterations")]
    FixpointNotReached(usize),

    #[error("Invalid CFG: {0}")]
    InvalidCfg(String),
}

/// Result type for data flow operations.
pub type DataFlowResult<T> = Result<T, DataFlowError>;

/// Direction of data flow analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisDirection {
    /// Forward analysis: flow from entry to exit.
    /// Information flows along control flow edges.
    Forward,

    /// Backward analysis: flow from exit to entry.
    /// Information flows against control flow edges.
    Backward,
}

/// Trait for data flow facts (lattice elements).
///
/// A data flow fact represents information about program state at a program
/// point. Facts must form a lattice with bottom, top, meet, and partial order.
pub trait DataFlowFact: Clone + Eq + Debug + Send + Sync {
    /// Bottom element (initial state, no information).
    fn bottom() -> Self;

    /// Top element (unknown/any state, all information).
    fn top() -> Self;

    /// Meet operator (join in lattice).
    /// Combines information from multiple predecessors/successors.
    fn meet(&self, other: &Self) -> Self;

    /// Partial order check: self ⊑ other.
    /// Returns true if self contains less or equal information than other.
    fn less_or_equal(&self, other: &Self) -> bool;
}

/// Transfer function for data flow analysis.
///
/// Defines how statements transform data flow facts.
/// Each analysis implements this trait to specify analysis-specific semantics.
pub trait TransferFunction<F: DataFlowFact>: Send + Sync {
    /// Apply transfer function to a statement.
    ///
    /// Given the fact before the statement and the statement itself,
    /// compute the fact after the statement.
    fn transfer(&self, stmt: &Stmt, fact: &F) -> F;

    /// Transfer function for a basic block.
    ///
    /// Default implementation applies transfer to each statement sequentially.
    fn transfer_block(&self, stmts: &[Stmt], mut fact: F) -> F {
        for stmt in stmts {
            fact = self.transfer(stmt, &fact);
        }
        fact
    }
}

/// Generic data flow solver using the worklist algorithm.
///
/// Computes fixpoint of data flow equations over a control flow graph.
pub struct DataFlowSolver<F: DataFlowFact, T: TransferFunction<F>> {
    /// Direction of analysis (forward or backward).
    direction: AnalysisDirection,

    /// Transfer function for this analysis.
    transfer: T,

    /// Control flow graph to analyze.
    cfg: Arc<ControlFlowGraph>,

    /// Maximum number of iterations before giving up.
    max_iterations: usize,

    /// Phantom data for fact type.
    _phantom: std::marker::PhantomData<F>,
}

impl<F: DataFlowFact, T: TransferFunction<F>> DataFlowSolver<F, T> {
    /// Create a new data flow solver.
    pub fn new(direction: AnalysisDirection, transfer: T, cfg: Arc<ControlFlowGraph>) -> Self {
        Self { direction, transfer, cfg, max_iterations: 1000, _phantom: std::marker::PhantomData }
    }

    /// Set maximum number of iterations.
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Run the worklist algorithm to compute fixpoint.
    ///
    /// Returns a map from BasicBlockId to the data flow fact at the entry
    /// (for forward analysis) or exit (for backward analysis) of each block.
    pub fn solve(&self) -> DataFlowResult<HashMap<BasicBlockId, F>> {
        match self.direction {
            AnalysisDirection::Forward => self.solve_forward(),
            AnalysisDirection::Backward => self.solve_backward(),
        }
    }

    /// Forward data flow analysis.
    ///
    /// For each block:
    /// - in[B] = meet(out[P]) for all predecessors P
    /// - out[B] = transfer(in[B], B)
    fn solve_forward(&self) -> DataFlowResult<HashMap<BasicBlockId, F>> {
        // Initialize: entry block gets bottom, all others get top
        let mut in_facts: HashMap<BasicBlockId, F> = HashMap::new();
        let mut out_facts: HashMap<BasicBlockId, F> = HashMap::new();

        for &block_id in self.cfg.blocks.keys() {
            if block_id == self.cfg.entry {
                in_facts.insert(block_id, F::bottom());
            } else {
                in_facts.insert(block_id, F::top());
            }
            out_facts.insert(block_id, F::top());
        }

        // Worklist: initially contains all blocks
        let mut worklist: VecDeque<BasicBlockId> = self.cfg.blocks.keys().copied().collect();
        let mut worklist_set: HashSet<BasicBlockId> = worklist.iter().copied().collect();

        let mut iterations = 0;

        while let Some(block_id) = worklist.pop_front() {
            worklist_set.remove(&block_id);
            iterations += 1;

            if iterations > self.max_iterations {
                return Err(DataFlowError::FixpointNotReached(self.max_iterations));
            }

            let block = self.cfg.blocks.get(&block_id).ok_or_else(|| {
                DataFlowError::InvalidCfg(format!("Block {:?} not found", block_id))
            })?;

            // Compute in[B] = meet(out[P]) for all predecessors P
            let new_in = if block.predecessors.is_empty() {
                // Entry block or unreachable
                in_facts.get(&block_id).unwrap().clone()
            } else {
                let mut result = F::top();
                for &pred in &block.predecessors {
                    if let Some(pred_out) = out_facts.get(&pred) {
                        result = result.meet(pred_out);
                    }
                }
                result
            };

            // Check if in[B] changed
            let old_in = in_facts.get(&block_id).unwrap();
            if new_in != *old_in {
                in_facts.insert(block_id, new_in.clone());

                // Compute out[B] = transfer(in[B], B)
                let new_out = self.transfer.transfer_block(&block.stmts, new_in);

                // Update out[B] and add successors to worklist
                out_facts.insert(block_id, new_out);

                for &succ in &block.successors {
                    if !worklist_set.contains(&succ) {
                        worklist.push_back(succ);
                        worklist_set.insert(succ);
                    }
                }
            }
        }

        Ok(in_facts)
    }

    /// Backward data flow analysis.
    ///
    /// For each block:
    /// - out[B] = meet(in[S]) for all successors S
    /// - in[B] = transfer(out[B], B)
    fn solve_backward(&self) -> DataFlowResult<HashMap<BasicBlockId, F>> {
        // Initialize: exit block gets bottom, all others get top
        let mut in_facts: HashMap<BasicBlockId, F> = HashMap::new();
        let mut out_facts: HashMap<BasicBlockId, F> = HashMap::new();

        for &block_id in self.cfg.blocks.keys() {
            // For backward analysis, blocks with no successors are exits
            let block = self.cfg.blocks.get(&block_id).unwrap();
            if block.successors.is_empty() {
                out_facts.insert(block_id, F::bottom());
            } else {
                out_facts.insert(block_id, F::top());
            }
            in_facts.insert(block_id, F::top());
        }

        // Worklist: initially contains all blocks
        let mut worklist: VecDeque<BasicBlockId> = self.cfg.blocks.keys().copied().collect();
        let mut worklist_set: HashSet<BasicBlockId> = worklist.iter().copied().collect();

        let mut iterations = 0;

        while let Some(block_id) = worklist.pop_front() {
            worklist_set.remove(&block_id);
            iterations += 1;

            if iterations > self.max_iterations {
                return Err(DataFlowError::FixpointNotReached(self.max_iterations));
            }

            let block = self.cfg.blocks.get(&block_id).ok_or_else(|| {
                DataFlowError::InvalidCfg(format!("Block {:?} not found", block_id))
            })?;

            // Compute out[B] = meet(in[S]) for all successors S
            let new_out = if block.successors.is_empty() {
                // Exit block
                out_facts.get(&block_id).unwrap().clone()
            } else {
                let mut result = F::top();
                for &succ in &block.successors {
                    if let Some(succ_in) = in_facts.get(&succ) {
                        result = result.meet(succ_in);
                    }
                }
                result
            };

            // Check if out[B] changed
            let old_out = out_facts.get(&block_id).unwrap();
            if new_out != *old_out {
                out_facts.insert(block_id, new_out.clone());

                // Compute in[B] = transfer(out[B], B)
                // Note: for backward analysis, we process statements in reverse
                let new_in = self.transfer_block_backward(&block.stmts, new_out);

                // Update in[B] and add predecessors to worklist
                in_facts.insert(block_id, new_in);

                for &pred in &block.predecessors {
                    if !worklist_set.contains(&pred) {
                        worklist.push_back(pred);
                        worklist_set.insert(pred);
                    }
                }
            }
        }

        Ok(out_facts)
    }

    /// Transfer function for backward analysis (processes statements in
    /// reverse).
    fn transfer_block_backward(&self, stmts: &[Stmt], mut fact: F) -> F {
        for stmt in stmts.iter().rev() {
            fact = self.transfer.transfer(stmt, &fact);
        }
        fact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::passes::ir::{BasicBlock, Terminator};

    // Simple test fact: set of integers
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestFact(HashSet<i32>);

    impl DataFlowFact for TestFact {
        fn bottom() -> Self {
            TestFact(HashSet::new())
        }

        fn top() -> Self {
            // For testing, use a special marker
            let mut set = HashSet::new();
            set.insert(-1);
            TestFact(set)
        }

        fn meet(&self, other: &Self) -> Self {
            TestFact(self.0.union(&other.0).copied().collect())
        }

        fn less_or_equal(&self, other: &Self) -> bool {
            self.0.is_subset(&other.0)
        }
    }

    // Simple transfer function: adds block id to the set
    struct TestTransfer;

    impl TransferFunction<TestFact> for TestTransfer {
        fn transfer(&self, _stmt: &Stmt, fact: &TestFact) -> TestFact {
            fact.clone()
        }
    }

    #[test]
    fn test_data_flow_fact_bottom() {
        let bottom = TestFact::bottom();
        assert_eq!(bottom.0.len(), 0);
    }

    #[test]
    fn test_data_flow_fact_meet() {
        let mut set1 = HashSet::new();
        set1.insert(1);
        let fact1 = TestFact(set1);

        let mut set2 = HashSet::new();
        set2.insert(2);
        let fact2 = TestFact(set2);

        let result = fact1.meet(&fact2);
        assert_eq!(result.0.len(), 2);
        assert!(result.0.contains(&1));
        assert!(result.0.contains(&2));
    }

    #[test]
    fn test_solver_creation() {
        use crate::analysis::passes::ir::ControlFlowGraph;

        let cfg = Arc::new(ControlFlowGraph::new("test".to_string(), BasicBlockId(0)));
        let solver = DataFlowSolver::new(AnalysisDirection::Forward, TestTransfer, cfg);

        assert_eq!(solver.direction, AnalysisDirection::Forward);
    }

    #[test]
    fn test_simple_forward_analysis() {
        use crate::analysis::passes::ir::ControlFlowGraph;

        // Create a simple CFG with one block
        let mut cfg = ControlFlowGraph::new("test".to_string(), BasicBlockId(0));
        let mut block = BasicBlock::new(BasicBlockId(0), Terminator::Return);
        block.compute_successors();
        cfg.add_block(block);

        let solver = DataFlowSolver::new(AnalysisDirection::Forward, TestTransfer, Arc::new(cfg));

        let result = solver.solve();
        assert!(result.is_ok());
    }
}
