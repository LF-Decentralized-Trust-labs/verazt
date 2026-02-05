//! Confidence level for detector findings.

use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};

/// Confidence level for a detector finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    /// Low confidence - may have false positives
    Low,
    /// Medium confidence - likely true positive
    Medium,
    /// High confidence - almost certainly true positive
    High,
}

impl Display for ConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfidenceLevel::Low => write!(f, "Low"),
            ConfidenceLevel::Medium => write!(f, "Medium"),
            ConfidenceLevel::High => write!(f, "High"),
        }
    }
}

impl Default for ConfidenceLevel {
    fn default() -> Self {
        ConfidenceLevel::Medium
    }
}
