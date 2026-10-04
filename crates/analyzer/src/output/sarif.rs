//! SARIF output formatter.
//!
//! SARIF (Static Analysis Results Interchange Format) is a standard format
//! for the output of static analysis tools.

use crate::detectors::DetectorMeta;
use crate::output::formatter::{AnalysisReport, OutputFormatter, builtin_detector_metas};
use bugs::bug::RiskLevel;
use common::loc::Loc;
use serde::{Deserialize, Serialize};

/// Project home page, reported as the SARIF tool's `informationUri`.
const INFORMATION_URI: &str = "https://github.com/taquangtrung/verazt";

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<SarifMessage>,
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
    /// Index of the rule in `tool.driver.rules`, absent for detectors that
    /// are not built in.
    #[serde(rename = "ruleIndex", skip_serializing_if = "Option::is_none")]
    pub rule_index: Option<usize>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<SarifRegion>,
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
        // One rule per built-in detector, sorted by ID; results point at
        // their rule by ID and index.
        let rules: Vec<_> = builtin_detector_metas().into_iter().map(SarifRule::from).collect();

        let results: Vec<_> = report
            .bugs
            .iter()
            .map(|bug| SarifResult {
                rule_id: bug.detector_id.clone(),
                rule_index: rules.iter().position(|r| r.id == bug.detector_id),
                level: risk_level_to_sarif(&bug.risk_level),
                message: SarifMessage {
                    text: bug.description.clone().unwrap_or_else(|| bug.name.clone()),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: bug
                                .loc
                                .file
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                        },
                        region: SarifRegion::of(&bug.loc),
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
                        name: "Verazt Analyzer".to_string(),
                        version: report.version.clone(),
                        information_uri: INFORMATION_URI.to_string(),
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

impl From<&DetectorMeta> for SarifRule {
    fn from(meta: &DetectorMeta) -> Self {
        Self {
            id: meta.id.as_str().to_string(),
            name: meta.name.to_string(),
            short_description: SarifMessage { text: meta.name.to_string() },
            full_description: Some(SarifMessage { text: meta.description.to_string() }),
            help: Some(SarifMessage { text: meta.recommendation.to_string() }),
            help_uri: meta
                .swc_ids
                .first()
                .map(|id| format!("https://swcregistry.io/docs/SWC-{}", id)),
            default_configuration: SarifRuleConfiguration {
                level: risk_level_to_sarif(&meta.risk_level),
            },
        }
    }
}

impl SarifRegion {
    /// The region of `loc`, or `None` when `loc` carries no line. SARIF
    /// lines and columns are 1-based, so zero (unknown) values are omitted.
    fn of(loc: &Loc) -> Option<Self> {
        let known = |n: usize| (n > 0).then_some(n);
        loc.is_valid().then(|| Self {
            start_line: loc.start_line,
            start_column: known(loc.start_col),
            end_line: known(loc.end_line),
            end_column: known(loc.end_col),
        })
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
