use crate::analysis::frameworks::dfa::lattice::PowerSetLattice;
use crate::analysis::frameworks::dfa::var::VarId;

use std::collections::HashSet;

/// A state-mutating operation
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct StateMutation {
    /// Variable being mutated
    pub var: VarId,
    /// Block in which the mutation occurs
    pub block: usize,
    /// Statement index
    pub stmt_index: usize,
}

/// State mutation lattice: set of mutated state variables
pub type MutationState = PowerSetLattice<VarId>;

/// State mutation analysis transfer function.
///
/// Tracks which storage/state variables have been modified
/// along each path.
pub struct StateMutationTransfer {
    /// Variables considered as "state" (e.g., storage variables)
    pub state_vars: HashSet<VarId>,
}

impl StateMutationTransfer {
    pub fn new(state_vars: HashSet<VarId>) -> Self {
        Self { state_vars }
    }
}

/// State mutation analysis pass (to be integrated with analysis framework)
pub struct StateMutationPass;
