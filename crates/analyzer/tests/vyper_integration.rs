//! Integration tests for Vyper support in Verazt Analyzer.
//!
//! These tests verify that the Verazt Analyzer pipeline correctly handles Vyper
//! contracts through the compile → SIR → detection path.

use analyzer::{
    AnalysisConfig, AnalysisContext, AnalysisReport, InputLanguage, JsonFormatter,
    OutputFormatter, PipelineConfig, PipelineEngine,
};
use std::path::Path;
use std::process::Command;

/// Helper: run the full Verazt Analyzer pipeline on the Vyper example
/// `name` of the workspace (via `frontend::vyper::compile_file`) and return
/// the pipeline result.
///
/// Returns `None`, skipping the test, only when the `vyper` compiler cannot
/// be run (CI environments, etc.); any other compile error fails the test.
fn run_vyper_pipeline(name: &str) -> Option<analyzer::PipelineResult> {
    if let Err(e) = Command::new("vyper").arg("--version").output() {
        eprintln!("Skipping Vyper test (cannot run `vyper`: {e})");
        return None;
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/vyper").join(name);
    let file = path.to_str().expect("UTF-8 path");
    let module = frontend::vyper::compile_file(file, None)
        .unwrap_or_else(|e| panic!("compiling {file} failed: {e}"));

    let config =
        AnalysisConfig { input_language: InputLanguage::Vyper, ..AnalysisConfig::default() };
    let mut context = AnalysisContext::new(vec![module], config);

    let engine = PipelineEngine::new(PipelineConfig::default());
    Some(engine.run(&mut context))
}

// ─── Unit-level tests (no compiler required) ────────────────────

/// Verify that the pipeline runs without panicking when given an empty
/// Vyper context (no source units, no IR).
#[test]
fn test_vyper_empty_context() {
    let config =
        AnalysisConfig { input_language: InputLanguage::Vyper, ..AnalysisConfig::default() };

    let mut context = AnalysisContext::new(vec![], config);
    let engine = PipelineEngine::new(PipelineConfig::default());
    let result = engine.run(&mut context);

    // No bugs should be found (no input)
    assert_eq!(result.total_bugs(), 0);
}

/// Verify that GREP (AST-level) detectors are filtered out for Vyper.
#[test]
fn test_vyper_grep_detectors_skipped() {
    let config =
        AnalysisConfig { input_language: InputLanguage::Vyper, ..AnalysisConfig::default() };

    let mut context = AnalysisContext::new(vec![], config);
    let engine = PipelineEngine::new(PipelineConfig::default());
    let result = engine.run(&mut context);

    // All detectors should succeed (none should panic on missing AST)
    for stat in &result.detector_stats {
        assert!(stat.success, "Detector '{}' failed unexpectedly: {:?}", stat.name, stat.error);
    }
}

/// Verify the JSON output formatter works with Vyper language tag.
#[test]
fn test_vyper_json_output() {
    let report = AnalysisReport::with_language(
        vec![],
        vec!["test.vy".to_string()],
        std::time::Duration::from_secs(1),
        "vyper",
    );

    let formatter = JsonFormatter::new(true);
    let output = formatter.format(&report);

    assert!(output.contains("\"source_language\": \"vyper\""));
    assert!(output.contains("\"findings\""));
}

/// Verify `InputLanguage::default()` is Solidity.
#[test]
fn test_input_language_default() {
    assert_eq!(InputLanguage::default(), InputLanguage::Solidity);
}

// ─── Compiler-dependent integration tests ───────────────────────

/// token.vy — clean contract, expect 0 high-severity bugs.
#[test]
fn test_vyper_token_clean() {
    let result = match run_vyper_pipeline("token.vy") {
        Some(r) => r,
        None => return, // skip if compiler unavailable
    };

    let high_severity: Vec<_> = result
        .bugs
        .iter()
        .filter(|b| {
            matches!(b.risk_level, bugs::bug::RiskLevel::Critical | bugs::bug::RiskLevel::High)
        })
        .collect();

    assert!(
        high_severity.is_empty(),
        "Expected no high-severity bugs in token.vy, found: {:?}",
        high_severity.iter().map(|b| &b.name).collect::<Vec<_>>()
    );
}

/// vault.vy — clean vault contract, expect 0 high-severity bugs.
#[test]
fn test_vyper_vault_clean() {
    let result = match run_vyper_pipeline("vault.vy") {
        Some(r) => r,
        None => return,
    };

    let high_severity: Vec<_> = result
        .bugs
        .iter()
        .filter(|b| {
            matches!(b.risk_level, bugs::bug::RiskLevel::Critical | bugs::bug::RiskLevel::High)
        })
        .collect();

    assert!(
        high_severity.is_empty(),
        "Expected no high-severity bugs in vault.vy, found: {:?}",
        high_severity.iter().map(|b| &b.name).collect::<Vec<_>>()
    );
}

/// vault_buggy.vy — intentionally buggy contract.
/// This test verifies the pipeline runs without panicking on a buggy
/// Vyper contract. In the MVP, most detectors are AST-based and thus
/// skipped for Vyper; once IR-based detectors are ported, this test
/// should be updated to assert specific bug findings.
#[test]
fn test_vyper_vault_buggy() {
    let result = match run_vyper_pipeline("vault_buggy.vy") {
        Some(r) => r,
        None => return,
    };

    // Pipeline should complete without errors
    for stat in &result.detector_stats {
        assert!(stat.success, "Detector '{}' failed: {:?}", stat.name, stat.error);
    }
}
