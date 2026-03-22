//! Module — the top-level BIR container.

use crate::bir::alias::AliasMap;
use crate::bir::call_graph::CallGraph;
use crate::bir::cfg::{Function, ICFG};
use crate::bir::pdg::PDG;
use crate::bir::summary::FunctionSummary;
use crate::bir::taint::TaintGraph;
use std::fmt::{self, Display};

/// The top-level BIR module — one per SIR Module.
#[derive(Debug, Clone)]
pub struct Module {
    /// Identifier of the source module.
    pub source_module_id: String,
    /// All lowered functions.
    pub functions: Vec<Function>,
    /// The interprocedural control flow graph.
    pub icfg: ICFG,
    /// The call graph.
    pub call_graph: CallGraph,
    /// Alias analysis sets.
    pub alias_sets: AliasMap,
    /// The program dependence graph.
    pub pdg: PDG,
    /// The taint graph.
    pub taint_graph: TaintGraph,
    /// Per-function summaries.
    pub summaries: Vec<FunctionSummary>,
}

impl Module {
    pub fn new(source_module_id: String) -> Self {
        Module {
            source_module_id,
            functions: Vec::new(),
            icfg: ICFG::new(),
            call_graph: CallGraph::new(),
            alias_sets: AliasMap::new(),
            pdg: PDG::new(),
            taint_graph: TaintGraph::new(),
            summaries: Vec::new(),
        }
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Module: {}", self.source_module_id)?;
        writeln!(f, "  functions: {}", self.functions.len())?;
        writeln!(f, "  {}", self.icfg)?;
        writeln!(f, "  {}", self.call_graph)?;
        writeln!(f, "  {}", self.alias_sets)?;
        writeln!(f, "  {}", self.taint_graph)?;
        writeln!(f, "  summaries: {}", self.summaries.len())?;
        for func in &self.functions {
            write!(f, "\n{func}")?;
        }
        Ok(())
    }
}
