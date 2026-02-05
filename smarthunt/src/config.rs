//! Configuration module for SmartHunt CLI
//!
//! Provides basic configuration for the CLI tool.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
    Sarif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityFilter {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DetectorConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            enabled: vec![],
            disabled: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub num_threads: usize,
    pub output_format: OutputFormat,
    pub min_severity: SeverityFilter,
    pub detectors: DetectorConfig,
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
    pub fn from_file(_path: &Path) -> Result<Self, String> {
        // TODO: Implement config file loading
        // For now, just return default
        Ok(Self::default())
    }

    pub fn is_detector_enabled(&self, id: &str) -> bool {
        // If enabled list is specified, only those are enabled
        if !self.detectors.enabled.is_empty() {
            return self.detectors.enabled.iter().any(|d| d == id);
        }

        // Check if it's disabled
        if self.detectors.disabled.iter().any(|d| d == id) {
            return false;
        }

        // Otherwise, it's enabled
        true
    }

    pub fn should_report_severity(&self, severity: &bugs::bug::RiskLevel) -> bool {
        use bugs::bug::RiskLevel;

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

    pub fn should_report_category(&self, _category: &bugs::bug::BugKind) -> bool {
        // For now, report all categories
        true
    }
}
