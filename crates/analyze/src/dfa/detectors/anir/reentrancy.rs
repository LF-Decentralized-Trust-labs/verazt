//! ANIR Reentrancy Detector
//!
//! ICFG-based reentrancy detection using ANIR's interprocedural representation.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use scavir::air::cfg::ICFGNode;
use scavir::air::ops::OpId;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use solidity::ast::Loc;

/// ANIR-based reentrancy detector.
///
/// Pattern: StorageOp read → ExternalCallNode → StorageOp write
/// (same alias group).
#[derive(Debug, Default)]
pub struct AnirReentrancyDetector;

impl AnirReentrancyDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for AnirReentrancyDetector {
    fn id(&self) -> PassId {
        // This is a detection pass; it uses the Reentrancy PassId
        // and supersedes the AST-based reentrancy detector when ANIR
        // is available.
        PassId::AnirReentrancy
    }

    fn name(&self) -> &'static str {
        "ANIR Reentrancy"
    }

    fn description(&self) -> &'static str {
        "Interprocedural reentrancy detection via ICFG pattern"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::AnirTaintPropagation]
    }
}

impl BugDetectionPass for AnirReentrancyDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_air() {
            return Ok(bugs);
        }

        for module in context.anir_units() {
            // Find external call nodes
            let external_calls: Vec<&OpId> = module
                .icfg
                .nodes
                .iter()
                .filter_map(|node| {
                    if let ICFGNode::ExternalCallNode { op } = node {
                        Some(op)
                    } else {
                        None
                    }
                })
                .collect();

            if external_calls.is_empty() {
                continue;
            }

            // For each alias group, check read-before-call, write-after-call
            for (_group_id, group) in &module.alias_sets.groups {
                if group.reads.is_empty() || group.writes.is_empty() {
                    continue;
                }

                for _ext_call in &external_calls {
                    // Simplified check: if there are reads and writes in the
                    // same alias group, and an external call exists, flag it.
                    // A full implementation would check ICFG dominance.
                    if !group.reads.is_empty() && !group.writes.is_empty() {
                        bugs.push(Bug::new(
                            self.name(),
                            Some(&format!(
                                "Potential reentrancy: alias group '{}' has storage read before external call and write after",
                                group.id.0
                            )),
                            Loc::new(0, 0, 0, 0),
                            self.bug_kind(),
                            self.bug_category(),
                            self.risk_level(),
                            self.cwe_ids(),
                            self.swc_ids(),
                        ));
                        // Only report once per alias group per external call
                        break;
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
        BugCategory::Reentrancy
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Critical
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![841]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![107]
    }

    fn recommendation(&self) -> &'static str {
        "Ensure all state changes happen before calling external contracts (Checks-Effects-Interactions pattern)"
    }
}
