use crate::irdfa::cfg::BasicBlockId;
use crate::irdfa::lattice::PowerSetLattice;
use crate::irdfa::solver::Transfer;
use crate::irdfa::var::VarId;
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

/// Reaching definitions analysis pass (to be integrated with analysis framework)
pub struct ReachingDefsPass;
