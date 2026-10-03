use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use scirs::sir::{ContractDecl, FunctionDecl, Module};

/// Confidence level for a scan finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::High => write!(f, "High"),
            Confidence::Medium => write!(f, "Medium"),
            Confidence::Low => write!(f, "Low"),
        }
    }
}

/// Target platform that a detector applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// Only applicable to EVM-based languages (Solidity, Vyper).
    Evm,
    /// Only applicable to Move-based languages (Sui, Aptos).
    Move,
    /// Only applicable to Solana programs.
    Solana,
}

/// The SIR hierarchy level at which a detector operates.
///
/// Inspired by `analyzer::PassLevel`, but simplified to the three
/// levels that scanner detectors actually need. The `ScanEngine`
/// uses this to dispatch detectors during its single-pass walk.
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
/// `check_*` method is called by the `ScanEngine` during its single
/// walk of the SIR tree.
pub trait ScanDetector: Send + Sync {
    // ── Identity ────────────────────────────────────────

    /// Stable kebab-case identifier (e.g., `"front-running"`).
    fn id(&self) -> &'static str;

    /// Human-readable name (e.g., `"Front Running"`).
    fn name(&self) -> &'static str;

    /// Short description of what this detector checks.
    fn description(&self) -> &'static str;

    // ── Classification ─────────────────────────────────

    /// The kind of issue this detector finds.
    fn bug_kind(&self) -> BugKind;

    /// The vulnerability category.
    fn bug_category(&self) -> BugCategory;

    /// The risk level of findings.
    fn risk_level(&self) -> RiskLevel;

    /// Confidence level of detection.
    fn confidence(&self) -> Confidence;

    /// Target platform this detector applies to.
    fn target(&self) -> Target;

    /// The SIR level at which this detector operates.
    fn level(&self) -> DetectionLevel;

    /// Associated CWE IDs.
    fn cwe_ids(&self) -> Vec<usize>;

    /// Associated SWC IDs (empty for non-EVM detectors).
    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }

    /// Fix recommendation.
    fn recommendation(&self) -> &'static str {
        ""
    }

    /// Reference URLs.
    fn references(&self) -> Vec<&'static str> {
        vec![]
    }

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
