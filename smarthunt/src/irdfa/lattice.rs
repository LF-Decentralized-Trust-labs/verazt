use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

/// A lattice for data flow analysis with clearer semantics
pub trait Lattice: Clone + Eq + Debug + Send + Sync {
    /// Bottom element (⊥) - no information / initial state
    fn bottom() -> Self;

    /// Top element (⊤) - all possible information
    fn top() -> Self;

    /// Join operation (⊔) - combines information from multiple paths
    /// For forward analysis: information at merge points
    /// For backward analysis: information from multiple successors
    fn join(&self, other: &Self) -> Self;

    /// Meet operation (⊓) - intersection of information
    fn meet(&self, other: &Self) -> Self;

    /// Partial order check: self ⊑ other
    fn less_or_equal(&self, other: &Self) -> bool;

    /// Check if this is the bottom element
    fn is_bottom(&self) -> bool {
        self == &Self::bottom()
    }

    /// Check if this is the top element
    fn is_top(&self) -> bool {
        self == &Self::top()
    }
}

/// PowerSet lattice for set-based analyses
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PowerSetLattice<T: Clone + Eq + Hash> {
    pub elements: HashSet<T>,
}

impl<T: Clone + Eq + Hash> PowerSetLattice<T> {
    pub fn new() -> Self {
        Self {
            elements: HashSet::new(),
        }
    }

    pub fn from_set(elements: HashSet<T>) -> Self {
        Self { elements }
    }

    pub fn insert(&mut self, elem: T) {
        self.elements.insert(elem);
    }

    pub fn contains(&self, elem: &T) -> bool {
        self.elements.contains(elem)
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

impl<T: Clone + Eq + Hash> Default for PowerSetLattice<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Eq + Hash + Send + Sync + Debug> Lattice for PowerSetLattice<T> {
    fn bottom() -> Self {
        Self {
            elements: HashSet::new(),
        }
    }

    fn top() -> Self {
        // PowerSet top requires knowing the universe, which we don't have
        // In practice, top is rarely needed for set-based analyses
        panic!("PowerSet top() requires universe - use join() for union")
    }

    fn join(&self, other: &Self) -> Self {
        Self {
            elements: self.elements.union(&other.elements).cloned().collect(),
        }
    }

    fn meet(&self, other: &Self) -> Self {
        Self {
            elements: self
                .elements
                .intersection(&other.elements)
                .cloned()
                .collect(),
        }
    }

    fn less_or_equal(&self, other: &Self) -> bool {
        self.elements.is_subset(&other.elements)
    }
}

/// Map lattice for variable-to-value analyses
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MapLattice<K: Clone + Eq + Hash, V: Lattice> {
    pub map: HashMap<K, V>,
}

impl<K: Clone + Eq + Hash, V: Lattice> MapLattice<K, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn set(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    pub fn remove(&mut self, key: &K) {
        self.map.remove(key);
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.map.keys()
    }
}

impl<K: Clone + Eq + Hash, V: Lattice> Default for MapLattice<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Eq + Hash + Send + Sync + Debug, V: Lattice> Lattice for MapLattice<K, V> {
    fn bottom() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn top() -> Self {
        // Map top would require all possible keys mapped to V::top()
        panic!("MapLattice top() is not well-defined")
    }

    fn join(&self, other: &Self) -> Self {
        let mut result = HashMap::new();

        // Add all keys from self
        for (k, v) in &self.map {
            if let Some(other_v) = other.map.get(k) {
                result.insert(k.clone(), v.join(other_v));
            } else {
                result.insert(k.clone(), v.clone());
            }
        }

        // Add keys only in other
        for (k, v) in &other.map {
            if !result.contains_key(k) {
                result.insert(k.clone(), v.clone());
            }
        }

        Self { map: result }
    }

    fn meet(&self, other: &Self) -> Self {
        let mut result = HashMap::new();

        // Only include keys present in both
        for (k, v) in &self.map {
            if let Some(other_v) = other.map.get(k) {
                result.insert(k.clone(), v.meet(other_v));
            }
        }

        Self { map: result }
    }

    fn less_or_equal(&self, other: &Self) -> bool {
        // self ⊑ other if all entries in self are ⊑ corresponding entries in other
        for (k, v) in &self.map {
            match other.map.get(k) {
                Some(other_v) => {
                    if !v.less_or_equal(other_v) {
                        return false;
                    }
                }
                None => return false, // self has key that other doesn't
            }
        }
        true
    }
}

/// Flat lattice for constant propagation
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum FlatLattice<T: Clone + Eq> {
    Bottom,
    Value(T),
    Top,
}

impl<T: Clone + Eq + Send + Sync + Debug> Lattice for FlatLattice<T> {
    fn bottom() -> Self {
        FlatLattice::Bottom
    }

    fn top() -> Self {
        FlatLattice::Top
    }

    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (FlatLattice::Bottom, x) | (x, FlatLattice::Bottom) => x.clone(),
            (FlatLattice::Top, _) | (_, FlatLattice::Top) => FlatLattice::Top,
            (FlatLattice::Value(v1), FlatLattice::Value(v2)) => {
                if v1 == v2 {
                    FlatLattice::Value(v1.clone())
                } else {
                    FlatLattice::Top
                }
            }
        }
    }

