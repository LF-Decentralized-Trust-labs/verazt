//! Evaluation logic: compile, analyze, and match detected bugs against
//! ground-truth annotations.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use analyzer::{AnalysisConfig, AnalysisContext, PipelineConfig, PipelineEngine};
use bugs::bug::BugCategory;
use bugs::datasets::smartbugs::{AnnotatedBug, parse_annotations};

/// Represents a bug detected by the analyzer.
#[derive(Debug, Clone)]
pub struct DetectedBug {
    pub name: String,
    pub category: BugCategory,
    pub start_line: usize,
    #[allow(dead_code)]
    pub severity: String,
}

/// Represents a matched true positive.
#[derive(Debug)]
pub struct MatchedBug {
    pub annotation: AnnotatedBug,
    #[allow(dead_code)]
    pub detection: DetectedBug,
}

/// Matching outcome for a single file.
#[derive(Debug, Default)]
pub struct MatchResult {
    pub true_positives: Vec<MatchedBug>,
    pub false_positives: Vec<DetectedBug>,
    pub false_negatives: Vec<AnnotatedBug>,
}

/// Result of evaluating a single file.
pub struct FileResult {
    pub file_path: PathBuf,
    pub compiled: bool,
    pub annotations: Vec<AnnotatedBug>,
    pub detections: Vec<DetectedBug>,
    pub match_result: MatchResult,
}

/// Aggregated statistics for a category.
#[derive(Debug, Default)]
pub struct CategoryStats {
    pub file_count: usize,
    pub expected: usize,
    pub tp: usize,
    pub fp: usize,
    pub r#fn: usize,
}

/// Aggregated statistics for the entire dataset.
pub struct DatasetResult {
    pub total_files: usize,
    pub compiled_files: usize,
    pub skipped_files: usize,
    pub per_category: HashMap<BugCategory, CategoryStats>,
    pub total_expected: usize,
    pub total_tp: usize,
    pub total_fp: usize,
    pub total_fn: usize,
    pub file_results: Vec<FileResult>,
}

/// Default line tolerance for fuzzy matching.
const LINE_TOLERANCE: usize = 3;

