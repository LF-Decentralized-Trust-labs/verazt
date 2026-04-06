//! Adapter: wraps `scanner::ScanDetector` → `analyzer::BugDetectionPass`
//!
//! This allows scan detectors from the `scanner` crate to participate
//! in the full `verazt analyze` pipeline without duplicating code.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::base::traits::{ConfidenceLevel, DetectorResult};
use crate::detectors::BugDetectionPass;
use crate::passes::base::Pass;
use crate::passes::base::meta::{PassLevel, PassRepresentation};
use bugs::bug::Bug;
use scanner::detector::{Confidence, DetectionLevel};
use scanner::ScanDetector;
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
        self.detector.name()
    }

    fn description(&self) -> &'static str {
        self.detector.description()
    }

    fn level(&self) -> PassLevel {
        match self.detector.level() {
            DetectionLevel::Module => PassLevel::Program,
            DetectionLevel::Contract => PassLevel::Contract,
            DetectionLevel::Function => PassLevel::Function,
        }
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<std::any::TypeId> {
        vec![]
    }
}

impl BugDetectionPass for ScanDetectorAdapter {
    fn detector_id(&self) -> DetectorId {
        DetectorId::from_str(self.detector.id())
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        if !context.has_ir() {
            return Ok(vec![]);
        }
        let modules = context.ir_units();
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
                                        self.detector
                                            .check_function(func, contract, module),
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

    fn bug_kind(&self) -> bugs::bug::BugKind {
        self.detector.bug_kind()
    }

    fn bug_category(&self) -> bugs::bug::BugCategory {
        self.detector.bug_category()
    }

    fn risk_level(&self) -> bugs::bug::RiskLevel {
        self.detector.risk_level()
    }

    fn confidence(&self) -> ConfidenceLevel {
        match self.detector.confidence() {
            Confidence::Low => ConfidenceLevel::Low,
            Confidence::Medium => ConfidenceLevel::Medium,
            Confidence::High => ConfidenceLevel::High,
        }
    }

    fn cwe_ids(&self) -> Vec<usize> {
        self.detector.cwe_ids()
    }

    fn swc_ids(&self) -> Vec<usize> {
        self.detector.swc_ids()
    }

    fn recommendation(&self) -> &'static str {
        self.detector.recommendation()
    }

    fn references(&self) -> Vec<&'static str> {
        self.detector.references()
    }
}
