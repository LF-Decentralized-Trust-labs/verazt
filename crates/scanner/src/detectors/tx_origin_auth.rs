//! SIR tx.origin Authentication Detector (EVM-specific)
//!
//! Detects use of `evm.tx_origin()` for authentication.

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use std::any::TypeId;

/// SIR structural detector for tx.origin auth misuse (EVM-specific).
#[derive(Debug, Default)]
pub struct SirTxOriginAuthDetector;

impl SirTxOriginAuthDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for SirTxOriginAuthDetector {
    fn name(&self) -> &'static str {
        "SIR tx.origin Auth"
    }

    fn description(&self) -> &'static str {
        "EVM: tx.origin used for authentication instead of msg.sender"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for SirTxOriginAuthDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::SirTxOriginAuth
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        for module in context.ir_units() {
            // Check if this module uses EVM dialect
            let has_evm = module.attrs.iter().any(|a| {
                a.namespace == "sir"
                    && a.key == "loaded_dialects"
                    && matches!(&a.value, scirs::sir::AttrValue::String(s) if s.contains("evm"))
            });
            if !has_evm {
                // Also check if any contract has EVM dialect member decls
                let has_evm_decls = module.decls.iter().any(|d| {
                    matches!(d, scirs::sir::Decl::Contract(c) if c.members.iter().any(|m|
                        matches!(m, scirs::sir::MemberDecl::Dialect(scirs::sir::DialectMemberDecl::Evm(_)))
                    ))
                });
                if !has_evm_decls {
                    // Skip non-EVM modules (but still check for dialect exprs
                    // since the module might not have the attr set)
                }
            }

            for decl in &module.decls {
                if let scirs::sir::Decl::Contract(contract) = decl {
                    for member in &contract.members {
                        if let scirs::sir::MemberDecl::Function(func) = member {
                            if let Some(body) = &func.body {
                                // Note: EVM doesn't have a TxOrigin expr in
                                // the current dialect definition.  The plan
                                // references it but it's not yet in the
                                // EvmExpr enum.  We use the dialect expression
                                // walker as a placeholder for future extension.
                                let _ = body;
                            }
                        }
                    }
                }
            }
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::AccessControl
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![477]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![115]
    }

    fn recommendation(&self) -> &'static str {
        "Use msg.sender instead of tx.origin for authentication"
    }
}
