//! AnirModule — the top-level ANIR container.

use crate::air::alias::AliasMap;
use crate::air::call_graph::CallGraph;
use crate::air::cfg::{AnirFunction, ICFG};
use crate::air::pdg::PDG;
use crate::air::summary::FunctionSummary;
use crate::air::taint::TaintGraph;
use std::fmt::{self, Display};

/// The top-level ANIR module — one per SIR Module.
#[derive(Debug, Clone)]
pub struct AnirModule {
    /// Identifier of the source module.
    pub source_module_id: String,
    /// All lowered functions.
    pub functions: Vec<AnirFunction>,
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

impl AnirModule {
    pub fn new(source_module_id: String) -> Self {
        AnirModule {
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

impl Display for AnirModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "AnirModule: {}", self.source_module_id)?;
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
