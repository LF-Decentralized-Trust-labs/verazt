//! Detector Registry
//!
//! Manages registration and discovery of bug detectors.

use crate::detectors::BugDetectionPass;
use crate::detectors::bir::{CrossFunctionReentrancyDetector, ReentrancyFlowDetector};
use crate::detectors::sir::evm::*;
use crate::detectors::sir::{ScanDetector, ScanDetectorAdapter};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Registry of bug detectors, looked up by their `DetectorId` string.
#[derive(Default)]
pub struct DetectorRegistry {
    /// Index into `detectors` by detector ID string.
    by_id: HashMap<&'static str, usize>,

    /// All registered detectors, in registration order.
    detectors: Vec<Box<dyn BugDetectionPass>>,
}

// ═══════════════════════════════════════════════════════════════════
// DetectorRegistry Implementations
// ═══════════════════════════════════════════════════════════════════

impl DetectorRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a detector.
    ///
    /// # Panics
    ///
    /// Panics if a detector with the same ID is already registered.
    pub fn register(&mut self, detector: Box<dyn BugDetectionPass>) {
        let id = detector.meta().id.as_str();
        let idx = self.detectors.len();
        let previous = self.by_id.insert(id, idx);
        assert!(previous.is_none(), "detector '{id}' registered twice");
        self.detectors.push(detector);
    }

    /// Get a detector by its ID string (e.g. `"tx-origin"`).
    pub fn get(&self, id: &str) -> Option<&dyn BugDetectionPass> {
        self.by_id.get(id).map(|&idx| self.detectors[idx].as_ref())
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
}

// ═══════════════════════════════════════════════════════════════════
// Registration
// ═══════════════════════════════════════════════════════════════════

/// Register all built-in detectors.
pub fn register_all_detectors(registry: &mut DetectorRegistry) {
    let scan_detectors: Vec<Box<dyn ScanDetector>> = vec![
        // ── Security: EVM ───────────────────────────────────────────
        Box::new(ArithmeticOverflowDetector::new()),
        Box::new(BadRandomnessDetector::new()),
        Box::new(CentralizationRiskDetector::new()),
        Box::new(DelegatecallDetector::new()),
        Box::new(DenialOfServiceDetector::new()),
        Box::new(FrontRunningDetector::new()),
        Box::new(LowLevelCallDetector::new()),
        Box::new(MissingAccessControlDetector::new()),
        Box::new(ReentrancyDetector::new()),
        Box::new(ShortAddressDetector::new()),
        Box::new(TimestampDependenceDetector::new()),
        Box::new(TxOriginDetector::new()),
        Box::new(UncheckedCallDetector::new()),
        Box::new(UninitializedDetector::new()),
        // ── Quality: EVM ────────────────────────────────────────────
        Box::new(ConstantStateVarDetector::new()),
        Box::new(DeadCodeDetector::new()),
        Box::new(DeprecatedFeaturesDetector::new()),
        Box::new(FloatingPragmaDetector::new()),
        Box::new(ShadowingDetector::new()),
        Box::new(VisibilityDetector::new()),
    ];
    for detector in scan_detectors {
        registry.register(Box::new(ScanDetectorAdapter::new(detector)));
    }

    // BIR dataflow detectors
    registry.register(Box::new(CrossFunctionReentrancyDetector));
    registry.register(Box::new(ReentrancyFlowDetector));
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::DetectorId;

    /// Every `DetectorId` variant, in the order of `ordinal`.
    const ALL_IDS: [DetectorId; 22] = [
        DetectorId::ArithmeticOverflow,
        DetectorId::BadRandomness,
        DetectorId::CentralizationRisk,
        DetectorId::ConstantStateVar,
        DetectorId::DeadCode,
        DetectorId::Delegatecall,
        DetectorId::DenialOfService,
        DetectorId::Deprecated,
        DetectorId::FloatingPragma,
        DetectorId::FrontRunning,
        DetectorId::LowLevelCall,
        DetectorId::MissingAccessControl,
        DetectorId::Reentrancy,
        DetectorId::Shadowing,
        DetectorId::ShortAddress,
        DetectorId::TimestampDependence,
        DetectorId::TxOrigin,
        DetectorId::UncheckedCall,
        DetectorId::UninitializedStorage,
        DetectorId::Visibility,
        DetectorId::CrossFunctionReentrancy,
        DetectorId::ReentrancyFlow,
    ];

    /// The position of `id` in `ALL_IDS`. The match is exhaustive, so adding
    /// a `DetectorId` variant fails to compile until it is given the next
    /// ordinal and appended to `ALL_IDS`.
    fn ordinal(id: DetectorId) -> usize {
        match id {
            DetectorId::ArithmeticOverflow => 0,
            DetectorId::BadRandomness => 1,
            DetectorId::CentralizationRisk => 2,
            DetectorId::ConstantStateVar => 3,
            DetectorId::DeadCode => 4,
            DetectorId::Delegatecall => 5,
            DetectorId::DenialOfService => 6,
            DetectorId::Deprecated => 7,
            DetectorId::FloatingPragma => 8,
            DetectorId::FrontRunning => 9,
            DetectorId::LowLevelCall => 10,
            DetectorId::MissingAccessControl => 11,
            DetectorId::Reentrancy => 12,
            DetectorId::Shadowing => 13,
            DetectorId::ShortAddress => 14,
            DetectorId::TimestampDependence => 15,
            DetectorId::TxOrigin => 16,
            DetectorId::UncheckedCall => 17,
            DetectorId::UninitializedStorage => 18,
            DetectorId::Visibility => 19,
            DetectorId::CrossFunctionReentrancy => 20,
            DetectorId::ReentrancyFlow => 21,
        }
    }

    #[test]
    fn test_every_detector_id_is_registered_exactly_once() {
        for (i, id) in ALL_IDS.iter().enumerate() {
            assert_eq!(ordinal(*id), i, "ALL_IDS is out of sync with ordinal at {id}");
        }
        let mut registry = DetectorRegistry::new();
        register_all_detectors(&mut registry);
        let mut counts = [0; ALL_IDS.len()];
        for detector in registry.all() {
            counts[ordinal(detector.meta().id)] += 1;
        }
        for (id, count) in ALL_IDS.iter().zip(counts) {
            assert_eq!(count, 1, "detector '{id}' registered {count} times");
        }
    }

    #[test]
    fn test_get_looks_up_by_id_only() {
        let mut registry = DetectorRegistry::new();
        register_all_detectors(&mut registry);
        let tx_origin = registry.get("tx-origin").expect("registered by id");
        assert_eq!(tx_origin.meta().id, DetectorId::TxOrigin);
        assert!(registry.get(tx_origin.meta().name).is_none());
    }

    #[test]
    #[should_panic(expected = "registered twice")]
    fn test_register_rejects_duplicate_id() {
        let mut registry = DetectorRegistry::new();
        registry.register(Box::new(ReentrancyFlowDetector));
        registry.register(Box::new(ReentrancyFlowDetector));
    }
}
