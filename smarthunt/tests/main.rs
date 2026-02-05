//! Integration tests for SmartHunt.

use smarthunt::{
    AnalysisContext, Config, PassScheduler, create_default_registry,
    AnalysisReport, OutputFormatter, JsonFormatter, MarkdownFormatter, SarifFormatter,
};
use solidity::compile::compile_input_file;
use std::time::Duration;

mod detectors;
mod output;
mod passes;

/// Test that the registry contains all expected detectors.
#[test]
fn test_registry_has_all_detectors() {
    let registry = create_default_registry();
    
    let expected_detectors = vec![
        "tx-origin",
        "reentrancy",
        "unchecked-call",
        "floating-pragma",
        "low-level-call",
        "shadowing",
        "uninitialized",
        "deprecated",
        "visibility",
        "dead-code",
        // New detectors added
        "cei-violation",
        "missing-access-control",
        "delegatecall",
        "centralization-risk",
        "constant-state-var",
        "timestamp-dependence",
    ];
    
    for detector_id in expected_detectors {
        assert!(
            registry.get(detector_id).is_some(),
            "Detector '{}' should be registered",
            detector_id
        );
    }
    
    assert_eq!(registry.count(), 16, "Should have 16 detectors");
}

/// Test that the config loads from TOML correctly.
#[test]
fn test_config_default() {
    let config = Config::default();
    
    assert!(config.detectors.vulnerabilities);
    assert!(config.detectors.refactoring);
    assert!(config.detectors.optimization);
}

/// Test creating an analysis report.
#[test]
fn test_analysis_report_creation() {
    let bugs = vec![];
    let files = vec!["test.sol".to_string()];
    let duration = Duration::from_secs(1);
    
    let report = AnalysisReport::new(bugs, files, duration);
    
    assert_eq!(report.total_bugs(), 0);
    assert!(!report.has_bugs());
    assert!(!report.has_high_severity());
    assert_eq!(report.files_analyzed.len(), 1);
}

/// Test JSON formatter.
#[test]
fn test_json_formatter() {
    let bugs = vec![];
    let files = vec!["test.sol".to_string()];
    let duration = Duration::from_secs(1);
    
    let report = AnalysisReport::new(bugs, files, duration);
    let formatter = JsonFormatter::new(true);
    let output = formatter.format(&report);
    
    assert!(output.contains("\"version\""));
    assert!(output.contains("\"findings\""));
    assert!(output.contains("\"summary\""));
}

/// Test Markdown formatter.
#[test]
fn test_markdown_formatter() {
    let bugs = vec![];
    let files = vec!["test.sol".to_string()];
    let duration = Duration::from_secs(1);
    
    let report = AnalysisReport::new(bugs, files, duration);
    let formatter = MarkdownFormatter::new();
    let output = formatter.format(&report);
    
    assert!(output.contains("# SmartHunt Analysis Report"));
    assert!(output.contains("No issues found"));
}

/// Test SARIF formatter.
#[test]
fn test_sarif_formatter() {
    let bugs = vec![];
    let files = vec!["test.sol".to_string()];
    let duration = Duration::from_secs(1);
    
    let report = AnalysisReport::new(bugs, files, duration);
    let formatter = SarifFormatter::new(true);
    let output = formatter.format(&report);
    
    assert!(output.contains("\"$schema\""));
    assert!(output.contains("\"version\": \"2.1.0\""));
    assert!(output.contains("SmartHunt"));
}
