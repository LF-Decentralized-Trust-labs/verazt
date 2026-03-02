//! SARIF output formatter.
//!
//! SARIF (Static Analysis Results Interchange Format) is a standard format
//! for the output of static analysis tools.

use crate::output::formatter::{AnalysisReport, OutputFormatter};
use bugs::bug::RiskLevel;
use serde::{Deserialize, Serialize};

/// SARIF output formatter.
#[derive(Debug, Default)]
pub struct SarifFormatter {
    /// Whether to pretty print the output.
    pub pretty: bool,
}

impl SarifFormatter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl OutputFormatter for SarifFormatter {
    fn format(&self, report: &AnalysisReport) -> String {
        let sarif = SarifLog::from(report);
        if self.pretty {
            serde_json::to_string_pretty(&sarif)
                .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
        } else {
            serde_json::to_string(&sarif).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
        }
    }

    fn extension(&self) -> &'static str {
        "sarif"
    }

    fn content_type(&self) -> &'static str {
        "application/sarif+json"
    }
}

/// SARIF log structure (v2.1.0).
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<SarifRun>,
}

/// A single run of analysis.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
    pub artifacts: Vec<SarifArtifact>,
    #[serde(rename = "invocations")]
    pub invocations: Vec<SarifInvocation>,
}

/// Tool information.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifTool {
    pub driver: SarifToolDriver,
}

/// Tool driver information.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifToolDriver {
    pub name: String,
    pub version: String,
    #[serde(rename = "informationUri")]
    pub information_uri: String,
    pub rules: Vec<SarifRule>,
}

/// A rule (detector).
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRule {
    pub id: String,
    pub name: String,
    #[serde(rename = "shortDescription")]
    pub short_description: SarifMessage,
    #[serde(rename = "fullDescription", skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMessage>,
    #[serde(rename = "helpUri", skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
    #[serde(rename = "defaultConfiguration")]
    pub default_configuration: SarifRuleConfiguration,
}

/// Rule configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRuleConfiguration {
    pub level: String,
}

/// A message.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifMessage {
    pub text: String,
}

/// An analysis result.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifResult {
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
}

/// A location.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    pub physical_location: SarifPhysicalLocation,
}

/// A physical location.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    pub artifact_location: SarifArtifactLocation,
    pub region: SarifRegion,
}

/// An artifact location.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    pub uri: String,
}

/// A region in a file.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRegion {
    #[serde(rename = "startLine")]
    pub start_line: usize,
    #[serde(rename = "startColumn", skip_serializing_if = "Option::is_none")]
    pub start_column: Option<usize>,
    #[serde(rename = "endLine", skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(rename = "endColumn", skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,
}

/// An artifact (source file).
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifArtifact {
    pub location: SarifArtifactLocation,
}

/// Invocation information.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifInvocation {
    #[serde(rename = "executionSuccessful")]
    pub execution_successful: bool,
    #[serde(rename = "endTimeUtc")]
    pub end_time_utc: String,
}

impl From<&AnalysisReport> for SarifLog {
    fn from(report: &AnalysisReport) -> Self {
        // Collect unique rules from bugs
        let mut rules_map = std::collections::HashMap::new();
        for bug in &report.bugs {
            let rule_id = bug.kind.as_str().to_lowercase().replace(' ', "-");
            if !rules_map.contains_key(&rule_id) {
                rules_map.insert(
                    rule_id.clone(),
                    SarifRule {
                        id: rule_id.clone(),
                        name: bug.name.clone(),
                        short_description: SarifMessage { text: bug.name.clone() },
                        full_description: bug
                            .description
                            .clone()
                            .map(|d| SarifMessage { text: d }),
                        help_uri: bug
                            .swc_ids
                            .first()
                            .map(|id| format!("https://swcregistry.io/docs/SWC-{}", id)),
                        default_configuration: SarifRuleConfiguration {
                            level: risk_level_to_sarif(&bug.risk_level),
                        },
                    },
                );
            }
        }

        let rules: Vec<_> = rules_map.into_values().collect();

        let results: Vec<_> = report
            .bugs
            .iter()
            .map(|bug| SarifResult {
                rule_id: format!("{:?}", bug.kind).to_lowercase().replace(' ', "-"),
                level: risk_level_to_sarif(&bug.risk_level),
                message: SarifMessage {
                    text: bug.description.clone().unwrap_or_else(|| bug.name.clone()),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation { uri: "unknown".to_string() },
                        region: SarifRegion {
                            start_line: bug.loc.start_line,
                            start_column: Some(bug.loc.start_col),
                            end_line: Some(bug.loc.end_line),
                            end_column: Some(bug.loc.end_col),
                        },
                    },
                }],
            })
            .collect();

        let artifacts: Vec<_> = report
            .files_analyzed
            .iter()
            .map(|f| SarifArtifact { location: SarifArtifactLocation { uri: f.clone() } })
            .collect();

        SarifLog {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifToolDriver {
                        name: "SmartHunt".to_string(),
                        version: report.version.clone(),
                        information_uri: "https://github.com/example/smarthunt".to_string(),
                        rules,
                    },
                },
                results,
                artifacts,
                invocations: vec![SarifInvocation {
                    execution_successful: true,
                    end_time_utc: report.timestamp.to_rfc3339(),
                }],
            }],
        }
    }
}

fn risk_level_to_sarif(level: &RiskLevel) -> String {
    match level {
        RiskLevel::Critical | RiskLevel::High => "error".to_string(),
        RiskLevel::Medium => "warning".to_string(),
        RiskLevel::Low | RiskLevel::No => "note".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_sarif_formatter() {
        let report = AnalysisReport::new(vec![], vec![], Duration::from_secs(1));
        let formatter = SarifFormatter::new(true);
        let output = formatter.format(&report);
        assert!(output.contains("\"$schema\""));
        assert!(output.contains("\"version\": \"2.1.0\""));
    }
}
