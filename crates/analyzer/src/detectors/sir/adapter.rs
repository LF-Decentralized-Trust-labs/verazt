//! Adapter: wraps a `ScanDetector` as a `BugDetectionPass`
//!
//! This allows SIR scan detectors to participate in the full
//! `verazt analyze` pipeline without duplicating code.

use crate::context::AnalysisContext;
use crate::detectors::base::traits::DetectorResult;
use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{BugDetectionPass, DetectorMeta};
use crate::passes::base::Pass;
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use bugs::bug::Bug;
use scirs::sir::{Decl, MemberDecl};

/// Wraps a `ScanDetector` so it can participate in the analyzer pipeline.
pub struct ScanDetectorAdapter {
    detector: Box<dyn ScanDetector>,
}

impl ScanDetectorAdapter {
    pub fn new(detector: Box<dyn ScanDetector>) -> Self {
        Self { detector }
    }
}

impl Pass for ScanDetectorAdapter {
    fn name(&self) -> &'static str {
        self.detector.meta().name
    }

    fn description(&self) -> &'static str {
        self.detector.meta().description
    }

    fn level(&self) -> PassLevel {
        match self.detector.level() {
            DetectionLevel::Module => PassLevel::Program,
            DetectionLevel::Contract => PassLevel::Contract,
            DetectionLevel::Function => PassLevel::Function,
        }
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Sir
    }

    fn dependencies(&self) -> Vec<std::any::TypeId> {
        vec![]
    }
}

impl BugDetectionPass for ScanDetectorAdapter {
    fn meta(&self) -> &'static DetectorMeta {
        self.detector.meta()
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        if !context.has_sir() {
            return Ok(vec![]);
        }
        let modules = context.sir_units();
        let mut bugs = Vec::new();

        match self.detector.level() {
            DetectionLevel::Module => {
                for module in modules {
                    bugs.extend(self.detector.check_module(module));
                }
            }
            DetectionLevel::Contract => {
                for module in modules {
                    for decl in &module.decls {
                        if let Decl::Contract(contract) = decl {
                            bugs.extend(self.detector.check_contract(contract, module));
                        }
                    }
                }
            }
            DetectionLevel::Function => {
                for module in modules {
                    for decl in &module.decls {
                        if let Decl::Contract(contract) = decl {
                            for member in &contract.members {
                                if let MemberDecl::Function(func) = member {
                                    bugs.extend(
                                        self.detector.check_function(func, contract, module),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(bugs)
    }
}
