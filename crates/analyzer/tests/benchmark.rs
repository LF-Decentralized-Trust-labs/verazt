//! Integration tests: benchmark analyze against the SmartBugs-curated
//! dataset (https://github.com/smartbugs/smartbugs-curated).
//!
//! These tests use annotations from the `bugs` crate to run analyze
//! and compare its findings with the ground truth.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use analyzer::{AnalysisConfig, AnalysisContext, PipelineConfig, PipelineEngine};
use bugs::bug::{Bug, BugCategory};
use bugs::datasets::smartbugs::{AnnotatedBug, scan_dataset};

/// Minimum recall on the reentrancy subset. Measured at 31/32 (96.9%).
const REENTRANCY_RECALL_FLOOR: f64 = 0.95;

/// Minimum recall on the whole dataset, over the files that compile.
/// Measured at 153/204 (75.0%).
const OVERALL_RECALL_FLOOR: f64 = 0.74;

/// Matching outcome counts.
#[derive(Debug, Default)]
struct MatchCounts {
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
}

impl MatchCounts {
    fn add(&mut self, other: MatchCounts) {
        self.true_positives += other.true_positives;
        self.false_positives += other.false_positives;
        self.false_negatives += other.false_negatives;
    }

    fn recall(&self) -> f64 {
        let expected = self.true_positives + self.false_negatives;
        if expected == 0 { 0.0 } else { self.true_positives as f64 / expected as f64 }
    }
}

/// The SmartBugs-curated dataset, or one of its category subfolders.
fn dataset_dir(category: Option<&str>) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../datasets/solidity/smartbugs-curated");
    category.map_or(root.clone(), |c| root.join(c))
}

/// Whether a Solidity compiler is installed, as the dataset tests need one.
fn solc_available() -> bool {
    Command::new("solc").arg("--version").output().is_ok_and(|o| o.status.success())
}

/// Matches detected bugs against ground truth annotations.
///
/// A detection is a true positive if it has the annotated category and
/// starts on the annotated bug line (the line after the annotation).
fn match_file(annotations: &[AnnotatedBug], detections: &[Bug]) -> MatchCounts {
    let mut matched_detections = HashSet::new();
    for ann in annotations {
        let matched = detections.iter().enumerate().find(|(idx, det)| {
            !matched_detections.contains(idx)
                && det.loc.start_line == ann.bug_line
                && det.category == ann.category
        });
        if let Some((idx, _)) = matched {
            matched_detections.insert(idx);
        }
    }
    let true_positives = matched_detections.len();
    MatchCounts {
        true_positives,
        false_positives: detections.len() - true_positives,
        false_negatives: annotations.len() - true_positives,
    }
}

/// Compile and lower `file_path` as `verazt analyze` does (solc version
/// picked from the pragma), then run all detectors on it. Fails if the file
/// does not compile; panics if the analysis itself fails.
fn run_analyze_on_file(file_path: &Path) -> Result<Vec<Bug>, String> {
    let file = file_path.to_str().ok_or("non UTF-8 path")?;
    let source_units = frontend::solidity::parsing::parse_input_file(file, None, &[], None)
        .map_err(|e| format!("parse error: {e}"))?;
    let modules = frontend::solidity::lowering::lower_source_units(&source_units)
        .map_err(|e| format!("lowering error: {e}"))?;
    let mut context = AnalysisContext::new(modules, AnalysisConfig::default());
    let engine =
        PipelineEngine::new(PipelineConfig { parallel: false, ..PipelineConfig::default() });
    let result = engine.run(&mut context);
    assert!(result.failures().is_empty(), "{}: {:?}", file_path.display(), result.failures());
    Ok(result.bugs)
}

/// Analyze every annotated file and match its findings with `annotations`.
/// Returns the match counts and the files that failed to compile, whose
/// annotations are left out of the counts.
fn benchmark(annotations: &[AnnotatedBug]) -> (MatchCounts, Vec<String>) {
    let mut by_file = BTreeMap::<PathBuf, Vec<AnnotatedBug>>::new();
    for ann in annotations {
        by_file.entry(ann.file_path.clone()).or_default().push(ann.clone());
    }

    let mut counts = MatchCounts::default();
    let mut uncompiled = Vec::new();
    for (file_path, file_annotations) in &by_file {
        match run_analyze_on_file(file_path) {
            Ok(detections) => counts.add(match_file(file_annotations, &detections)),
            Err(e) => uncompiled.push(format!("{}: {e}", file_path.display())),
        }
    }
    (counts, uncompiled)
}

/// Reentrancy recall on the reentrancy dataset subset.
#[test]
fn test_reentrancy_detection_accuracy() {
    if !solc_available() {
        eprintln!("Skipping: solc is not installed");
        return;
    }

    let annotations: Vec<_> = scan_dataset(&dataset_dir(Some("reentrancy")))
        .into_iter()
        .filter(|a| a.category == BugCategory::Reentrancy)
        .collect();
    assert!(!annotations.is_empty(), "Should find REENTRANCY annotations");

    let (counts, uncompiled) = benchmark(&annotations);
    assert!(uncompiled.is_empty(), "files failed to compile: {uncompiled:#?}");
    println!("Reentrancy benchmark: {counts:?} recall={:.3}", counts.recall());
    assert!(
        counts.recall() >= REENTRANCY_RECALL_FLOOR,
        "reentrancy recall {:.3} fell below {REENTRANCY_RECALL_FLOOR}: {counts:?}",
        counts.recall()
    );
}

/// Test annotation parsing on the entire dataset.
#[test]
fn test_dataset_annotation_parsing() {
    let annotations = scan_dataset(&dataset_dir(None));
    assert!(!annotations.is_empty(), "Should find annotations in the full dataset");

    let categories: HashSet<_> = annotations.iter().map(|a| a.category).collect();
    assert!(
        categories.len() >= 3,
        "Should find annotations from at least 3 categories, found {:?}",
        categories
    );
}

/// Recall over all categories of the dataset. Slow: run with `--ignored`.
#[test]
#[ignore]
fn test_all_categories_detection() {
    if !solc_available() {
        eprintln!("Skipping: solc is not installed");
        return;
    }

    let (counts, uncompiled) = benchmark(&scan_dataset(&dataset_dir(None)));
    println!("Full benchmark: {counts:?} recall={:.3}", counts.recall());
    println!("Skipped {} files failing to compile: {uncompiled:#?}", uncompiled.len());
    assert!(
        counts.recall() >= OVERALL_RECALL_FLOOR,
        "overall recall {:.3} fell below {OVERALL_RECALL_FLOOR}: {counts:?}",
        counts.recall()
    );
}
