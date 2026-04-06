use crate::detector::{ScanDetector, Target};

/// Registry for scan detectors.
pub struct ScanRegistry {
    detectors: Vec<Box<dyn ScanDetector>>,
}

impl ScanRegistry {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    pub fn register(&mut self, detector: Box<dyn ScanDetector>) {
        self.detectors.push(detector);
    }

    pub fn all(&self) -> &[Box<dyn ScanDetector>] {
        &self.detectors
    }

    /// Filter detectors by target platform.
    pub fn for_target(&self, target: Target) -> Vec<&dyn ScanDetector> {
        self.detectors
            .iter()
            .filter(|d| d.target() == target)
            .map(|d| d.as_ref())
            .collect()
    }

    /// Consume the registry and return owned detectors (for adapter use).
    pub fn into_detectors(self) -> Vec<Box<dyn ScanDetector>> {
        self.detectors
    }

    pub fn len(&self) -> usize {
        self.detectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.detectors.is_empty()
    }
}

impl Default for ScanRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register all built-in scan detectors.
pub fn register_all_detectors(registry: &mut ScanRegistry) {
    use crate::detectors::*;

    // ── Security: EVM ───────────────────────────────────────────
    registry.register(Box::new(ArithmeticOverflowDetector::new()));
    registry.register(Box::new(BadRandomnessDetector::new()));
    registry.register(Box::new(CeiViolationDetector::new()));
    registry.register(Box::new(CentralizationRiskDetector::new()));
    registry.register(Box::new(DelegatecallDetector::new()));
    registry.register(Box::new(DenialOfServiceDetector::new()));
    registry.register(Box::new(FrontRunningDetector::new()));
    registry.register(Box::new(LowLevelCallDetector::new()));
    registry.register(Box::new(MissingAccessControlDetector::new()));
    registry.register(Box::new(ReentrancyDetector::new()));
    registry.register(Box::new(ShortAddressDetector::new()));
    registry.register(Box::new(TimestampDependenceDetector::new()));
    registry.register(Box::new(TxOriginDetector::new()));
    registry.register(Box::new(UncheckedCallDetector::new()));
    registry.register(Box::new(UninitializedDetector::new()));

    // ── Quality: EVM ────────────────────────────────────────────
    registry.register(Box::new(ConstantStateVarDetector::new()));
    registry.register(Box::new(DeadCodeDetector::new()));
    registry.register(Box::new(DeprecatedFeaturesDetector::new()));
    registry.register(Box::new(FloatingPragmaDetector::new()));
    registry.register(Box::new(ShadowingDetector::new()));
    registry.register(Box::new(VisibilityDetector::new()));
}
