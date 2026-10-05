//! Unit tests for output formatters.

use analyzer::output::{
    AnalysisReport, JsonFormatter, MarkdownFormatter, OutputFormatter, SarifFormatter,
};
use analyzer::{DetectorRegistry, register_all_detectors};
use common::loc::Loc;
use serde_json::Value;
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

/// A report of one `tx-origin` finding at a known span and one
/// `floating-pragma` finding without a span, as the detectors emit them.
fn report_with_findings() -> AnalysisReport {
    let mut registry = DetectorRegistry::new();
    register_all_detectors(&mut registry);
    let meta_of = |id: &str| registry.get(id).expect("built-in detector").meta();
    let loc = Loc::new(3, 5, 3, 20).with_file("a.sol".to_string());
    let bugs = vec![
        meta_of("tx-origin").bug(Some("uses tx.origin"), loc),
        meta_of("floating-pragma").bug(None, Loc::default()),
    ];
    AnalysisReport::new(bugs, vec!["a.sol".to_string()], Duration::from_millis(1))
}

#[test]
fn test_sarif_results_reference_detector_rules() {
    let output = SarifFormatter::new(false).format(&report_with_findings());
    let sarif: Value = serde_json::from_str(&output).unwrap();
    let run = &sarif["runs"][0];
    let rules = run["tool"]["driver"]["rules"].as_array().unwrap();
    let rule_ids: Vec<_> = rules.iter().map(|r| r["id"].as_str().unwrap()).collect();
    let mut sorted = rule_ids.clone();
    sorted.sort();
    assert_eq!(rule_ids, sorted, "rules must be sorted by detector ID");

    let results = run["results"].as_array().unwrap();
    for result in results {
        let index = result["ruleIndex"].as_u64().unwrap() as usize;
        assert_eq!(rules[index]["id"], result["ruleId"]);
    }
    assert_eq!(results[0]["ruleId"], "tx-origin");
    assert_eq!(results[0]["locations"][0]["physicalLocation"]["region"]["startLine"], 3);
    assert_eq!(results[1]["ruleId"], "floating-pragma");
    assert!(
        results[1]["locations"][0]["physicalLocation"]
            .get("region")
            .is_none()
    );
}

#[test]
fn test_json_findings_use_detector_metadata() {
    let output = JsonFormatter::new(false).format(&report_with_findings());
    let json: Value = serde_json::from_str(&output).unwrap();
    let finding = &json["findings"][0];
    assert_eq!(finding["id"], "tx-origin");
    assert!(finding["swc_ids"].is_array());
    assert!(finding["cwe_ids"].is_array());

    let mut registry = DetectorRegistry::new();
    register_all_detectors(&mut registry);
    let confidence = registry.get("tx-origin").unwrap().meta().confidence;
    assert_eq!(finding["confidence"], confidence.to_string().to_lowercase());
}
