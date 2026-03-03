//! Report generation for benchmark results.

use crate::matcher::MatchResult;
use bugs::bug::BugCategory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-category accuracy report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryReport {
    pub category: String,
    pub true_positives: usize,
    pub false_negatives: usize,
    pub false_positives: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
}

/// Per-file report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReport {
    pub file_path: String,
    pub true_positives: usize,
    pub false_negatives: usize,
    pub false_positives: usize,
}

/// Full benchmark report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub total_files: usize,
    pub files_with_errors: usize,
    pub total_annotations: usize,
    pub true_positives: usize,
    pub false_negatives: usize,
    pub false_positives: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub per_category: HashMap<String, CategoryReport>,
    pub per_file: Vec<FileReport>,
}

/// Generate a benchmark report from match results.
pub fn generate_report(results: &[MatchResult], files_with_errors: usize) -> BenchmarkReport {
    let mut total_tp = 0usize;
    let mut total_fn = 0usize;
    let mut total_fp = 0usize;
    let mut total_annotations = 0usize;

    let mut category_tp: HashMap<BugCategory, usize> = HashMap::new();
    let mut category_fn: HashMap<BugCategory, usize> = HashMap::new();
    let mut category_fp: HashMap<BugCategory, usize> = HashMap::new();

    let mut per_file = Vec::new();

    for result in results {
        let tp = result.true_positives.len();
        let fn_ = result.false_negatives.len();
        let fp = result.false_positives.len();

        total_tp += tp;
        total_fn += fn_;
        total_fp += fp;
        total_annotations += tp + fn_;

        // Per-category counts
        for matched in &result.true_positives {
            *category_tp.entry(matched.annotation.category).or_insert(0) += 1;
        }
        for missed in &result.false_negatives {
            *category_fn.entry(missed.category).or_insert(0) += 1;
        }
        for extra in &result.false_positives {
            *category_fp.entry(extra.category).or_insert(0) += 1;
        }

        per_file.push(FileReport {
            file_path: result.file_path.display().to_string(),
            true_positives: tp,
            false_negatives: fn_,
            false_positives: fp,
        });
    }

    // Compute overall metrics
    let precision = safe_div(total_tp as f64, (total_tp + total_fp) as f64);
    let recall = safe_div(total_tp as f64, (total_tp + total_fn) as f64);
    let f1_score = safe_f1(precision, recall);

    // Build per-category reports
    let mut per_category = HashMap::new();
    let all_categories: Vec<BugCategory> = {
        let mut cats: std::collections::HashSet<BugCategory> = std::collections::HashSet::new();
        for key in category_tp.keys() {
            cats.insert(*key);
        }
        for key in category_fn.keys() {
            cats.insert(*key);
        }
        for key in category_fp.keys() {
            cats.insert(*key);
        }
        cats.into_iter().collect()
    };

    for cat in all_categories {
        let tp = *category_tp.get(&cat).unwrap_or(&0);
        let fn_ = *category_fn.get(&cat).unwrap_or(&0);
        let fp = *category_fp.get(&cat).unwrap_or(&0);
        let p = safe_div(tp as f64, (tp + fp) as f64);
        let r = safe_div(tp as f64, (tp + fn_) as f64);
        let f1 = safe_f1(p, r);

        per_category.insert(
            cat.as_str().to_string(),
            CategoryReport {
                category: cat.as_str().to_string(),
                true_positives: tp,
                false_negatives: fn_,
                false_positives: fp,
                precision: p,
                recall: r,
                f1_score: f1,
            },
        );
    }

    BenchmarkReport {
        total_files: results.len(),
        files_with_errors,
        total_annotations,
        true_positives: total_tp,
        false_negatives: total_fn,
        false_positives: total_fp,
        precision,
        recall,
        f1_score,
        per_category,
        per_file,
    }
}

/// Print a human-readable benchmark report.
pub fn print_report(report: &BenchmarkReport, tool_desc: &str, dataset_path: &str) {
    println!("SmartBench Accuracy Report");
    println!("==========================\n");
    println!("Tool: {}", tool_desc);
    println!("Dataset: {}", dataset_path);
    println!(
        "Files analyzed: {}  |  Files with errors: {}\n",
        report.total_files, report.files_with_errors
    );

    println!(
        "Overall:  TP={}  FN={}  FP={}  Precision={:.1}%  Recall={:.1}%  F1={:.1}%\n",
        report.true_positives,
        report.false_negatives,
        report.false_positives,
        report.precision * 100.0,
        report.recall * 100.0,
        report.f1_score * 100.0,
    );

    if !report.per_category.is_empty() {
        println!("Per Category:");
        let mut cats: Vec<_> = report.per_category.iter().collect();
        cats.sort_by_key(|(name, _)| name.to_string());

        for (name, cat) in cats {
            println!(
                "  {:<30} TP={:<4} FN={:<4} FP={:<4} Recall={:.1}%",
                format!("{}:", name),
                cat.true_positives,
                cat.false_negatives,
                cat.false_positives,
                cat.recall * 100.0,
            );
        }
    }
}