    fn meet(&self, other: &Self) -> Self {
        match (self, other) {
            (FlatLattice::Top, x) | (x, FlatLattice::Top) => x.clone(),
            (FlatLattice::Bottom, _) | (_, FlatLattice::Bottom) => FlatLattice::Bottom,
            (FlatLattice::Value(v1), FlatLattice::Value(v2)) => {
                if v1 == v2 {
                    FlatLattice::Value(v1.clone())
                } else {
                    FlatLattice::Bottom
                }
            }
        }
    }

    fn less_or_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (FlatLattice::Bottom, _) => true,
            (_, FlatLattice::Top) => true,
            (FlatLattice::Value(v1), FlatLattice::Value(v2)) => v1 == v2,
            _ => false,
        }
    }
}

/// Product lattice for combining multiple analyses
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ProductLattice<A: Lattice, B: Lattice> {
    pub first: A,
    pub second: B,
}

impl<A: Lattice, B: Lattice> ProductLattice<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A: Lattice, B: Lattice> Lattice for ProductLattice<A, B> {
    fn bottom() -> Self {
        Self {
            first: A::bottom(),
            second: B::bottom(),
        }
    }

    fn top() -> Self {
        Self {
            first: A::top(),
            second: B::top(),
        }
    }

    fn join(&self, other: &Self) -> Self {
        Self {
            first: self.first.join(&other.first),
            second: self.second.join(&other.second),
        }
    }

    fn meet(&self, other: &Self) -> Self {
        Self {
            first: self.first.meet(&other.first),
            second: self.second.meet(&other.second),
        }
    }

    fn less_or_equal(&self, other: &Self) -> bool {
        self.first.less_or_equal(&other.first) && self.second.less_or_equal(&other.second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powerset_lattice() {
        let mut s1 = PowerSetLattice::new();
        s1.insert(1);
        s1.insert(2);

        let mut s2 = PowerSetLattice::new();
        s2.insert(2);
        s2.insert(3);

        let joined = s1.join(&s2);
        assert_eq!(joined.len(), 3);
        assert!(joined.contains(&1));
        assert!(joined.contains(&2));
        assert!(joined.contains(&3));

        let met = s1.meet(&s2);
        assert_eq!(met.len(), 1);
        assert!(met.contains(&2));
    }

    #[test]
    fn test_flat_lattice() {
        let bottom: FlatLattice<i32> = FlatLattice::bottom();
        let top = FlatLattice::top();
        let val1 = FlatLattice::Value(42);
        let val2 = FlatLattice::Value(42);
        let val3 = FlatLattice::Value(99);

        assert_eq!(val1.join(&val2), FlatLattice::Value(42));
        assert_eq!(val1.join(&val3), FlatLattice::Top);
        assert_eq!(bottom.join(&val1), FlatLattice::Value(42));
        assert_eq!(top.join(&val1), FlatLattice::Top);
    }
}
