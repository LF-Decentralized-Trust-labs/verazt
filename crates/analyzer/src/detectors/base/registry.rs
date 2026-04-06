//! Detector Registry
//!
//! Manages registration and discovery of bug detectors.

use crate::detectors::BugDetectionPass;
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
        let detector_id_str = detector.detector_id().as_str().to_string();
        let idx = self.detectors.len();
        self.detectors.push(detector);
        self.by_id.insert(name.clone(), idx);
        self.by_id.insert(detector_id_str, idx);
    }

    /// Get a detector by name or ID.
    pub fn get(&self, name_or_id: &str) -> Option<&dyn BugDetectionPass> {
        self.by_id
            .get(name_or_id)
            .map(|&idx| self.detectors[idx].as_ref())
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
    pub fn required_passes(&self) -> Vec<std::any::TypeId> {
        let mut passes = std::collections::HashSet::new();
        for detector in &self.detectors {
            for dep in detector.dependencies() {
                passes.insert(dep);
            }
        }
        passes.into_iter().collect()
    }

    /// Get detectors that depend on a specific pass.
    pub fn detectors_requiring(&self, pass: std::any::TypeId) -> Vec<&dyn BugDetectionPass> {
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
    use crate::detectors::scan_adapter::ScanDetectorAdapter;

    // Wrap all scan detectors as BugDetectionPass via the adapter
    let mut scan_registry = scanner::ScanRegistry::new();
    scanner::register_all_detectors(&mut scan_registry);
    for detector in scan_registry.into_detectors() {
        registry.register(Box::new(ScanDetectorAdapter::new(detector)));
    }
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
