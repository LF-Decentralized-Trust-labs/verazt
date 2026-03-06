//! Taint graph data structures.

use crate::interfaces::TaintLabel;
use crate::ops::OpId;
use std::fmt::{self, Display};

/// The taint graph tracks how tainted values flow through the program.
#[derive(Debug, Clone, Default)]
pub struct TaintGraph {
    /// Initial taint seeds (from TaintSource ops).
    pub seeds: Vec<TaintSeed>,
    /// Propagation edges: (source_op, derived_op).
    pub propagation: Vec<(OpId, OpId)>,
    /// Registered taint sinks.
    pub sinks: Vec<TaintSinkEntry>,
}

/// A taint seed — an op that introduces tainted data.
#[derive(Debug, Clone)]
pub struct TaintSeed {
    pub op: OpId,
    pub label: TaintLabel,
}

/// A registered taint sink entry.
#[derive(Debug, Clone)]
pub struct TaintSinkEntry {
    pub op: OpId,
    pub category: crate::interfaces::SinkCategory,
}

impl TaintGraph {
    pub fn new() -> Self {
        TaintGraph { seeds: Vec::new(), propagation: Vec::new(), sinks: Vec::new() }
    }

    /// Add a taint seed.
    pub fn seed(&mut self, op: OpId, label: TaintLabel) {
        self.seeds.push(TaintSeed { op, label });
    }

    /// Register a taint sink.
    pub fn register_sink(&mut self, op: OpId, category: crate::interfaces::SinkCategory) {
        self.sinks.push(TaintSinkEntry { op, category });
    }

    /// Add a propagation edge.
    pub fn propagate(&mut self, from: OpId, to: OpId) {
        self.propagation.push((from, to));
    }
}

impl Display for TaintGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TaintGraph:")?;
        writeln!(f, "  seeds: {}", self.seeds.len())?;
        writeln!(f, "  propagation edges: {}", self.propagation.len())?;
        writeln!(f, "  sinks: {}", self.sinks.len())?;
        Ok(())
    }
}
