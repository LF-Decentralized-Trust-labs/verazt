//! Report formatting for benchmark evaluation results.

use common::utils::{print_header, print_subheader};

use crate::evaluate::{DatasetResult, FileResult};

/// Print the full evaluation report.
pub fn print_report(result: &DatasetResult, verbose: bool) {
    print_header("SmartBugs-Curated Evaluation Report");

    println!(
        "Files: {} total, {} compiled, {} skipped",
        result.total_files, result.compiled_files, result.skipped_files,
    );

    // Verbose: per-file details
    if verbose {
        print_subheader("Per-File Details");
        for file_result in &result.file_results {
            print_file_details(file_result);
        }
    }

    // Per-category results
    print_subheader("Per-Category Results");

    // Sort categories for deterministic output
    let mut categories: Vec<_> = result.per_category.keys().copied().collect();
    categories.sort_by_key(|c| c.to_annotation());

    for cat in &categories {
        let stats = &result.per_category[cat];
        println!(
            "- {} ({} files, {} expected)  TP: {}  FP: {}  FP*: {}  FN: {}  \
             Recall: {}  Precision: {}  Precision*: {}",
            cat,
            stats.file_count,
            stats.expected,
            stats.tp,
            stats.fp,
            stats.fp_annotated,
            stats.r#fn,
            percent(stats.tp, stats.expected),
            percent(stats.tp, stats.tp + stats.fp),
            percent(stats.tp, stats.tp + stats.fp_annotated),
        );
    }

    // Per-detector results
    print_subheader("Per-Detector Results");

    println!(
        "{:<26} {:>5} {:>5} {:>5} {:>10} {:>11}",
        "Detector", "TP", "FP", "FP*", "Precision", "Precision*"
    );
    for (id, stats) in &result.per_detector {
        println!(
            "{:<26} {:>5} {:>5} {:>5} {:>10} {:>11}",
            id,
            stats.tp,
            stats.fp,
            stats.fp_annotated,
            percent(stats.tp, stats.tp + stats.fp),
            percent(stats.tp, stats.tp + stats.fp_annotated),
        );
    }

    // Overall
    print_subheader("Overall");

    println!("  Total Expected:  {}", result.total_expected);
    println!("  True Positives:  {}", result.total_tp);
    println!("  False Positives: {} ({} FP*)", result.total_fp, result.total_fp_annotated);
    println!("  False Negatives: {}", result.total_fn);
    println!("  Recall:          {}", percent(result.total_tp, result.total_expected));
    println!(
        "  Precision:       {} ({} Precision*)",
        percent(result.total_tp, result.total_tp + result.total_fp),
        percent(result.total_tp, result.total_tp + result.total_fp_annotated),
    );

    println!();
    println!(
        "FP* counts only the false positives in files annotated with the finding's \
         category. Datasets annotate only the bugs of a file's main category, so \
         Precision is a lower bound and Precision* a closer estimate."
    );
}

/// `part / whole` as a percentage, or `-` when `whole` is zero.
fn percent(part: usize, whole: usize) -> String {
    if whole == 0 {
        "-".to_string()
    } else {
        format!("{:.1}%", 100.0 * part as f64 / whole as f64)
    }
}

/// Print details for a single file.
fn print_file_details(result: &FileResult) {
    println!("--- {}", result.file_path.display());

    if !result.compiled {
        println!("    [SKIPPED] Compilation failed");
        println!();
        return;
    }

    // Expected bugs
    if result.annotations.is_empty() {
        println!("    Expected: (none)");
    } else {
        for ann in &result.annotations {
            println!("    Expected: {} @ line {}", ann.category, ann.bug_line,);
        }
    }

    // Detected bugs
    if result.detections.is_empty() {
        println!("    Detected: (none)");
    } else {
        for det in &result.detections {
            println!("    Detected: {}({}) @ line {}", det.name, det.category, det.start_line,);
        }
    }

    // Match summary
    println!(
        "    TP: {}  FP: {}  FN: {}",
        result.match_result.true_positives.len(),
        result.match_result.false_positives.len(),
        result.match_result.false_negatives.len(),
    );
    println!();
}