/// Match detected bugs against ground-truth annotations.
///
/// Primary match: exact category + exact `bug_line`.
/// Fallback match: exact category + line within ±LINE_TOLERANCE.
pub fn match_file(
    _file_path: &Path,
    annotations: &[AnnotatedBug],
    detections: &[DetectedBug],
) -> MatchResult {
    let mut result = MatchResult::default();
    let mut matched_detections = HashSet::new();
    let mut matched_annotations = HashSet::new();

    // Pass 1: exact line match
    for (ann_idx, ann) in annotations.iter().enumerate() {
        for (det_idx, det) in detections.iter().enumerate() {
            if matched_detections.contains(&det_idx) {
                continue;
            }

            let line_match = det.start_line == ann.bug_line;
            let category_match = det.category == ann.category;

            if line_match && category_match {
                result
                    .true_positives
                    .push(MatchedBug { annotation: ann.clone(), detection: det.clone() });
                matched_detections.insert(det_idx);
                matched_annotations.insert(ann_idx);
                break;
            }
        }
    }

    // Pass 2: fuzzy line match (±LINE_TOLERANCE) for unmatched annotations
    for (ann_idx, ann) in annotations.iter().enumerate() {
        if matched_annotations.contains(&ann_idx) {
            continue;
        }
        for (det_idx, det) in detections.iter().enumerate() {
            if matched_detections.contains(&det_idx) {
                continue;
            }

            let category_match = det.category == ann.category;
            let line_diff = (det.start_line as isize - ann.bug_line as isize).unsigned_abs();
            let fuzzy_match = line_diff <= LINE_TOLERANCE;

            if category_match && fuzzy_match {
                result
                    .true_positives
                    .push(MatchedBug { annotation: ann.clone(), detection: det.clone() });
                matched_detections.insert(det_idx);
                matched_annotations.insert(ann_idx);
                break;
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

/// Compile and analyze a single `.sol` file, returning detected bugs.
///
/// Returns an empty vec and logs a warning if compilation fails.
pub fn run_analyze_on_file(file_path: &Path, solc_version: &str) -> (bool, Vec<DetectedBug>) {
    let file_str = match file_path.to_str() {
        Some(s) => s,
        None => {
            eprintln!("Warning: invalid file path: {}", file_path.display());
            return (false, vec![]);
        }
    };

    // Step 1: Compile .sol file to AST
    let source_units = match frontend::solidity::parsing::parse_input_file(
        file_str,
        None,
        &[],
        Some(solc_version),
    ) {
        Ok(units) => units,
        Err(err) => {
            eprintln!("Warning: compilation failed for {}: {}", file_path.display(), err);
            return (false, vec![]);
        }
    };

    // Step 2: Lower AST → SIR
    let sir_modules = match frontend::solidity::lowering::lower_source_units(&source_units) {
        Ok(modules) => modules,
        Err(err) => {
            eprintln!("Warning: SIR lowering failed for {}: {}", file_path.display(), err);
            return (false, vec![]);
        }
    };

    // Step 3: Build analysis context
    let config = AnalysisConfig::default();
    let mut context = AnalysisContext::new(sir_modules, config);

    // Step 4: Run pipeline
    let engine =
        PipelineEngine::new(PipelineConfig { parallel: false, ..PipelineConfig::default() });

    let result = engine.run(&mut context);

    // Step 5: Convert Bug → DetectedBug
    let detected = result
        .bugs
        .iter()
        .map(|bug| DetectedBug {
            name: bug.name.clone(),
            category: bug.category,
            start_line: bug.loc.start_line,
            severity: bug.risk_level.as_str().to_string(),
        })
        .collect();

    (true, detected)
}

/// Evaluate a single file: parse annotations, run analyzer, match results.
pub fn evaluate_file(file_path: &Path, solc_version: &str) -> FileResult {
    let annotations = parse_annotations(file_path);
    let (compiled, detections) = run_analyze_on_file(file_path, solc_version);
    let match_result = match_file(file_path, &annotations, &detections);

    FileResult {
        file_path: file_path.to_path_buf(),
        compiled,
        annotations,
        detections,
        match_result,
    }
}

/// Evaluate the entire dataset, returning aggregated results.
pub fn evaluate_dataset(sol_files: &[PathBuf], solc_version: &str) -> DatasetResult {
    let mut file_results = Vec::new();
    let mut compiled_files = 0usize;
    let mut skipped_files = 0usize;
    let mut per_category: HashMap<BugCategory, CategoryStats> = HashMap::new();
    let mut total_tp = 0usize;
    let mut total_fp = 0usize;
    let mut total_fn = 0usize;
    let mut total_expected = 0usize;

    for file_path in sol_files {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            common::utils::print_header(&format!("Test Case: {}", file_name));
        } else {
            common::utils::print_header("Test Case");
        }
        let result = evaluate_file(file_path, solc_version);

        if result.compiled {
            compiled_files += 1;
        } else {
            skipped_files += 1;
        }

        // Aggregate per-category stats
        // Track which categories appear in this file's annotations
        let mut file_categories: HashSet<BugCategory> = HashSet::new();
        for ann in &result.annotations {
            file_categories.insert(ann.category);
        }

        for cat in &file_categories {
            let stats = per_category.entry(*cat).or_default();
            stats.file_count += 1;
        }

        for ann in &result.annotations {
            let stats = per_category.entry(ann.category).or_default();
            stats.expected += 1;
            total_expected += 1;
        }

        let tp = result.match_result.true_positives.len();
        let fp = result.match_result.false_positives.len();
        let fn_count = result.match_result.false_negatives.len();

        total_tp += tp;
        total_fp += fp;
        total_fn += fn_count;

        // Attribute TP/FP/FN to categories
        for matched in &result.match_result.true_positives {
            let stats = per_category.entry(matched.annotation.category).or_default();
            stats.tp += 1;
        }
        for det in &result.match_result.false_positives {
            let stats = per_category.entry(det.category).or_default();
            stats.fp += 1;
        }
        for ann in &result.match_result.false_negatives {
            let stats = per_category.entry(ann.category).or_default();
            stats.r#fn += 1;
        }

        file_results.push(result);
    }

    DatasetResult {
        total_files: sol_files.len(),
        compiled_files,
        skipped_files,
        per_category,
        total_expected,
        total_tp,
        total_fp,
        total_fn,
        file_results,
    }
}
