use crate::absint::cfg::BasicBlockId;
use crate::absint::lattice::PowerSetLattice;
use crate::absint::solver::Transfer;
use crate::absint::var::VarId;
use mlir::sir::Stmt;
use solidity::ast::Loc;

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
