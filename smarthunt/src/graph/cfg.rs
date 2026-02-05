//! Control Flow Graph structures.
//!
//! This module provides data structures for representing control flow graphs.

use solidity::ast::{Expr, Loc, Stmt, VarDecl};
use std::collections::HashMap;
use crate::graph::symbol_table::FunctionId;

/// A basic block in a control flow graph.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier for this block
    pub id: BlockId,
    
    /// Statements in this block
    pub statements: Vec<CfgStatement>,
    
    /// Successor blocks
    pub successors: Vec<BlockId>,
    
    /// Predecessor blocks
    pub predecessors: Vec<BlockId>,
    
    /// Is this the entry block?
    pub is_entry: bool,
    
    /// Is this an exit block?
    pub is_exit: bool,
}

/// Unique identifier for a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

impl BlockId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

/// A statement in a CFG basic block.
#[derive(Debug, Clone)]
pub enum CfgStatement {
    /// Assignment: lhs = rhs
    Assign { lhs: Expr, rhs: Expr, loc: Option<Loc> },
    
    /// Variable declaration
    VarDecl { var: VarDecl, value: Option<Expr>, loc: Option<Loc> },
    
    /// Expression statement
    Expr { expr: Expr, loc: Option<Loc> },
    
    /// Return statement
    Return { value: Option<Expr>, loc: Option<Loc> },
    
    /// Conditional branch
    Branch { condition: Expr, true_target: BlockId, false_target: BlockId, loc: Option<Loc> },
    
    /// Unconditional jump
    Jump { target: BlockId },
    
    /// Function call
    Call { expr: Expr, loc: Option<Loc> },
    
    /// Revert
    Revert { error: Option<Expr>, loc: Option<Loc> },
    
    /// Assembly block (opaque)
    Assembly { loc: Option<Loc> },
    
    /// Placeholder for modifiers
    Placeholder { loc: Option<Loc> },
}

impl BasicBlock {
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            statements: Vec::new(),
            successors: Vec::new(),
            predecessors: Vec::new(),
            is_entry: false,
            is_exit: false,
        }
    }

    pub fn entry(id: BlockId) -> Self {
        let mut block = Self::new(id);
        block.is_entry = true;
        block
    }

    pub fn exit(id: BlockId) -> Self {
        let mut block = Self::new(id);
        block.is_exit = true;
        block
    }

    pub fn add_statement(&mut self, stmt: CfgStatement) {
        self.statements.push(stmt);
    }

    pub fn add_successor(&mut self, block: BlockId) {
        if !self.successors.contains(&block) {
            self.successors.push(block);
        }
    }

    pub fn add_predecessor(&mut self, block: BlockId) {
        if !self.predecessors.contains(&block) {
            self.predecessors.push(block);
        }
    }
}

/// Control flow graph for a function.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Function this CFG belongs to
    pub function_id: FunctionId,
    
    /// All basic blocks
    pub blocks: HashMap<BlockId, BasicBlock>,
    
    /// Entry block ID
    pub entry: BlockId,
    
    /// Exit block IDs (there can be multiple return points)
    pub exits: Vec<BlockId>,
    
    /// Next available block ID
    next_block_id: usize,
}

impl ControlFlowGraph {
    pub fn new(function_id: FunctionId) -> Self {
        let entry_id = BlockId::new(0);
        let exit_id = BlockId::new(1);
        
        let entry = BasicBlock::entry(entry_id);
        let exit = BasicBlock::exit(exit_id);
        
        let mut blocks = HashMap::new();
        blocks.insert(entry_id, entry);
        blocks.insert(exit_id, exit);
        
        Self {
            function_id,
            blocks,
            entry: entry_id,
            exits: vec![exit_id],
            next_block_id: 2,
        }
    }

    /// Create a new basic block and return its ID.
    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId::new(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.insert(id, BasicBlock::new(id));
        id
    }

    /// Get a basic block by ID.
    pub fn get_block(&self, id: &BlockId) -> Option<&BasicBlock> {
        self.blocks.get(id)
    }

    /// Get a mutable basic block by ID.
    pub fn get_block_mut(&mut self, id: &BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(id)
    }

    /// Add an edge between two blocks.
    pub fn add_edge(&mut self, from: BlockId, to: BlockId) {
        if let Some(from_block) = self.blocks.get_mut(&from) {
            from_block.add_successor(to);
        }
        if let Some(to_block) = self.blocks.get_mut(&to) {
            to_block.add_predecessor(from);
        }
    }

    /// Get all blocks in topological order (from entry).
    pub fn blocks_in_order(&self) -> Vec<&BasicBlock> {
        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();
        self.dfs_order(self.entry, &mut visited, &mut result);
        result
    }

    fn dfs_order<'a>(
        &'a self,
        current: BlockId,
        visited: &mut std::collections::HashSet<BlockId>,
        result: &mut Vec<&'a BasicBlock>,
    ) {
        if visited.contains(&current) {
            return;
        }
        visited.insert(current);
        
        if let Some(block) = self.blocks.get(&current) {
            result.push(block);
            for succ in &block.successors {
                self.dfs_order(*succ, visited, result);
            }
        }
    }

    /// Check if this CFG has any external calls.
    pub fn has_external_calls(&self) -> bool {
        for block in self.blocks.values() {
            for stmt in &block.statements {
                if let CfgStatement::Call { expr, .. } = stmt {
                    if is_external_call(expr) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Check if an expression is an external call.
pub fn is_external_call(expr: &Expr) -> bool {
    match expr {
        Expr::Call(call) => {
            // Check if it's a call to an external address
            match call.callee.as_ref() {
                Expr::Member(member) => {
                    let member_name = member.member.base.as_str();
                    matches!(member_name, "call" | "delegatecall" | "staticcall" | "transfer" | "send")
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Collection of CFGs for all functions.
#[derive(Debug, Clone, Default)]
pub struct CfgCollection {
    pub cfgs: HashMap<FunctionId, ControlFlowGraph>,
}

impl CfgCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, cfg: ControlFlowGraph) {
        self.cfgs.insert(cfg.function_id.clone(), cfg);
    }

    pub fn get(&self, id: &FunctionId) -> Option<&ControlFlowGraph> {
        self.cfgs.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solidity::ast::{FuncKind, Name};

    #[test]
    fn test_cfg_creation() {
        let func_id = FunctionId::new(
            Some(Name::new("TestContract".to_string(), None)),
            Name::new("testFunc".to_string(), None),
            FuncKind::ContractFunc,
        );
        let cfg = ControlFlowGraph::new(func_id);
        
        assert_eq!(cfg.blocks.len(), 2); // Entry and exit
        assert_eq!(cfg.exits.len(), 1);
    }

    #[test]
    fn test_create_block() {
        let func_id = FunctionId::new(
            Some(Name::new("TestContract".to_string(), None)),
            Name::new("testFunc".to_string(), None),
            FuncKind::ContractFunc,
        );
        let mut cfg = ControlFlowGraph::new(func_id);
        
        let block_id = cfg.create_block();
        assert_eq!(block_id.0, 2); // 0 and 1 are entry and exit
        assert_eq!(cfg.blocks.len(), 3);
    }
}
