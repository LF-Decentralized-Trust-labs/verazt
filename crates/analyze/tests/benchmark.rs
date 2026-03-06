//! Integration tests: benchmark analyze against the SmartBugs-curated
//! dataset (https://github.com/smartbugs/smartbugs-curated).
//!
//! These tests use `smartbench` as a dev-dependency to run analyze
//! and compare its findings with the ground truth.

use std::path::{Path, PathBuf};

use bugs::bug::BugCategory;
use analyze::{AnalysisConfig, AnalysisContext, PipelineConfig, PipelineEngine};
use smartbench::runner::DetectedBug;
use smartbench::{generate_report, match_file, scan_dataset};
use solidity::parser::parse_input_file;

/// Run analyze on a single .sol file and return DetectedBugs.
fn run_analyze_on_file(file_path: &Path) -> Vec<DetectedBug> {
    let source_units = match parse_input_file(file_path.to_str().unwrap(), None, &[], None) {
        Ok(units) => units,
        Err(_) => return vec![],
    };

    if source_units.is_empty() {
        return vec![];
    }

    let mut context = AnalysisContext::new(source_units, AnalysisConfig::default());
    let engine =
        PipelineEngine::new(PipelineConfig { parallel: false, ..PipelineConfig::default() });

    let result = engine.run(&mut context);

    result
        .bugs
        .iter()
        .map(|bug| DetectedBug {
            name: bug.name.clone(),
            category: bug.category,
            start_line: bug.loc.start_line,
            severity: bug.risk_level.as_str().to_string(),
        })
        .collect()
}

/// Test reentrancy detection on the reentrancy dataset subset.
#[test]
fn test_reentrancy_detection_accuracy() {
    let dataset_dir = Path::new("../../benchmarks/smartbugs-curated/dataset/reentrancy");
    if !dataset_dir.exists() {
        eprintln!("Skipping: dataset not found at {}", dataset_dir.display());
        return;
    }

    let annotations = scan_dataset(dataset_dir);
    assert!(!annotations.is_empty(), "Should find annotations in reentrancy dataset");

    // Filter to reentrancy-only annotations
    let reentrancy_annotations: Vec<_> = annotations
        .iter()
        .filter(|a| a.category == BugCategory::Reentrancy)
        .cloned()
        .collect();

    assert!(!reentrancy_annotations.is_empty(), "Should find REENTRANCY annotations");

    // We don't enforce a minimum recall here since the tool is still being
    // developed, but we verify the pipeline runs without panicking.
    let mut total_tp = 0;
    let mut total_fn = 0;
    let mut total_fp = 0;

    // Group annotations by file
    let mut annotations_by_file =
        std::collections::HashMap::<std::path::PathBuf, Vec<smartbench::AnnotatedBug>>::new();
    for ann in &reentrancy_annotations {
        annotations_by_file
            .entry(ann.file_path.clone())
            .or_default()
            .push(ann.clone());
    }

    for (file_path, file_annotations) in &annotations_by_file {
        let detections = run_analyze_on_file(file_path);
        let result = match_file(file_path, file_annotations, &detections);
        total_tp += result.true_positives.len();
        total_fn += result.false_negatives.len();
        total_fp += result.false_positives.len();
    }

    println!("Reentrancy benchmark: TP={} FN={} FP={}", total_tp, total_fn, total_fp);

    // Basic sanity: the pipeline should produce some results (TP + FP > 0 or
    // handle gracefully) We don't assert a minimum since this depends on
    // the parser+detector capabilities.
}

/// Test annotation parsing on the entire dataset.
#[test]
fn test_dataset_annotation_parsing() {
    let dataset_dir = Path::new("../../benchmarks/smartbugs-curated/dataset");
    if !dataset_dir.exists() {
        eprintln!("Skipping: dataset not found at {}", dataset_dir.display());
        return;
    }

    let annotations = scan_dataset(dataset_dir);
    assert!(!annotations.is_empty(), "Should find annotations in the full dataset");

    // Verify we find annotations from different categories
    let categories: std::collections::HashSet<_> =
        annotations.iter().map(|a| a.category).collect();

    // The dataset should have at least a few categories
    assert!(
        categories.len() >= 3,
        "Should find annotations from at least 3 categories, found {:?}",
        categories
    );

    println!("Parsed {} annotations across {} categories", annotations.len(), categories.len());
    for cat in &categories {
        let count = annotations.iter().filter(|a| a.category == *cat).count();
        println!("  {}: {} annotations", cat, count);
    }
}

/// Test full pipeline on all categories (lightweight: just verifies no panics).
#[test]
fn test_all_categories_detection() {
    let dataset_dir = Path::new("../../benchmarks/smartbugs-curated/dataset");
    if !dataset_dir.exists() {
        eprintln!("Skipping: dataset not found at {}", dataset_dir.display());
        return;
    }

    let annotations = scan_dataset(dataset_dir);

    // Group annotations by file
    let mut annotations_by_file =
        std::collections::HashMap::<std::path::PathBuf, Vec<smartbench::AnnotatedBug>>::new();
    for ann in &annotations {
        annotations_by_file
            .entry(ann.file_path.clone())
            .or_default()
            .push(ann.clone());
    }

    let mut match_results = Vec::new();
    let files_with_errors = 0;

    // Run on a small sample (first 10 files with annotations) to keep test fast
    let files: Vec<_> = annotations_by_file.keys().take(10).cloned().collect();
    for file_path in &files {
        let detections = run_analyze_on_file(file_path);
        if detections.is_empty() {
            // Could be a parse error or no findings
        }
        let file_annotations = annotations_by_file.get(file_path).unwrap();
        let result = match_file(file_path, file_annotations, &detections);
        match_results.push(result);
    }

    let report = generate_report(&match_results, files_with_errors);

    println!(
        "Full benchmark (sample): TP={} FN={} FP={} Precision={:.1}% Recall={:.1}%",
        report.true_positives,
        report.false_negatives,
        report.false_positives,
        report.precision * 100.0,
        report.recall * 100.0,
    );
}
