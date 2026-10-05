//! CFG and ICFG data structures for BIR.

use crate::bir::ops::{Op, OpId, OpRef, SsaName};
use crate::sir::{Attr, Loc, Type};
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

impl FunctionId {
    /// The contract part of a `Contract.function` id, if it has one.
    pub fn contract(&self) -> Option<&str> {
        self.0.split_once('.').map(|(contract, _)| contract)
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%bb{}", self.0)
    }
}

impl Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
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
///
/// Blocks take parameters instead of using phi nodes (MLIR style): each
/// predecessor's terminator passes one argument per parameter.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub params: Vec<BlockParam>,
    pub ops: Vec<Op>,
    pub term: Terminator,
}

/// A block parameter: an SSA value bound on entry to the block.
#[derive(Debug, Clone)]
pub struct BlockParam {
    pub id: OpId,
    pub name: SsaName,
    pub ty: Type,
}

/// A control transfer to `block` that binds its parameters to `args`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockCall {
    pub block: BlockId,
    pub args: Vec<OpRef>,
}

/// A terminator instruction at the end of a basic block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    /// Conditional branch.
    Branch {
        cond: OpRef,
        then_dest: BlockCall,
        else_dest: BlockCall,
    },
    /// Unconditional jump.
    Jump(BlockCall),
    /// Transaction exit (normal or reverted).
    TxnExit { reverted: bool },
    /// Unreachable (e.g., after a revert with no continuation).
    Unreachable,
}

impl BasicBlock {
    pub fn new(id: BlockId) -> Self {
        BasicBlock { id, params: Vec::new(), ops: Vec::new(), term: Terminator::Unreachable }
    }
}

impl BlockCall {
    /// A transfer to `block` passing no arguments.
    pub fn new(block: BlockId) -> Self {
        BlockCall { block, args: Vec::new() }
    }
}

impl Terminator {
    /// An unconditional jump to `block` passing no arguments.
    pub fn jump(block: BlockId) -> Self {
        Terminator::Jump(BlockCall::new(block))
    }

    /// A conditional branch passing no arguments to either successor.
    pub fn branch(cond: OpRef, then_bb: BlockId, else_bb: BlockId) -> Self {
        Terminator::Branch {
            cond,
            then_dest: BlockCall::new(then_bb),
            else_dest: BlockCall::new(else_bb),
        }
    }

    /// The outgoing control transfers, in order.
    pub fn block_calls(&self) -> Vec<&BlockCall> {
        match self {
            Terminator::Branch { then_dest, else_dest, .. } => vec![then_dest, else_dest],
            Terminator::Jump(dest) => vec![dest],
            Terminator::TxnExit { .. } | Terminator::Unreachable => vec![],
        }
    }

    /// The outgoing control transfers, mutably.
    pub fn block_calls_mut(&mut self) -> Vec<&mut BlockCall> {
        match self {
            Terminator::Branch { then_dest, else_dest, .. } => vec![then_dest, else_dest],
            Terminator::Jump(dest) => vec![dest],
            Terminator::TxnExit { .. } | Terminator::Unreachable => vec![],
        }
    }

    /// The successor blocks, in order.
    pub fn successors(&self) -> Vec<BlockId> {
        self.block_calls().iter().map(|call| call.block).collect()
    }

    /// All SSA values read by the terminator (condition and block arguments).
    pub fn operands(&self) -> Vec<OpRef> {
        let cond = match self {
            Terminator::Branch { cond, .. } => Some(*cond),
            Terminator::Jump(_) | Terminator::TxnExit { .. } | Terminator::Unreachable => None,
        };
        let args = self
            .block_calls()
            .into_iter()
            .flat_map(|call| call.args.iter().copied());
        cond.into_iter().chain(args).collect()
    }

    /// Mutable access to all SSA values read by the terminator, in the same
    /// order as `operands`.
    pub fn operands_mut(&mut self) -> Vec<&mut OpRef> {
        match self {
            Terminator::Branch { cond, then_dest, else_dest } => std::iter::once(cond)
                .chain(then_dest.args.iter_mut())
                .chain(else_dest.args.iter_mut())
                .collect(),
            Terminator::Jump(dest) => dest.args.iter_mut().collect(),
            Terminator::TxnExit { .. } | Terminator::Unreachable => vec![],
        }
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  {}", self.id)?;
        if !self.params.is_empty() {
            let params: Vec<_> = self
                .params
                .iter()
                .map(|p| format!("{}: {}", p.id, p.ty))
                .collect();
            write!(f, "({})", params.join(", "))?;
        }
        writeln!(f, ":")?;
        for op in &self.ops {
            writeln!(f, "    {op}")?;
        }
        writeln!(f, "    {}", self.term)
    }
}

impl Display for BlockCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.block)?;
        if !self.args.is_empty() {
            let args: Vec<_> = self.args.iter().map(|a| a.to_string()).collect();
            write!(f, "({})", args.join(", "))?;
        }
        Ok(())
    }
}

impl Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Branch { cond, then_dest, else_dest } => {
                write!(f, "branch {cond}, {then_dest}, {else_dest}")
            }
            Terminator::Jump(dest) => write!(f, "jump {dest}"),
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
    /// Entry of a function (a transaction entry when it is public).
    TxnEntry { func: FunctionId },
    /// Exit of a function (normal or reverted).
    TxnExit { func: FunctionId, reverted: bool },
    /// The start of a basic block.
    BlockEntry(BlockLoc),
    /// An internal call; control continues in the callee.
    CallSite(OpLoc),
    /// The point where control resumes after a call.
    ReturnSite(OpLoc),
    /// An external call; may re-enter the contract's public functions.
    ExternalCallNode(OpLoc),
    /// Any other op.
    StmtNode(OpLoc),
}

/// A basic block, qualified by its function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockLoc {
    pub func: FunctionId,
    pub block: BlockId,
}

/// An op, qualified by its function (`OpId`s are unique per function).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpLoc {
    pub func: FunctionId,
    pub op: OpId,
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
    /// Re-entry edge from an external call to a public function's entry.
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
// AIRFunction — per-function CFG container
// ═══════════════════════════════════════════════════════════════════

/// A function in BIR form with SSA-renamed basic blocks.
#[derive(Debug, Clone)]
pub struct Function {
    pub id: FunctionId,
    /// Source attributes of the function (visibility, guards, ...).
    pub attrs: Vec<Attr>,
    pub blocks: Vec<BasicBlock>,
    pub is_public: bool,
    /// Source location of the function declaration.
    pub span: Option<Loc>,
}

impl Function {
    pub fn new(id: FunctionId, is_public: bool) -> Self {
        Function { id, attrs: Vec::new(), blocks: Vec::new(), is_public, span: None }
    }

    pub fn with_attrs(mut self, attrs: Vec<Attr>) -> Self {
        self.attrs = attrs;
        self
    }

    pub fn with_span(mut self, span: Option<Loc>) -> Self {
        self.span = span;
        self
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vis = if self.is_public { "public " } else { "" };
        writeln!(f, "{vis}function {} {{", self.id)?;
        for (i, bb) in self.blocks.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{bb}")?;
        }
        writeln!(f, "}}")
    }
}
