use crate::irdfa::var::VarId;
use solidity::ir::{Expr, Stmt};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Unique identifier for a basic block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BasicBlockId(pub usize);

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// Terminator instruction for a basic block
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    /// Unconditional jump to a block
    Jump(BasicBlockId),

    /// Conditional branch: if condition then true_block else false_block
    Branch {
        condition: Expr,
        true_block: BasicBlockId,
        false_block: BasicBlockId,
    },

    /// Return from function
    Return,

    /// Revert/throw
    Revert,

    /// Unreachable code
    Unreachable,
}

/// Enhanced basic block with def/use information
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier for this block
    pub id: BasicBlockId,

    /// Statements in this block (excluding terminator)
    pub stmts: Vec<Stmt>,

    /// Terminator instruction
    pub terminator: Terminator,

    /// Predecessor blocks
    pub predecessors: Vec<BasicBlockId>,

    /// Successor blocks (derived from terminator)
    pub successors: Vec<BasicBlockId>,

    /// Variables defined in this block (precomputed)
    pub defs: HashSet<VarId>,

    /// Variables used in this block (precomputed)
    pub uses: HashSet<VarId>,

    /// Variables used before defined in this block (upward exposed)
    pub upward_exposed: HashSet<VarId>,
}

impl BasicBlock {
    /// Create a new basic block
    pub fn new(id: BasicBlockId, terminator: Terminator) -> Self {
        Self {
            id,
            stmts: Vec::new(),
            terminator,
            predecessors: Vec::new(),
            successors: Vec::new(),
            defs: HashSet::new(),
            uses: HashSet::new(),
            upward_exposed: HashSet::new(),
        }
    }

    /// Add a statement to this block
    pub fn add_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    /// Compute successors from terminator
    pub fn compute_successors(&mut self) {
        self.successors = match &self.terminator {
            Terminator::Jump(target) => vec![*target],
            Terminator::Branch {
                true_block,
                false_block,
                ..
            } => vec![*true_block, *false_block],
            Terminator::Return | Terminator::Revert | Terminator::Unreachable => vec![],
        };
    }

    /// Compute def/use sets for this block
    /// This should be called after all statements are added
    pub fn compute_def_use(&mut self) {
        self.defs.clear();
        self.uses.clear();
        self.upward_exposed.clear();

        let mut local_defs = HashSet::new();

        // Process statements in order
        for stmt in &self.stmts {
            // Collect uses in this statement
            let uses = collect_used_vars(stmt);
            for var in uses {
                // If not locally defined yet, it's upward exposed
                if !local_defs.contains(&var) {
                    self.upward_exposed.insert(var.clone());
                }
                self.uses.insert(var);
            }

            // Collect definitions in this statement
            let defs = collect_defined_vars(stmt);
            for var in defs {
                local_defs.insert(var.clone());
                self.defs.insert(var);
            }
        }

        // Also collect uses from terminator condition
        if let Terminator::Branch { condition, .. } = &self.terminator {
            let uses = crate::irdfa::utils::get_vars_in_expr(condition);
            for var in uses {
                if !local_defs.contains(&var) {
                    self.upward_exposed.insert(var.clone());
                }
                self.uses.insert(var);
            }
        }
    }
}

/// Enhanced CFG with additional metadata
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Function identifier (contract::function name)
    pub function_id: String,

    /// All basic blocks
    pub blocks: HashMap<BasicBlockId, BasicBlock>,

    /// Entry block
    pub entry: BasicBlockId,

    /// Exit blocks (can have multiple due to multiple returns)
    pub exit_blocks: Vec<BasicBlockId>,

    /// Reverse postorder traversal (precomputed for efficiency)
    pub reverse_postorder: Vec<BasicBlockId>,
}

impl ControlFlowGraph {
    /// Create a new CFG
    pub fn new(function_id: String, entry: BasicBlockId) -> Self {
        Self {
            function_id,
            blocks: HashMap::new(),
            entry,
            exit_blocks: Vec::new(),
            reverse_postorder: Vec::new(),
        }
    }

    /// Add a block to the CFG
    pub fn add_block(&mut self, block: BasicBlock) {
        self.blocks.insert(block.id, block);
    }

    /// Get a block by ID
    pub fn get_block(&self, id: BasicBlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    /// Get a mutable block by ID
    pub fn get_block_mut(&mut self, id: BasicBlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }

    /// Compute predecessors for all blocks
    pub fn compute_predecessors(&mut self) {
        // Clear existing predecessors
        for block in self.blocks.values_mut() {
            block.predecessors.clear();
        }

        // Compute successors first
        let block_ids: Vec<_> = self.blocks.keys().copied().collect();
        for id in block_ids {
            if let Some(block) = self.blocks.get_mut(&id) {
                block.compute_successors();
            }
        }

        // Build predecessor lists
        let successors: Vec<_> = self
            .blocks
            .iter()
            .map(|(id, block)| (*id, block.successors.clone()))
            .collect();

        for (pred_id, succs) in successors {
            for succ_id in succs {
                if let Some(succ_block) = self.blocks.get_mut(&succ_id) {
                    succ_block.predecessors.push(pred_id);
                }
            }
        }
    }

    /// Compute reverse postorder traversal
    pub fn compute_reverse_postorder(&mut self) {
        let mut visited = HashSet::new();
        let mut postorder = Vec::new();

        self.postorder_dfs(self.entry, &mut visited, &mut postorder);

        postorder.reverse();
        self.reverse_postorder = postorder;
    }

    /// Helper for postorder DFS
    fn postorder_dfs(
        &self,
        block_id: BasicBlockId,
        visited: &mut HashSet<BasicBlockId>,
        postorder: &mut Vec<BasicBlockId>,
    ) {
        if visited.contains(&block_id) {
            return;
        }

        visited.insert(block_id);

        if let Some(block) = self.blocks.get(&block_id) {
            for &succ in &block.successors {
                self.postorder_dfs(succ, visited, postorder);
            }
        }

        postorder.push(block_id);
    }

    /// Identify exit blocks (blocks with no successors or return/revert terminators)
    pub fn compute_exit_blocks(&mut self) {
        self.exit_blocks.clear();

        for (id, block) in &self.blocks {
            match &block.terminator {
                Terminator::Return | Terminator::Revert => {
                    self.exit_blocks.push(*id);
                }
                _ if block.successors.is_empty() => {
                    self.exit_blocks.push(*id);
                }
                _ => {}
            }
        }
    }

    /// Compute all metadata (predecessors, reverse postorder, exit blocks, def/use)
    pub fn compute_metadata(&mut self) {
        self.compute_predecessors();
        self.compute_reverse_postorder();
        self.compute_exit_blocks();

        // Compute def/use for each block
        let block_ids: Vec<_> = self.blocks.keys().copied().collect();
        for id in block_ids {
            if let Some(block) = self.blocks.get_mut(&id) {
                block.compute_def_use();
            }
        }
    }
}

/// Collect variables used in a statement
fn collect_used_vars(stmt: &Stmt) -> Vec<VarId> {
    crate::irdfa::utils::collect_used_vars(stmt)
}

/// Collect variables defined in a statement
fn collect_defined_vars(stmt: &Stmt) -> Vec<VarId> {
    crate::irdfa::utils::collect_defined_vars(stmt)
}

