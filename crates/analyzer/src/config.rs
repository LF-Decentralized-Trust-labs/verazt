//! Configuration module for Verazt Analyzer CLI
//!
//! Provides the CLI configuration and its TOML file format, as written by
//! `verazt init-config`.

use bugs::bug::RiskLevel;
use clap::ValueEnum;
use serde::Deserialize;
use std::path::Path;

// Re-export InputLanguage from the analysis crate so existing code using
// `crate::config::InputLanguage` continues to work without changes.
pub use crate::context::InputLanguage;

/// The configuration file written by `init-config`, listing every key
/// [`Config::from_file`] reads.
pub const DEFAULT_CONFIG_TOML: &str = r#"# Verazt Configuration File

[analysis]
# Enable parallel analysis
parallel = true
# Maximum number of worker threads (0 = auto-detect)
max_workers = 0

[detectors]
# Explicitly enable specific detectors (empty = all enabled)
# enabled = ["reentrancy", "tx-origin"]

# Explicitly disable specific detectors
# disabled = []

[output]
# Output format: "text", "json", "markdown", "sarif"
format = "text"
# Minimum severity to report: "info", "low", "medium", "high", "critical"
min_severity = "info"
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Text,
    Json,
    #[serde(alias = "md")]
    #[value(alias = "md")]
    Markdown,
    Sarif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SeverityFilter {
    #[serde(rename = "info")]
    #[value(name = "info")]
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DetectorConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub num_threads: usize,
    pub output_format: OutputFormat,
    pub min_severity: SeverityFilter,
    pub detectors: DetectorConfig,
}

/// Layout of a configuration file.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ConfigFile {
    analysis: AnalysisSection,
    detectors: DetectorConfig,
    output: OutputSection,
}

/// The `[analysis]` section of a configuration file.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct AnalysisSection {
    parallel: bool,
    /// Worker threads when `parallel` is set; 0 means one per CPU.
    max_workers: usize,
}

/// The `[output]` section of a configuration file.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct OutputSection {
    format: Option<OutputFormat>,
    min_severity: Option<SeverityFilter>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_threads: 1,
            output_format: OutputFormat::Text,
            min_severity: SeverityFilter::Informational,
            detectors: DetectorConfig::default(),
        }
    }
}

impl Config {
    /// Load the configuration file at `path`. Keys it omits keep their
    /// default values.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;
        Self::from_toml(&text).map_err(|e| format!("invalid config '{}': {e}", path.display()))
    }

    /// Parse a configuration from TOML text.
    fn from_toml(text: &str) -> Result<Self, String> {
        let file: ConfigFile = toml::from_str(text).map_err(|e| e.to_string())?;
        let defaults = Self::default();
        let num_threads = match (file.analysis.parallel, file.analysis.max_workers) {
            (false, _) => defaults.num_threads,
            (true, 0) => available_threads(),
            (true, n) => n,
        };
        Ok(Self {
            num_threads,
            output_format: file.output.format.unwrap_or(defaults.output_format),
            min_severity: file.output.min_severity.unwrap_or(defaults.min_severity),
            detectors: file.detectors,
        })
    }

    pub fn should_report_severity(&self, severity: &RiskLevel) -> bool {
        let severity_level = match severity {
            RiskLevel::Critical => 5,
            RiskLevel::High => 4,
            RiskLevel::Medium => 3,
            RiskLevel::Low => 2,
            RiskLevel::No => 1,
        };

        let min_level = match self.min_severity {
            SeverityFilter::Critical => 5,
            SeverityFilter::High => 4,
            SeverityFilter::Medium => 3,
            SeverityFilter::Low => 2,
            SeverityFilter::Informational => 1,
        };

        severity_level >= min_level
    }
}

/// Number of threads the machine can run in parallel.
pub fn available_threads() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_toml_reads_every_section() {
        let config = Config::from_toml(
            r#"
            [analysis]
            parallel = true
            max_workers = 3

            [detectors]
            enabled = ["tx-origin"]
            disabled = ["dead-code"]

            [output]
            format = "sarif"
            min_severity = "high"
            "#,
        )
        .unwrap();
        assert_eq!(config.num_threads, 3);
        assert_eq!(config.detectors.enabled, ["tx-origin"]);
        assert_eq!(config.detectors.disabled, ["dead-code"]);
        assert_eq!(config.output_format, OutputFormat::Sarif);
        assert_eq!(config.min_severity, SeverityFilter::High);
    }

    #[test]
    fn test_from_toml_reads_default_config_file() {
        let config = Config::from_toml(DEFAULT_CONFIG_TOML).unwrap();
        assert_eq!(config.num_threads, available_threads());
        assert_eq!(config.min_severity, SeverityFilter::Informational);
    }

    #[test]
    fn test_from_toml_empty_is_default() {
        let config = Config::from_toml("").unwrap();
        assert_eq!(config.num_threads, 1);
        assert_eq!(config.output_format, OutputFormat::Text);
        assert_eq!(config.min_severity, SeverityFilter::Informational);
    }

    #[test]
    fn test_from_toml_rejects_unknown_keys() {
        assert!(Config::from_toml("[output]\nmin_severty = \"high\"").is_err());
    }

    #[test]
    fn test_should_report_severity() {
        let config = Config { min_severity: SeverityFilter::High, ..Config::default() };
        assert!(config.should_report_severity(&RiskLevel::Critical));
        assert!(config.should_report_severity(&RiskLevel::High));
        assert!(!config.should_report_severity(&RiskLevel::Medium));
    }
}
