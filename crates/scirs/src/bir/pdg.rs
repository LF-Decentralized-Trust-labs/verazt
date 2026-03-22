//! Program Dependence Graph (PDG) for BIR.

use crate::bir::cfg::BlockId;
use crate::bir::ops::OpId;
use std::fmt::{self, Display};

/// The Program Dependence Graph.
#[derive(Debug, Clone, Default)]
pub struct PDG {
    /// Data dependency edges: def → use.
    pub data_edges: Vec<(OpId, OpId)>,
    /// Control dependency edges: condition → dominated block.
    pub control_edges: Vec<(OpId, BlockId)>,
}

impl PDG {
    pub fn new() -> Self {
        PDG { data_edges: Vec::new(), control_edges: Vec::new() }
    }

    /// Add a data dependency edge (def → use).
    pub fn add_data_edge(&mut self, def: OpId, use_op: OpId) {
        self.data_edges.push((def, use_op));
    }

    /// Add a control dependency edge.
    pub fn add_control_edge(&mut self, cond: OpId, block: BlockId) {
        self.control_edges.push((cond, block));
    }
}

impl Display for PDG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "PDG: {} data edges, {} control edges",
            self.data_edges.len(),
            self.control_edges.len()
        )
    }
}
