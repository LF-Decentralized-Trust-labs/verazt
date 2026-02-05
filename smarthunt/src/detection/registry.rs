//! Detector Registry
//!
//! Manages registration and discovery of bug detectors.

use crate::detection::pass::BugDetectionPass;
use solidity::analysis::pass_id::PassId;
use std::collections::HashMap;

/// Registry for managing bug detectors.
///
/// The registry provides:
/// - Detector registration and lookup
/// - Filtering by representation, level, or enabled state
/// - Dependency collection for required analysis passes
#[derive(Default)]
pub struct DetectorRegistry {
    /// All registered detectors.
    detectors: Vec<Box<dyn BugDetectionPass>>,

    /// Index by detector ID (pass name).
    by_id: HashMap<String, usize>,
}

impl DetectorRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a detector.
    pub fn register(&mut self, detector: Box<dyn BugDetectionPass>) {
        let name = detector.name().to_string();
        let pass_id = detector.id().as_str().to_string();
        let idx = self.detectors.len();
        self.detectors.push(detector);
        self.by_id.insert(name.clone(), idx);
        self.by_id.insert(pass_id, idx);
    }

    /// Get a detector by name or ID.
    pub fn get(&self, name_or_id: &str) -> Option<&dyn BugDetectionPass> {
        self.by_id.get(name_or_id).map(|&idx| self.detectors[idx].as_ref())
    }

    /// Get all registered detectors.
    pub fn all(&self) -> impl Iterator<Item = &dyn BugDetectionPass> {
        self.detectors.iter().map(|d| d.as_ref())
    }

    /// Get the number of registered detectors.
    pub fn len(&self) -> usize {
        self.detectors.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.detectors.is_empty()
    }

    /// Get all required analysis passes for all registered detectors.
    pub fn required_passes(&self) -> Vec<PassId> {
        let mut passes = std::collections::HashSet::new();
        for detector in &self.detectors {
            for dep in detector.dependencies() {
                passes.insert(dep);
            }
        }
        passes.into_iter().collect()
    }

    /// Get detectors that depend on a specific pass.
    pub fn detectors_requiring(&self, pass: PassId) -> Vec<&dyn BugDetectionPass> {
        self.detectors
            .iter()
            .filter(|d| d.dependencies().contains(&pass))
            .map(|d| d.as_ref())
            .collect()
    }

    /// Get detector names.
    pub fn names(&self) -> Vec<&str> {
        self.detectors.iter().map(|d| d.name()).collect()
    }

    /// Filter detectors by a predicate.
    pub fn filter<F>(&self, predicate: F) -> Vec<&dyn BugDetectionPass>
    where
        F: Fn(&dyn BugDetectionPass) -> bool,
    {
        self.detectors
            .iter()
            .filter(|d| predicate(d.as_ref()))
            .map(|d| d.as_ref())
            .collect()
    }
}

/// Register all built-in detectors.
pub fn register_all_detectors(registry: &mut DetectorRegistry) {
    use crate::detection::detectors::ast::*;

    // AST-based detectors
    registry.register(Box::new(TxOriginDetector::new()));
    registry.register(Box::new(FloatingPragmaDetector::new()));
    registry.register(Box::new(VisibilityDetector::new()));
    registry.register(Box::new(DeprecatedDetector::new()));
    registry.register(Box::new(LowLevelCallDetector::new()));
    registry.register(Box::new(UncheckedCallDetector::new()));
    registry.register(Box::new(ShadowingDetector::new()));
    registry.register(Box::new(TimestampDependenceDetector::new()));
    registry.register(Box::new(DelegatecallDetector::new()));
    registry.register(Box::new(UninitializedStorageDetector::new()));
    registry.register(Box::new(CentralizationRiskDetector::new()));
    registry.register(Box::new(CeiViolationDetector::new()));
    registry.register(Box::new(ReentrancyDetector::new()));
    registry.register(Box::new(MissingAccessControlDetector::new()));
    registry.register(Box::new(DeadCodeDetector::new()));
    registry.register(Box::new(ConstantStateVarDetector::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_empty() {
        let registry = DetectorRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
