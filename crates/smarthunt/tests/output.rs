//! Unit tests for output formatters.

use smarthunt::output::{
    AnalysisReport, JsonFormatter, MarkdownFormatter, OutputFormatter, SarifFormatter,
};
use std::time::Duration;

#[test]
fn test_json_formatter_extension() {
    let formatter = JsonFormatter::new(false);
    assert_eq!(formatter.extension(), "json");
    assert_eq!(formatter.content_type(), "application/json");
}

#[test]
fn test_markdown_formatter_extension() {
    let formatter = MarkdownFormatter::new();
    assert_eq!(formatter.extension(), "md");
    assert_eq!(formatter.content_type(), "text/markdown");
}

#[test]
fn test_sarif_formatter_extension() {
    let formatter = SarifFormatter::new(false);
    assert_eq!(formatter.extension(), "sarif");
    assert_eq!(formatter.content_type(), "application/sarif+json");
}

#[test]
fn test_json_formatter_compact() {
    let report = AnalysisReport::new(vec![], vec![], Duration::from_millis(100));
    let formatter = JsonFormatter::new(false);
    let output = formatter.format(&report);

    // Compact JSON should not have newlines
    assert!(!output.contains("  "), "Compact JSON should not have indentation");
}

#[test]
fn test_json_formatter_pretty() {
    let report = AnalysisReport::new(vec![], vec![], Duration::from_millis(100));
    let formatter = JsonFormatter::new(true);
    let output = formatter.format(&report);

    // Pretty JSON should have newlines
    assert!(output.contains("\n"), "Pretty JSON should have newlines");
}

#[test]
fn test_report_stats() {
    let report = AnalysisReport::new(
        vec![],
        vec!["file1.sol".to_string(), "file2.sol".to_string()],
        Duration::from_secs(5),
    );

    assert_eq!(report.files_analyzed.len(), 2);
    assert_eq!(report.duration.as_secs(), 5);
    assert_eq!(report.stats.bugs_by_severity.critical, 0);
    assert_eq!(report.stats.bugs_by_severity.high, 0);
    assert_eq!(report.stats.bugs_by_severity.medium, 0);
    assert_eq!(report.stats.bugs_by_severity.low, 0);
    assert_eq!(report.stats.bugs_by_severity.info, 0);
}
