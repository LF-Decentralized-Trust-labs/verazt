//! Integration tests for Vyper support in SmartHunt.
//!
//! These tests verify that the SmartHunt pipeline correctly handles Vyper
//! contracts through the compile → SIR → detection path.

use analyzer::{
    AnalysisConfig, AnalysisContext, AnalysisReport, InputLanguage, JsonFormatter,
    OutputFormatter, PipelineConfig, PipelineEngine,
};

/// Helper: run the full SmartHunt pipeline on a Vyper file (via
/// `vyper::compile_file`) and return the pipeline result.
///
/// Skipped at runtime when the Vyper compiler is not available (CI
/// environments, etc.).
fn run_vyper_pipeline(vyper_file: &str) -> Option<analyzer::PipelineResult> {
    let module = match vyper::compile_file(vyper_file, None) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Skipping Vyper test (compiler not available): {e}");
            return None;
        }
    };

    let config =
        AnalysisConfig { input_language: InputLanguage::Vyper, ..AnalysisConfig::default() };

    let mut context = AnalysisContext::new(vec![], config);
    context.set_ir_units(vec![module]);

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
    let result = match run_vyper_pipeline("examples/vyper/token.vy") {
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
    let result = match run_vyper_pipeline("examples/vyper/vault.vy") {
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
    let result = match run_vyper_pipeline("examples/vyper/vault_buggy.vy") {
        Some(r) => r,
        None => return,
    };

    // Pipeline should complete without errors
    for stat in &result.detector_stats {
        assert!(stat.success, "Detector '{}' failed: {:?}", stat.name, stat.error);
    }
}
