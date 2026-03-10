use crate::frameworks::dfa::annotated_cfg::BasicBlockId;
use crate::frameworks::dfa::lattice::PowerSetLattice;
use crate::frameworks::dfa::solver::Transfer;
use crate::frameworks::dfa::var::VarId;
use mlir::sir::Stmt;
use mlir::sir::Span;

/// A definition point
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Definition {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Option<Span>,
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
