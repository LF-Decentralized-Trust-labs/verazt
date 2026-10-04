//! Registry of constructible analysis passes, so that dependencies declared
//! by pass type can be instantiated on demand.

use crate::passes::base::{AnalysisPass, PassError, PassResult};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Constructors of analysis passes, keyed by pass type.
#[derive(Default)]
pub struct PassRegistry {
    constructors: HashMap<TypeId, fn() -> Box<dyn AnalysisPass>>,
}

// ═══════════════════════════════════════════════════════════════════
// PassRegistry Implementations
// ═══════════════════════════════════════════════════════════════════

impl PassRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Make pass `P` available to satisfy dependencies on it.
    pub fn register<P: AnalysisPass + Default>(&mut self) {
        self.constructors.insert(TypeId::of::<P>(), construct::<P>);
    }

    /// Instances of the passes in `roots` and of everything they depend on,
    /// transitively, each once.
    pub fn instantiate_closure(
        &self,
        roots: impl IntoIterator<Item = TypeId>,
    ) -> PassResult<Vec<Box<dyn AnalysisPass>>> {
        let mut pending: Vec<TypeId> = roots.into_iter().collect();
        let mut seen = HashSet::new();
        let mut passes = Vec::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            let construct = self
                .constructors
                .get(&id)
                .ok_or_else(|| PassError::PassNotFound(format!("{id:?}")))?;
            let pass = construct();
            pending.extend(pass.dependencies());
            passes.push(pass);
        }
        Ok(passes)
    }
}

fn construct<P: AnalysisPass + Default>() -> Box<dyn AnalysisPass> {
    Box::new(P::default())
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::bir::{ICFGPass, TaintPass};
    use crate::passes::register_all_passes;

    #[test]
    fn test_instantiate_closure_includes_transitive_dependencies() {
        let mut registry = PassRegistry::new();
        register_all_passes(&mut registry);
        let passes = registry.instantiate_closure([TypeId::of::<TaintPass>()]).unwrap();
        let ids: HashSet<TypeId> = passes.iter().map(|p| p.id()).collect();
        assert_eq!(ids, HashSet::from([TypeId::of::<TaintPass>(), TypeId::of::<ICFGPass>()]));
    }

    #[test]
    fn test_instantiate_closure_rejects_unregistered_dependency() {
        let mut registry = PassRegistry::new();
        registry.register::<TaintPass>();
        let result = registry.instantiate_closure([TypeId::of::<TaintPass>()]);
        assert!(matches!(result, Err(PassError::PassNotFound(_))));
    }
}
