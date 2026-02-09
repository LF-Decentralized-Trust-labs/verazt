use crate::dfa::cfg::BasicBlockId;
use crate::dfa::lattice::PowerSetLattice;
use crate::dfa::solver::Transfer;
use crate::dfa::var::VarId;
use solidity::ast::Loc;
use solidity::ir::Stmt;

/// A definition point
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Definition {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Option<Loc>,
}

/// Reaching definitions lattice (set of definitions)
pub type ReachingDefsLattice = PowerSetLattice<Definition>;

/// Transfer function for reaching definitions
pub struct ReachingDefsTransfer;

impl Transfer<ReachingDefsLattice> for ReachingDefsTransfer {
    fn transfer_stmt(&self, _stmt: &Stmt, fact: &ReachingDefsLattice) -> ReachingDefsLattice {
        // TODO: Implement reaching definitions transfer function
        fact.clone()
    }
}

/// Reaching definitions analysis pass (to be integrated with analysis
/// framework)
pub struct ReachingDefsPass;
