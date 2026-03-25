use crate::analysis::frameworks::dfa::lattice::PowerSetLattice;
use crate::analysis::frameworks::dfa::var::VarId;

use std::collections::HashSet;

/// Taint source: an initial taint marking
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum TaintSource {
    /// User input or external data
    UserInput(VarId),
    /// Return value from an untrusted call
    UntrustedCall(VarId),
    /// Custom taint source
    Custom(String, VarId),
}

/// Taint state per variable
pub type TaintState = PowerSetLattice<VarId>;

/// Taint analysis transfer function.
///
/// Propagates taint: if any operand of an assignment is tainted,
/// the result is tainted.
pub struct TaintTransfer {
    /// Initially tainted variables (sources)
    pub sources: HashSet<VarId>,
}

impl TaintTransfer {
    pub fn new(sources: HashSet<VarId>) -> Self {
        Self { sources }
    }
}

/// Taint analysis pass (to be integrated with analysis framework)
pub struct TaintAnalysisPass;
