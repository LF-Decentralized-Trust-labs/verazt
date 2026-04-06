use crate::detector::{DetectionLevel, ScanDetector};
use bugs::bug::Bug;
use scirs::sir::{Decl, MemberDecl, Module};
use std::time::Instant;

/// Configuration for the scan engine.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Enable timing information.
    pub timing: bool,
    /// Enable verbose logging.
    pub verbose: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            timing: true,
            verbose: false,
        }
    }
}

/// Result of a scan run.
#[derive(Debug)]
pub struct ScanReport {
    pub bugs: Vec<Bug>,
    pub detectors_run: usize,
    pub duration: std::time::Duration,
}

/// The scan engine: single-pass AST walker with level-based dispatch.
pub struct ScanEngine {
    _config: ScanConfig,
    /// Detectors grouped by detection level for O(1) dispatch.
    module_detectors: Vec<Box<dyn ScanDetector>>,
    contract_detectors: Vec<Box<dyn ScanDetector>>,
    function_detectors: Vec<Box<dyn ScanDetector>>,
}

impl ScanEngine {
    pub fn new(config: ScanConfig, detectors: Vec<Box<dyn ScanDetector>>) -> Self {
        let mut module_detectors = Vec::new();
        let mut contract_detectors = Vec::new();
        let mut function_detectors = Vec::new();

        for d in detectors {
            match d.level() {
                DetectionLevel::Module => module_detectors.push(d),
                DetectionLevel::Contract => contract_detectors.push(d),
                DetectionLevel::Function => function_detectors.push(d),
            }
        }

        Self {
            _config: config,
            module_detectors,
            contract_detectors,
            function_detectors,
        }
    }

    /// Run all detectors on the given SIR modules.
    /// Performs exactly ONE walk of the SIR hierarchy.
    pub fn run(&self, modules: &[Module]) -> ScanReport {
        let start = Instant::now();
        let mut bugs = Vec::new();
        let detectors_run = self.module_detectors.len()
            + self.contract_detectors.len()
            + self.function_detectors.len();

        for module in modules {
            // ── Module-level detectors ──────────────────
            for d in &self.module_detectors {
                bugs.extend(d.check_module(module));
            }

            // ── Walk contracts ──────────────────────────
            for decl in &module.decls {
                if let Decl::Contract(contract) = decl {
                    // ── Contract-level detectors ────────
                    for d in &self.contract_detectors {
                        bugs.extend(d.check_contract(contract, module));
                    }

                    // ── Walk functions ──────────────────
                    for member in &contract.members {
                        if let MemberDecl::Function(func) = member {
                            // ── Function-level detectors
                            for d in &self.function_detectors {
                                bugs.extend(d.check_function(func, contract, module));
                            }
                        }
                    }
                }
            }
        }

        ScanReport {
            bugs,
            detectors_run,
            duration: start.elapsed(),
        }
    }
}
