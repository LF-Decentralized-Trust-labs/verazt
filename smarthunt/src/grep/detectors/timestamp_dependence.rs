//! Timestamp Dependence Detector (GREP-based)
//!
//! Detects dangerous usage of block.timestamp for critical decisions
//! using declarative pattern matching.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use crate::grep::{MatchContext, PatternBuilder, PatternMatcher};
use bugs::bug::{Bug, BugKind, RiskLevel};

/// GREP-based detector for timestamp dependence.
///
/// Detects usage of `block.timestamp` and `now` (deprecated alias)
/// which can be manipulated by miners.
#[derive(Debug, Default)]
pub struct TimestampDependenceGrepDetector;

impl TimestampDependenceGrepDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Pass for TimestampDependenceGrepDetector {
    fn id(&self) -> PassId {
        PassId::TimestampDependence
    }

    fn name(&self) -> &'static str {
        "Timestamp Dependence"
    }

    fn description(&self) -> &'static str {
        "Detects dangerous reliance on block.timestamp."
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

impl BugDetectionPass for TimestampDependenceGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        let mut matcher = PatternMatcher::new();

        // Match block.timestamp
        matcher.add_pattern("block_timestamp", PatternBuilder::block_timestamp());

        // Match deprecated 'now' keyword
        matcher.add_pattern("now", PatternBuilder::ident("now"));

        let ctx = MatchContext::new();
        let results = matcher.match_all(&context.source_units, &ctx);

        if let Some(matches) = results.get("block_timestamp") {
            for m in matches {
                if let Some(loc) = m.loc {
                    let bug = create_bug(
                        self,
                        Some(
                            "Usage of block.timestamp detected. Miners can manipulate \
                             this value within a range of ~15 seconds.",
                        ),
                        loc,
                    );
                    bugs.push(bug);
                }
            }
        }

        if let Some(matches) = results.get("now") {
            for m in matches {
                if let Some(loc) = m.loc {
                    let bug = create_bug(
                        self,
                        Some(
                            "Usage of 'now' (alias for block.timestamp) detected. \
                             This is deprecated and can be manipulated by miners.",
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
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![829]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![116] // SWC-116: Block values as a proxy for time
    }

    fn recommendation(&self) -> &'static str {
        "Avoid using block.timestamp for critical logic or randomness."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-116"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_dependence_grep_detector() {
        let detector = TimestampDependenceGrepDetector::new();
        assert_eq!(detector.id(), PassId::TimestampDependence);
        assert_eq!(detector.swc_ids(), vec![116]);
    }
}
