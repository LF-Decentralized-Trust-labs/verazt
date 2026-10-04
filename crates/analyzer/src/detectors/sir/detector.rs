use crate::detectors::DetectorMeta;
use bugs::bug::Bug;
use scirs::sir::{ContractDecl, FunctionDecl, Module};

/// The SIR hierarchy level at which a detector operates.
///
/// Inspired by `analyzer::PassLevel`, but simplified to the three
/// levels that scanner detectors actually need. `ScanDetectorAdapter`
/// uses this to dispatch detectors while walking the SIR tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionLevel {
    /// Operates on whole modules (e.g., pragma checks).
    Module,
    /// Operates on individual contracts (e.g., access control, state vars).
    Contract,
    /// Operates on individual functions (e.g., reentrancy, front-running).
    Function,
}

/// A lightweight scan detector that operates on SIR at a specific level.
///
/// Unlike `analyzer::BugDetectionPass`, this trait has no dependency on
/// `Pass`, `AnalysisContext`, or any analysis framework.
///
/// Each detector declares its `level()`, and only the corresponding
/// `check_*` method is called by `ScanDetectorAdapter` as it walks the
/// SIR tree.
pub trait ScanDetector: Send + Sync {
    /// The detector's identity, classification, and guidance.
    fn meta(&self) -> &'static DetectorMeta;

    /// The SIR level at which this detector operates.
    fn level(&self) -> DetectionLevel;

    // ── Detection (only one is called, based on level()) ──

    /// Check a module. Called when `level() == Module`.
    fn check_module(&self, _module: &Module) -> Vec<Bug> {
        vec![]
    }

    /// Check a contract. Called when `level() == Contract`.
    fn check_contract(&self, _contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        vec![]
    }

    /// Check a function. Called when `level() == Function`.
    fn check_function(
        &self,
        _func: &FunctionDecl,
        _contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        vec![]
    }
}
