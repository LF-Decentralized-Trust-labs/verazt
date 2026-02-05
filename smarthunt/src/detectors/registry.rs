//! Detector registry.
//!
//! This module provides the detector registry for managing all detectors.

use crate::detectors::Detector;
use crate::config::Config;
use bugs::bug::{BugKind, RiskLevel};
use std::collections::HashMap;

/// Registry for all detectors.
#[derive(Default)]
pub struct DetectorRegistry {
    /// All registered detectors
    detectors: HashMap<String, Box<dyn Detector>>,

    /// Detectors grouped by category
    by_category: HashMap<BugKindKey, Vec<String>>,

    /// Detectors grouped by severity
    by_severity: HashMap<RiskLevelKey, Vec<String>>,
}

// Wrapper types for HashMap keys since BugKind and RiskLevel don't implement Hash
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BugKindKey(String);

impl From<&BugKind> for BugKindKey {
    fn from(kind: &BugKind) -> Self {
        BugKindKey(kind.as_str().to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RiskLevelKey(String);

impl From<&RiskLevel> for RiskLevelKey {
    fn from(level: &RiskLevel) -> Self {
        RiskLevelKey(level.as_str().to_string())
    }
}

impl DetectorRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a detector.
    pub fn register(&mut self, detector: Box<dyn Detector>) {
        let id = detector.id().to_string();

        self.by_category
            .entry(BugKindKey::from(&detector.bug_kind()))
            .or_default()
            .push(id.clone());

        self.by_severity
            .entry(RiskLevelKey::from(&detector.risk_level()))
            .or_default()
            .push(id.clone());

        self.detectors.insert(id, detector);
    }

    /// Get a detector by ID.
    pub fn get(&self, id: &str) -> Option<&dyn Detector> {
        self.detectors.get(id).map(|d| d.as_ref())
    }

    /// Get all detector IDs.
    pub fn all_ids(&self) -> Vec<&str> {
        self.detectors.keys().map(|s| s.as_str()).collect()
    }

    /// Get enabled detectors based on configuration.
    pub fn get_enabled(&self, config: &Config) -> Vec<&dyn Detector> {
        self.detectors
            .iter()
            .filter(|(id, detector)| {
                // Check if detector is enabled in config
                if !config.is_detector_enabled(id) {
                    return false;
                }

                // Check severity filter
                if !config.should_report_severity(&detector.risk_level()) {
                    return false;
                }

                // All category filters passed
                true
            })
            .map(|(_, d)| d.as_ref())
            .collect()
    }

    /// Get all detectors.
    pub fn all(&self) -> Vec<&dyn Detector> {
        self.detectors.values().map(|d| d.as_ref()).collect()
    }

    /// Get detector count.
    pub fn count(&self) -> usize {
        self.detectors.len()
    }

    /// Print detector list.
    pub fn print_list(&self) {
        println!("Available Detectors ({}):", self.count());
        println!("========================");

        let mut detectors: Vec<_> = self.detectors.values().collect();
        detectors.sort_by(|a, b| a.id().cmp(b.id()));

        for detector in detectors {
            println!(
                "  {} - {} [{}] [{}]",
                detector.id(),
                detector.name(),
                detector.risk_level(),
                detector.confidence(),
            );
        }
    }
}

/// Register all built-in detectors.
pub fn register_builtin_detectors(registry: &mut DetectorRegistry) {
    use crate::detectors::*;

    // Critical/High Vulnerability detectors
    registry.register(Box::new(reentrancy::ReentrancyDetector::new()));
    registry.register(Box::new(cei_violation::CeiViolationDetector::new()));
    registry.register(Box::new(missing_access_control::MissingAccessControlDetector::new()));

    // Medium Vulnerability detectors
    registry.register(Box::new(centralization_risk::CentralizationRiskDetector::new()));

    // Code quality detectors
    registry.register(Box::new(uninitialized::UninitializedDetector::new()));
    registry.register(Box::new(dead_code::DeadCodeDetector::new()));

    // Optimization detectors
    registry.register(Box::new(constant_state_var::ConstantStateVarDetector::new()));
    
    // Note: The following detectors have been migrated to the new detection framework
    // and are now available in smarthunt/src/detection/detectors/ast/:
    // - tx_origin, delegatecall, unchecked_call, low_level_call
    // - floating_pragma, shadowing, deprecated, visibility, timestamp_dependence
}

/// Create a registry with all built-in detectors.
pub fn create_default_registry() -> DetectorRegistry {
    let mut registry = DetectorRegistry::new();
    register_builtin_detectors(&mut registry);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = DetectorRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_default_registry() {
        let registry = create_default_registry();
        assert!(registry.count() > 0);
    }
}