/// Format report as JSON string.
pub fn report_to_json(report: &BenchmarkReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

fn safe_div(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

fn safe_f1(precision: f64, recall: f64) -> f64 {
    if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::AnnotatedBug;
    use crate::matcher::{MatchResult, MatchedBug};
    use crate::runner::DetectedBug;
    use std::path::PathBuf;

    fn make_match_result(tp: usize, fn_: usize, fp: usize) -> MatchResult {
        let true_positives: Vec<MatchedBug> = (0..tp)
            .map(|_| MatchedBug {
                annotation: AnnotatedBug {
                    file_path: PathBuf::from("test.sol"),
                    line_number: 10,
                    bug_line: 11,
                    bug_name: "REENTRANCY".to_string(),
                    category: BugCategory::Reentrancy,
                },
                detection: DetectedBug {
                    name: "Reentrancy".to_string(),
                    category: BugCategory::Reentrancy,
                    start_line: 11,
                    severity: "Critical".to_string(),
                },
            })
            .collect();

        let false_negatives: Vec<AnnotatedBug> = (0..fn_)
            .map(|_| AnnotatedBug {
                file_path: PathBuf::from("test.sol"),
                line_number: 20,
                bug_line: 21,
                bug_name: "REENTRANCY".to_string(),
                category: BugCategory::Reentrancy,
            })
            .collect();

        let false_positives: Vec<DetectedBug> = (0..fp)
            .map(|_| DetectedBug {
                name: "False alarm".to_string(),
                category: BugCategory::AccessControl,
                start_line: 30,
                severity: "Medium".to_string(),
            })
            .collect();

        MatchResult {
            file_path: PathBuf::from("test.sol"),
            true_positives,
            false_negatives,
            false_positives,
        }
    }

    #[test]
    fn test_generate_report_perfect() {
        let results = vec![make_match_result(5, 0, 0)];
        let report = generate_report(&results, 0);

        assert_eq!(report.true_positives, 5);
        assert_eq!(report.false_negatives, 0);
        assert_eq!(report.false_positives, 0);
        assert!((report.precision - 1.0).abs() < f64::EPSILON);
        assert!((report.recall - 1.0).abs() < f64::EPSILON);
        assert!((report.f1_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generate_report_mixed() {
        let results = vec![make_match_result(3, 2, 1)];
        let report = generate_report(&results, 0);

        assert_eq!(report.true_positives, 3);
        assert_eq!(report.false_negatives, 2);
        assert_eq!(report.false_positives, 1);
        assert_eq!(report.total_annotations, 5);

        // Precision = 3/(3+1) = 0.75
        assert!((report.precision - 0.75).abs() < 0.01);
        // Recall = 3/(3+2) = 0.6
        assert!((report.recall - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_generate_report_empty() {
        let results: Vec<MatchResult> = vec![];
        let report = generate_report(&results, 0);

        assert_eq!(report.total_files, 0);
        assert_eq!(report.true_positives, 0);
        assert!((report.precision - 0.0).abs() < f64::EPSILON);
        assert!((report.recall - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generate_report_per_category() {
        let results = vec![make_match_result(2, 1, 1)];
        let report = generate_report(&results, 0);

        assert!(!report.per_category.is_empty());
        if let Some(reentrancy) = report.per_category.get("Reentrancy") {
            assert_eq!(reentrancy.true_positives, 2);
            assert_eq!(reentrancy.false_negatives, 1);
        }
    }

    #[test]
    fn test_report_to_json() {
        let results = vec![make_match_result(1, 0, 0)];
        let report = generate_report(&results, 0);
        let json = report_to_json(&report);
        assert!(json.contains("\"true_positives\""));
        assert!(json.contains("\"precision\""));
    }

    #[test]
    fn test_safe_div_zero() {
        assert_eq!(safe_div(1.0, 0.0), 0.0);
        assert_eq!(safe_div(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_safe_f1_zero() {
        assert_eq!(safe_f1(0.0, 0.0), 0.0);
    }
}
