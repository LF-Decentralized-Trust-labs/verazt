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
            "- {} ({} files, {} expected)  TP: {}  FP: {}  FN: {}",
            cat, stats.file_count, stats.expected, stats.tp, stats.fp, stats.r#fn,
        );
    }

    // Overall
    print_subheader("Overall");

    println!("  Total Expected:  {}", result.total_expected);
    println!("  True Positives:  {}", result.total_tp);
    println!("  False Positives: {}", result.total_fp);
    println!("  False Negatives: {}", result.total_fn);

    println!("  False Negatives: {}", result.total_fn);
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
