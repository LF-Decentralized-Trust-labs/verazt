//! Deprecated Features Detector (GREP-based)
//!
//! Detects usage of deprecated Solidity features using pattern matching.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use crate::grep::{MatchContext, PatternBuilder, PatternMatcher};
use bugs::bug::{Bug, BugKind, RiskLevel};

/// Known deprecated function names in Solidity.
#[allow(dead_code)]
const DEPRECATED_FUNCTIONS: &[&str] = &[
    "suicide",         // replaced by selfdestruct
    "sha3",            // replaced by keccak256
    "block.blockhash", // replaced by blockhash()
    "callcode",        // replaced by delegatecall
];

/// GREP-based detector for deprecated features.
#[derive(Debug, Default)]
pub struct DeprecatedGrepDetector;

impl DeprecatedGrepDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for DeprecatedGrepDetector {
    fn id(&self) -> PassId {
        PassId::Deprecated
    }

    fn name(&self) -> &'static str {
        "Deprecated Features"
    }

    fn description(&self) -> &'static str {
        "Detects usage of deprecated Solidity constructs."
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

impl BugDetectionPass for DeprecatedGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        let mut matcher = PatternMatcher::new();

        // Match deprecated function names
        matcher.add_pattern("suicide", PatternBuilder::ident("suicide"));
        matcher.add_pattern("sha3", PatternBuilder::ident("sha3"));
        matcher.add_pattern("callcode", PatternBuilder::member(PatternBuilder::any(), "callcode"));
        matcher.add_pattern(
            "block_blockhash",
            PatternBuilder::member(PatternBuilder::ident("block"), "blockhash"),
        );

        let ctx = MatchContext::new();
        let results = matcher.match_all(&context.source_units, &ctx);

        let replacements = [
            ("suicide", "selfdestruct"),
            ("sha3", "keccak256"),
            ("callcode", "delegatecall"),
            ("block_blockhash", "blockhash()"),
        ];

        for (name, replacement) in &replacements {
            if let Some(matches) = results.get(*name) {
                for m in matches {
                    if let Some(loc) = m.loc {
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Deprecated function '{}' used. Use '{}' instead.",
                                name, replacement
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
            }
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Refactoring
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![111] // SWC-111: Use of Deprecated Solidity Functions
    }

    fn recommendation(&self) -> &'static str {
        "Replace deprecated features with their modern equivalents."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-111"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecated_grep_detector() {
        let detector = DeprecatedGrepDetector::new();
        assert_eq!(detector.id(), PassId::Deprecated);
        assert_eq!(detector.swc_ids(), vec![111]);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
