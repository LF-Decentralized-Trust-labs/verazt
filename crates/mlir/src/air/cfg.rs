//! CFG and ICFG data structures for ANIR.

use crate::air::ops::{Op, OpId, OpRef};
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// ID types
// ═══════════════════════════════════════════════════════════════════

/// A unique identifier for a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// A unique identifier for a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId(pub String);

/// A unique identifier for an ICFG node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ICFGNodeId(pub usize);

impl Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

impl Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for ICFGNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "icfg{}", self.0)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Basic Block
// ═══════════════════════════════════════════════════════════════════

/// A basic block in the CFG.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub ops: Vec<Op>,
    pub term: Terminator,
}

/// A terminator instruction at the end of a basic block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    /// Conditional branch.
    Branch {
        cond: OpRef,
        then_bb: BlockId,
        else_bb: BlockId,
    },
    /// Unconditional jump.
    Jump(BlockId),
    /// Transaction exit (normal or reverted).
    TxnExit { reverted: bool },
    /// Unreachable (e.g., after a revert with no continuation).
    Unreachable,
}

impl BasicBlock {
    pub fn new(id: BlockId) -> Self {
        BasicBlock { id, ops: Vec::new(), term: Terminator::Unreachable }
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", self.id)?;
        for op in &self.ops {
            writeln!(f, "  {op}")?;
        }
        writeln!(f, "  {}", self.term)
    }
}

impl Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Branch { cond, then_bb, else_bb } => {
                write!(f, "branch {cond}, {then_bb}, {else_bb}")
            }
            Terminator::Jump(bb) => write!(f, "jump {bb}"),
            Terminator::TxnExit { reverted } => {
                if *reverted {
                    write!(f, "txn_exit(reverted)")
                } else {
                    write!(f, "txn_exit(ok)")
                }
            }
            Terminator::Unreachable => write!(f, "unreachable"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// ICFG
// ═══════════════════════════════════════════════════════════════════

/// An ICFG node type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICFGNode {
    /// Entry point for a transaction / public function.
    TxnEntry { func: FunctionId },
    /// Exit point for a transaction (normal or reverted).
    TxnExit { func: FunctionId, reverted: bool },
    /// A call site (implements CallOp).
    CallSite { op: OpId },
    /// A return site after a call.
    ReturnSite { op: OpId },
    /// An external call node (reentrancy risk).
    ExternalCallNode { op: OpId },
    /// A re-entry point for reentrancy analysis.
    ReentryPoint { func: FunctionId },
    /// A regular statement node.
    StmtNode { op: OpId },
}

/// Edge kind in the ICFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Intra-procedural control flow edge.
    CfgEdge,
    /// Call edge from call site to callee entry.
    CallEdge,
    /// Return edge from callee exit to return site.
    ReturnEdge,
    /// Re-entry edge from external call to re-entry point.
    ReentryEdge,
}

/// The interprocedural control flow graph.
#[derive(Debug, Clone, Default)]
pub struct ICFG {
    pub nodes: Vec<ICFGNode>,
    pub edges: Vec<(ICFGNodeId, ICFGNodeId, EdgeKind)>,
}

impl ICFG {
    pub fn new() -> Self {
        ICFG { nodes: Vec::new(), edges: Vec::new() }
    }

    /// Add a node and return its ID.
    pub fn add_node(&mut self, node: ICFGNode) -> ICFGNodeId {
        let id = ICFGNodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// Add an edge between two nodes.
    pub fn add_edge(&mut self, from: ICFGNodeId, to: ICFGNodeId, kind: EdgeKind) {
        self.edges.push((from, to, kind));
    }

    /// Count nodes of a specific type.
    pub fn count_nodes<F>(&self, predicate: F) -> usize
    where
        F: Fn(&ICFGNode) -> bool,
    {
        self.nodes.iter().filter(|n| predicate(n)).count()
    }
}

impl Display for ICFG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ICFG ({} nodes, {} edges):", self.nodes.len(), self.edges.len())?;
        for (i, node) in self.nodes.iter().enumerate() {
            writeln!(f, "  icfg{i}: {node:?}")?;
        }
        for (from, to, kind) in &self.edges {
            writeln!(f, "  {from} --{kind:?}--> {to}")?;
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════
// AnirFunction — per-function CFG container
// ═══════════════════════════════════════════════════════════════════

/// A function in ANIR form with SSA-renamed basic blocks.
#[derive(Debug, Clone)]
pub struct AnirFunction {
    pub id: FunctionId,
    pub blocks: Vec<BasicBlock>,
    pub is_public: bool,
}

impl AnirFunction {
    pub fn new(id: FunctionId, is_public: bool) -> Self {
        AnirFunction { id, blocks: Vec::new(), is_public }
    }
}

impl Display for AnirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vis = if self.is_public { "public " } else { "" };
        writeln!(f, "{vis}function {} {{", self.id)?;
        for bb in &self.blocks {
            write!(f, "{bb}")?;
        }
        writeln!(f, "}}")
    }
}
