//! Delegatecall Detector (GREP-based)
//!
//! Detects dangerous usage of delegatecall using pattern matching.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use crate::grep::{MatchContext, PatternBuilder, PatternMatcher};
use bugs::bug::{Bug, BugKind, RiskLevel};

/// GREP-based detector for delegatecall usage.
///
/// Delegatecall to untrusted addresses can lead to storage corruption
/// and complete contract compromise.
#[derive(Debug, Default)]
pub struct DelegatecallGrepDetector;

impl DelegatecallGrepDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for DelegatecallGrepDetector {
    fn id(&self) -> PassId {
        PassId::Delegatecall
    }

    fn name(&self) -> &'static str {
        "Dangerous Delegatecall"
    }

    fn description(&self) -> &'static str {
        "Detects potentially dangerous delegatecall usage."
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

impl BugDetectionPass for DelegatecallGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        let mut matcher = PatternMatcher::new();

        // Match any .delegatecall() usage
        matcher.add_pattern(
            "delegatecall",
            PatternBuilder::member(PatternBuilder::any(), "delegatecall"),
        );

        let ctx = MatchContext::new();
        let results = matcher.match_all(&context.source_units, &ctx);

        if let Some(matches) = results.get("delegatecall") {
            for m in matches {
                if let Some(loc) = m.loc {
                    let bug = create_bug(
                        self,
                        Some(
                            "Usage of delegatecall detected. Delegatecall to an untrusted \
                             address can lead to storage corruption and contract compromise.",
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
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![112] // SWC-112: Delegatecall to Untrusted Callee
    }

    fn recommendation(&self) -> &'static str {
        "Verify the target contract is trusted and update state variables carefully. \
         Consider using a library pattern instead of direct delegatecall."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-112"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegatecall_grep_detector() {
        let detector = DelegatecallGrepDetector::new();
        assert_eq!(detector.id(), PassId::Delegatecall);
        assert_eq!(detector.swc_ids(), vec![112]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
