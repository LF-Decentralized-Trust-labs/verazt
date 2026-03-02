//! JSON output formatter.

use crate::output::formatter::{AnalysisReport, OutputFormatter};
use bugs::bug::Bug;
use serde::{Deserialize, Serialize};

/// JSON output formatter.
#[derive(Debug, Default)]
pub struct JsonFormatter {
    /// Whether to pretty print the output.
    pub pretty: bool,
}

impl JsonFormatter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, report: &AnalysisReport) -> String {
        let json_report = JsonReport::from(report);
        if self.pretty {
            serde_json::to_string_pretty(&json_report)
                .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
        } else {
            serde_json::to_string(&json_report)
                .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
        }
    }

    fn extension(&self) -> &'static str {
        "json"
    }

    fn content_type(&self) -> &'static str {
        "application/json"
    }
}

/// JSON-serializable report structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonReport {
    /// SmartHunt version
    pub version: String,

    /// Analysis timestamp
    pub timestamp: String,

    /// Analysis duration in milliseconds
    pub duration_ms: u64,

    /// Files analyzed
    pub files_analyzed: Vec<String>,

    /// Summary statistics
    pub summary: JsonSummary,

    /// All findings
    pub findings: Vec<JsonFinding>,
}

/// Summary statistics.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonSummary {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
}

/// Individual finding.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub category: String,
    pub location: JsonLocation,
    pub swc_id: Option<String>,
    pub cwe_id: Option<String>,
    pub confidence: String,
}

/// Location information.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonLocation {
    pub file: Option<String>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub start_column: Option<usize>,
    pub end_column: Option<usize>,
}

impl From<&AnalysisReport> for JsonReport {
    fn from(report: &AnalysisReport) -> Self {
        Self {
            version: report.version.clone(),
            timestamp: report.timestamp.to_rfc3339(),
            duration_ms: report.duration.as_millis() as u64,
            files_analyzed: report.files_analyzed.clone(),
            summary: JsonSummary {
                total: report.bugs.len(),
                critical: report.stats.bugs_by_severity.critical,
                high: report.stats.bugs_by_severity.high,
                medium: report.stats.bugs_by_severity.medium,
                low: report.stats.bugs_by_severity.low,
                info: report.stats.bugs_by_severity.info,
            },
            findings: report.bugs.iter().map(JsonFinding::from).collect(),
        }
    }
}

impl From<&Bug> for JsonFinding {
    fn from(bug: &Bug) -> Self {
        Self {
            id: bug.kind.as_str().to_lowercase().replace(' ', "-"),
            title: bug.name.clone(),
            description: bug.description.clone().unwrap_or_default(),
            severity: bug.risk_level.as_str().to_string(),
            category: bug.kind.as_str().to_string(),
            location: JsonLocation {
                file: None,
                start_line: Some(bug.loc.start_line),
                end_line: Some(bug.loc.end_line),
                start_column: Some(bug.loc.start_col),
                end_column: Some(bug.loc.end_col),
            },
            swc_id: bug.swc_ids.first().map(|id| format!("SWC-{}", id)),
            cwe_id: bug.cwe_ids.first().map(|id| format!("CWE-{}", id)),
            confidence: "high".to_string(), // Default confidence
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_json_formatter() {
        let report = AnalysisReport::new(vec![], vec![], Duration::from_secs(1));
        let formatter = JsonFormatter::new(true);
        let output = formatter.format(&report);
        assert!(output.contains("\"version\""));
        assert!(output.contains("\"findings\""));
    }
}
