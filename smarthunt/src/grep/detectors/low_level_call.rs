//! Low-Level Call Detector (GREP-based)
//!
//! Detects usage of low-level calls like call, delegatecall, staticcall
//! using declarative pattern matching.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use crate::grep::{MatchContext, PatternBuilder, PatternMatcher};
use bugs::bug::{Bug, BugKind, RiskLevel};

/// GREP-based detector for low-level calls.
#[derive(Debug, Default)]
pub struct LowLevelCallGrepDetector;

impl LowLevelCallGrepDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for LowLevelCallGrepDetector {
    fn id(&self) -> PassId {
        PassId::LowLevelCall
    }

    fn name(&self) -> &'static str {
        "Low-Level Calls"
    }

    fn description(&self) -> &'static str {
        "Detects usage of low-level calls that may be dangerous."
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

impl BugDetectionPass for LowLevelCallGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        let mut matcher = PatternMatcher::new();

        // Match .call(), .delegatecall(), .staticcall()
        matcher.add_pattern("call", PatternBuilder::member(PatternBuilder::any(), "call"));
        matcher.add_pattern(
            "delegatecall",
            PatternBuilder::member(PatternBuilder::any(), "delegatecall"),
        );
        matcher.add_pattern(
            "staticcall",
            PatternBuilder::member(PatternBuilder::any(), "staticcall"),
        );

        let ctx = MatchContext::new();
        let results = matcher.match_all(&context.source_units, &ctx);

        for (name, matches) in &results {
            for m in matches {
                if let Some(loc) = m.loc {
                    let bug = create_bug(
                        self,
                        Some(&format!(
                            "Low-level '{}' detected. Consider using higher-level \
                             function calls when possible.",
                            name,
                        )),
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
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn recommendation(&self) -> &'static str {
        "Avoid low-level calls. If necessary, ensure proper checks and handling."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://docs.soliditylang.org/en/latest/units-and-global-variables.html#members-of-address-types",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_level_call_grep_detector() {
        let detector = LowLevelCallGrepDetector::new();
        assert_eq!(detector.id(), PassId::LowLevelCall);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
