//! Deprecated Features Detector (GREP-based)
//!
//! Detects usage of deprecated Solidity features using pattern matching.

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

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for DeprecatedGrepDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::Deprecated
    }

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

        let empty = vec![];
        let source_units: &Vec<SourceUnit> = context
            .get::<crate::artifacts::SourceUnitsArtifact>()
            .unwrap_or(&empty);

        let ctx = MatchContext::new();
        let results = matcher.match_all(source_units, &ctx);

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

    fn bug_category(&self) -> BugCategory {
        BugCategory::CodeQuality
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
        assert_eq!(detector.detector_id(), DetectorId::Deprecated);
        assert_eq!(detector.swc_ids(), vec![111]);
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
