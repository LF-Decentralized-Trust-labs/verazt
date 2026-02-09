//! tx.origin Detector (GREP-based)
//!
//! Detects dangerous usage of tx.origin for authentication using
//! declarative pattern matching.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use crate::grep::{MatchContext, PatternBuilder, PatternMatcher};
use bugs::bug::{Bug, BugKind, RiskLevel};

/// GREP-based detector for tx.origin usage.
///
/// Using tx.origin for authentication is vulnerable to phishing attacks.
#[derive(Debug, Default)]
pub struct TxOriginGrepDetector;

impl TxOriginGrepDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for TxOriginGrepDetector {
    fn id(&self) -> PassId {
        PassId::TxOrigin
    }

    fn name(&self) -> &'static str {
        "Dangerous use of tx.origin"
    }

    fn description(&self) -> &'static str {
        "Using tx.origin for authentication is vulnerable to phishing attacks. \
         An attacker can trick a user into calling a malicious contract that then \
         calls the vulnerable contract, and tx.origin will be the user's address."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![]
    }
}

impl BugDetectionPass for TxOriginGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        // Create pattern: tx.origin
        let pattern = PatternBuilder::tx_origin();

        let mut matcher = PatternMatcher::new();
        matcher.add_pattern("tx_origin", pattern);

        let ctx = MatchContext::new();
        let results = matcher.match_all(&context.source_units, &ctx);

        if let Some(matches) = results.get("tx_origin") {
            for m in matches {
                if let Some(loc) = m.loc {
                    let bug = create_bug(
                        self,
                        Some(
                            "tx.origin used for authentication. \
                             Consider using msg.sender instead.",
                        ),
                        loc,
                    );
                    bugs.push(bug);
                }
            }
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![345] // CWE-345: Insufficient Verification of Data Authenticity
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![115] // SWC-115: Authorization through tx.origin
    }

    fn recommendation(&self) -> &'static str {
        "Use msg.sender instead of tx.origin for authentication."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-115",
            "https://consensys.github.io/smart-contract-best-practices/development-recommendations/solidity-specific/tx-origin/",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_origin_grep_detector() {
        let detector = TxOriginGrepDetector::new();
        assert_eq!(detector.id(), PassId::TxOrigin);
        assert_eq!(detector.swc_ids(), vec![115]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
