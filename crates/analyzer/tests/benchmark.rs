//! Integration tests: benchmark analyze against the SmartBugs-curated
//! dataset (https://github.com/smartbugs/smartbugs-curated).
//!
//! These tests use annotations from the `bugs` crate to run analyze
//! and compare its findings with the ground truth.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use analyzer::{AnalysisConfig, AnalysisContext, PipelineConfig, PipelineEngine};
use bugs::bug::BugCategory;
use bugs::datasets::smartbugs::{AnnotatedBug, scan_dataset};
use frontend::solidity::parser::parse_input_file;

/// Represents a bug detected by the local analyzer.
#[derive(Debug, Clone)]
struct DetectedBug {
    pub name: String,
    pub category: BugCategory,
    pub start_line: usize,
    pub severity: String,
}

/// Represents the matching outcome for a single file.
#[derive(Debug, Default)]
struct MatchResult {
    pub true_positives: Vec<MatchedBug>,
    pub false_positives: Vec<DetectedBug>,
    pub false_negatives: Vec<AnnotatedBug>,
}

#[derive(Debug)]
struct MatchedBug {
    #[allow(dead_code)]
    pub annotation: AnnotatedBug,
    #[allow(dead_code)]
    pub detection: DetectedBug,
}

/// Matches detected bugs against ground truth annotations.
///
/// A detection is considered a True Positive if it matches the exact line and
/// category. In SmartBugs, the annotation is placed one line above the bug.
fn match_file(
    _file_path: &Path,
    annotations: &[AnnotatedBug],
    detections: &[DetectedBug],
) -> MatchResult {
    let mut result = MatchResult::default();
    let mut matched_detections = HashSet::new();
    let mut matched_annotations = HashSet::new();

    // Find true positives
    for (ann_idx, ann) in annotations.iter().enumerate() {
        for (det_idx, det) in detections.iter().enumerate() {
            if matched_detections.contains(&det_idx) {
                continue;
            }

            // Check if categories match and line numbers are close (exact or contiguous)
            // Note: `bug_line` is the vulnerable code line (annotation line + 1).
            let line_match = det.start_line == ann.bug_line;
            let category_match = det.category == ann.category;

            if line_match && category_match {
                result
                    .true_positives
                    .push(MatchedBug { annotation: ann.clone(), detection: det.clone() });
                matched_detections.insert(det_idx);
                matched_annotations.insert(ann_idx);
                break; // One annotation matched by one detection
            }
        }
    }

    // Unmatched annotations are false negatives
    for (ann_idx, ann) in annotations.iter().enumerate() {
        if !matched_annotations.contains(&ann_idx) {
            result.false_negatives.push(ann.clone());
        }
    }

    // Unmatched detections are false positives
    for (det_idx, det) in detections.iter().enumerate() {
        if !matched_detections.contains(&det_idx) {
            result.false_positives.push(det.clone());
        }
    }

    result
}

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

    let mut total_tp = 0;
    let mut total_fn = 0;
    let mut total_fp = 0;

    // Group annotations by file
    let mut annotations_by_file = HashMap::<PathBuf, Vec<AnnotatedBug>>::new();
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
    let categories: HashSet<_> = annotations.iter().map(|a| a.category).collect();

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
    let mut annotations_by_file = HashMap::<PathBuf, Vec<AnnotatedBug>>::new();
    for ann in &annotations {
        annotations_by_file
            .entry(ann.file_path.clone())
            .or_default()
            .push(ann.clone());
    }

    let mut total_tp = 0;
    let mut total_fn = 0;
    let mut total_fp = 0;

    // Run on a small sample (first 10 files with annotations) to keep test fast
    let files: Vec<_> = annotations_by_file.keys().take(10).cloned().collect();
    for file_path in &files {
        let detections = run_analyze_on_file(file_path);
        let file_annotations = annotations_by_file.get(file_path).unwrap();
        let result = match_file(file_path, file_annotations, &detections);

        total_tp += result.true_positives.len();
        total_fn += result.false_negatives.len();
        total_fp += result.false_positives.len();
    }

    let precision = if total_tp + total_fp > 0 {
        total_tp as f64 / (total_tp + total_fp) as f64
    } else {
        0.0
    };
    let recall = if total_tp + total_fn > 0 {
        total_tp as f64 / (total_tp + total_fn) as f64
    } else {
        0.0
    };

    println!(
        "Full benchmark (sample): TP={} FN={} FP={} Precision={:.1}% Recall={:.1}%",
        total_tp,
        total_fn,
        total_fp,
        precision * 100.0,
        recall * 100.0,
    );
}
