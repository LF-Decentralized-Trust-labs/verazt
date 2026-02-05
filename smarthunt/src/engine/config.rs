//-------------------------------------------------------------------------
// Data structures Representing analysis configuration
//-------------------------------------------------------------------------

use bugs::bug::RiskLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Main configuration for SmartHunt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Enable logging
    #[serde(default)]
    pub enable_logging: bool,
    
    /// Enable IR-based analysis
    #[serde(default)]
    pub enable_ir_analysis: bool,
    
    /// Enable symbolic execution
    #[serde(default)]
    pub enable_symbolic_execution: bool,
    
    /// Max depth for symbolic execution
    #[serde(default = "default_max_symbolic_depth")]
    pub max_symbolic_depth: usize,
    
    /// Number of threads for parallel analysis
    #[serde(default = "default_num_threads")]
    pub num_threads: usize,
    
    /// Enable caching
    #[serde(default = "default_true")]
    pub cache_enabled: bool,
    
    /// Cache directory
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
    
    /// Output format
    #[serde(default)]
    pub output_format: OutputFormat,
    
    /// Output file
    #[serde(default)]
    pub output_file: Option<PathBuf>,
    
    /// Verbose output
    #[serde(default)]
    pub verbose: bool,
    
    /// Minimum severity level to report
    #[serde(default)]
    pub min_severity: SeverityFilter,
    
    /// Exclude informational issues
    #[serde(default)]
    pub exclude_informational: bool,
    
    /// Exclude optimization issues
    #[serde(default)]
    pub exclude_optimization: bool,
    
    /// Detector configuration
    #[serde(default)]
    pub detectors: DetectorConfig,
    
    /// Ignore configuration
    #[serde(default)]
    pub ignore: IgnoreConfig,
    
    // Legacy flags for backward compatibility
    #[serde(default)]
    pub check_bug_reentrancy: bool,
}

fn default_max_symbolic_depth() -> usize {
    5
}

fn default_num_threads() -> usize {
    num_cpus::get()
}

fn default_true() -> bool {
    true
}

/// Number of CPUs helper
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_logging: false,
            enable_ir_analysis: false,
            enable_symbolic_execution: false,
            max_symbolic_depth: 5,
            num_threads: num_cpus::get(),
            cache_enabled: true,
            cache_dir: None,
            output_format: OutputFormat::default(),
            output_file: None,
            verbose: false,
            min_severity: SeverityFilter::default(),
            exclude_informational: false,
            exclude_optimization: false,
            detectors: DetectorConfig::default(),
            ignore: IgnoreConfig::default(),
            check_bug_reentrancy: true,
        }
    }
}

impl Config {
    /// Create a new default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a TOML file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Self::from_toml(&content)
    }

    /// Parse configuration from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        toml::from_str(content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Check if a detector is enabled.
    pub fn is_detector_enabled(&self, detector_id: &str) -> bool {
        // Check if explicitly disabled
        if self.detectors.disabled.contains(detector_id) {
            return false;
        }
        
        // Check if only specific detectors are enabled
        if !self.detectors.enabled.is_empty() {
            return self.detectors.enabled.contains(detector_id);
        }
        
        true
    }

    /// Check if a severity level should be reported.
    pub fn should_report_severity(&self, level: &RiskLevel) -> bool {
        let level_order = match level {
            RiskLevel::Critical => 4,
            RiskLevel::High => 3,
            RiskLevel::Medium => 2,
            RiskLevel::Low => 1,
            RiskLevel::No => 0,
        };
        
        let min_order = match self.min_severity {
            SeverityFilter::Critical => 4,
            SeverityFilter::High => 3,
            SeverityFilter::Medium => 2,
            SeverityFilter::Low => 1,
            SeverityFilter::Informational => 0,
        };
        
        level_order >= min_order
    }
}

/// Output format for reports.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Markdown,
    Sarif,
    Text,
}

/// Severity filter.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SeverityFilter {
    Critical,
    High,
    #[default]
    Medium,
    Low,
    Informational,
}

/// Detector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    /// Enable vulnerability detection
    #[serde(default = "default_true")]
    pub vulnerabilities: bool,
    
    /// Enable refactoring suggestions
    #[serde(default = "default_true")]
    pub refactoring: bool,
    
    /// Enable optimization suggestions
    #[serde(default = "default_true")]
    pub optimization: bool,
    
    /// Enable best practices checks
    #[serde(default = "default_true")]
    pub best_practices: bool,
    
    /// Explicitly enabled detectors
    #[serde(default)]
    pub enabled: HashSet<String>,
    
    /// Explicitly disabled detectors
    #[serde(default)]
    pub disabled: HashSet<String>,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            vulnerabilities: true,
            refactoring: true,
            optimization: true,
            best_practices: true,
            enabled: HashSet::new(),
            disabled: HashSet::new(),
        }
    }
}

/// Ignore configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IgnoreConfig {
    /// Files to ignore (glob patterns)
    #[serde(default)]
    pub files: Vec<String>,
    
    /// Specific ignore rules
    #[serde(default)]
    pub rules: Vec<IgnoreRule>,
}

/// A specific ignore rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRule {
    /// Detector ID to ignore
    pub detector: String,
    
    /// File pattern (optional)
    #[serde(default)]
    pub file: Option<String>,
    
    /// Function name (optional)
    #[serde(default)]
    pub function: Option<String>,
    
    /// Reason for ignoring
    #[serde(default)]
    pub reason: Option<String>,
}

/// Configuration errors.
#[derive(Debug, Clone)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.cache_enabled);
        assert!(!config.enable_ir_analysis);
    }

    #[test]
    fn test_config_from_toml() {
        let toml = r#"
            enable_ir_analysis = true
            verbose = true
            
            [detectors]
            vulnerabilities = true
            optimization = false
        "#;
        
        let config = Config::from_toml(toml).unwrap();
        assert!(config.enable_ir_analysis);
        assert!(config.verbose);
        assert!(config.detectors.vulnerabilities);
        assert!(!config.detectors.optimization);
    }

    #[test]
    fn test_detector_enabled() {
        let mut config = Config::default();
        assert!(config.is_detector_enabled("reentrancy"));
        
        config.detectors.disabled.insert("reentrancy".to_string());
        assert!(!config.is_detector_enabled("reentrancy"));
    }
}

