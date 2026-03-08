//! Call graph for AIR.

use crate::air::cfg::FunctionId;
use crate::air::ops::OpId;
use std::fmt::{self, Display};

/// The call graph tracks static and dynamic call edges.
#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    /// Statically resolved call edges (caller → callee).
    pub static_edges: Vec<(FunctionId, FunctionId)>,
    /// Dynamic call edges (call site op → potential callee).
    pub dynamic_edges: Vec<(OpId, FunctionId)>,
}

impl CallGraph {
    pub fn new() -> Self {
        CallGraph { static_edges: Vec::new(), dynamic_edges: Vec::new() }
    }

    /// Add a static call edge.
    pub fn add_static_edge(&mut self, caller: FunctionId, callee: FunctionId) {
        self.static_edges.push((caller, callee));
    }

    /// Add a dynamic call edge.
    pub fn add_dynamic_edge(&mut self, call_site: OpId, callee: FunctionId) {
        self.dynamic_edges.push((call_site, callee));
    }
}

impl Display for CallGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "CallGraph: {} static, {} dynamic edges",
            self.static_edges.len(),
            self.dynamic_edges.len()
        )
    }
}
