//! Registry of constructible analysis passes, so that dependencies declared
//! by pass type can be instantiated on demand.

use crate::passes::base::{AnalysisPass, ErasedAnalysisPass, PassError, PassResult};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Constructors of analysis passes, keyed by pass type.
#[derive(Default)]
pub struct PassRegistry {
    constructors: HashMap<TypeId, fn() -> Box<dyn ErasedAnalysisPass>>,
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
    /// transitively, each once. Each root comes with the name of the pass
    /// or detector requiring it, for errors.
    pub fn instantiate_closure<'a>(
        &self,
        roots: impl IntoIterator<Item = (TypeId, &'a str)>,
    ) -> PassResult<Vec<Box<dyn ErasedAnalysisPass>>> {
        let mut pending: Vec<(TypeId, &str)> = roots.into_iter().collect();
        let mut seen = HashSet::new();
        let mut passes: Vec<Box<dyn ErasedAnalysisPass>> = Vec::new();
        while let Some((id, requester)) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            let construct = self
                .constructors
                .get(&id)
                .ok_or_else(|| PassError::UnregisteredDependency(requester.to_string()))?;
            let pass = construct();
            pending.extend(pass.dependencies().into_iter().map(|dep| (dep, pass.name())));
            passes.push(pass);
        }
        Ok(passes)
    }
}

fn construct<P: AnalysisPass + Default>() -> Box<dyn ErasedAnalysisPass> {
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
        let passes = registry.instantiate_closure([(TypeId::of::<TaintPass>(), "d")]).unwrap();
        let ids: HashSet<TypeId> = passes.iter().map(|p| p.id()).collect();
        assert_eq!(ids, HashSet::from([TypeId::of::<TaintPass>(), TypeId::of::<ICFGPass>()]));
    }

    #[test]
    fn test_instantiate_closure_rejects_unregistered_dependency() {
        let mut registry = PassRegistry::new();
        registry.register::<TaintPass>();
        let result = registry.instantiate_closure([(TypeId::of::<TaintPass>(), "d")]);
        assert!(matches!(result, Err(PassError::UnregisteredDependency(name)) if name == "taint"));
    }
}
