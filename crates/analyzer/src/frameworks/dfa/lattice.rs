//! Join-semilattices for dataflow facts.

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// The powerset of `T` ordered by inclusion, for set-based analyses.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PowerSetLattice<T: Clone + Eq + Hash> {
    pub elements: HashSet<T>,
}

// ═══════════════════════════════════════════════════════════════════
// Traits
// ═══════════════════════════════════════════════════════════════════

/// A join-semilattice of dataflow facts.
pub trait Lattice: Clone + Eq + Debug {
    /// The least element: no information, the initial state.
    fn bottom() -> Self;

    /// The least upper bound, combining facts from merging paths.
    fn join(&self, other: &Self) -> Self;
}

// ═══════════════════════════════════════════════════════════════════
// PowerSetLattice Implementations
// ═══════════════════════════════════════════════════════════════════

impl<T: Clone + Eq + Hash> PowerSetLattice<T> {
    pub fn insert(&mut self, elem: T) {
        self.elements.insert(elem);
    }

    pub fn contains(&self, elem: &T) -> bool {
        self.elements.contains(elem)
    }
}

impl<T: Clone + Eq + Hash + Debug> Lattice for PowerSetLattice<T> {
    fn bottom() -> Self {
        Self { elements: HashSet::new() }
    }

    fn join(&self, other: &Self) -> Self {
        Self { elements: self.elements.union(&other.elements).cloned().collect() }
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powerset_join_is_union() {
        let mut s1 = PowerSetLattice::bottom();
        s1.insert(1);
        s1.insert(2);
        let mut s2 = PowerSetLattice::bottom();
        s2.insert(2);
        s2.insert(3);

        let joined = s1.join(&s2);
        assert_eq!(joined.elements, HashSet::from([1, 2, 3]));
    }
}
