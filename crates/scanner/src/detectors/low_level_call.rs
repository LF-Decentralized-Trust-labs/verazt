//! Low-Level Call Detector (GREP-based)
//!
//! Detects usage of low-level calls like call, delegatecall, staticcall
//! using declarative pattern matching.

use crate::detector::id::DetectorId;
use crate::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use crate::engines::pattern::{MatchContext, PatternBuilder, PatternMatcher};
use analysis::context::AnalysisContext;
use analysis::pass::Pass;
use analysis::pass::meta::PassLevel;
use analysis::pass::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::SourceUnit;
use std::any::TypeId;

/// GREP-based detector for low-level calls.
#[derive(Debug, Default)]
pub struct LowLevelCallGrepDetector;

impl LowLevelCallGrepDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for LowLevelCallGrepDetector {
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

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for LowLevelCallGrepDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::LowLevelCall
    }

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

        let empty = vec![];
        let source_units: &Vec<SourceUnit> = context
            .get::<crate::artifacts::SourceUnitsArtifact>()
            .unwrap_or(&empty);

        let ctx = MatchContext::new();
        let results = matcher.match_all(source_units, &ctx);

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

    fn bug_category(&self) -> BugCategory {
        BugCategory::UncheckedLowLevelCalls
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
        assert_eq!(detector.detector_id(), DetectorId::LowLevelCall);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
