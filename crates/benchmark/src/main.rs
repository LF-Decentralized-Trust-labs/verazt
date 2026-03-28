//! Benchmark CLI: evaluate Verazt analyzer against SmartBugs-curated dataset.
//!
//! Usage:
//!   cargo run -p benchmark -- --dataset solidity/smartbugs-curated
//!   cargo run -p benchmark -- --dataset solidity/smartbugs-curated --category
//! reentrancy   cargo run -p benchmark -- --dataset solidity/smartbugs-curated
//! --verbose

mod evaluate;
mod report;

use std::path::PathBuf;

use bugs::datasets::smartbugs::collect_sol_files;
use clap::Parser;

#[derive(Parser)]
#[command(name = "benchmark", about = "Evaluate Verazt analyzer on bug datasets")]
struct Args {
    /// Path to dataset relative to the datasets/ folder
    /// (e.g. "solidity/smartbugs-curated")
    #[arg(long)]
    dataset: String,

    /// Filter to a specific category subfolder
    #[arg(long)]
    category: Option<String>,

    /// Show per-file details
    #[arg(long)]
    verbose: bool,

    /// Solc version override
    #[arg(long, default_value = "0.4.26")]
    solc_version: String,
}

/// Resolve the workspace root by searching upward for the workspace Cargo.toml.
fn find_workspace_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("Failed to get current directory");
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();
            if content.contains("[workspace]") {
                return dir;
            }
        }
        if !dir.pop() {
            eprintln!("Error: could not find workspace root");
            std::process::exit(1);
        }
    }
}

fn main() {
    let args = Args::parse();

    // Resolve dataset path
    let workspace_root = find_workspace_root();
    let dataset_path = workspace_root.join("datasets").join(&args.dataset);

    if !dataset_path.exists() {
        eprintln!("Error: dataset not found at {}", dataset_path.display());
        std::process::exit(1);
    }

    // Determine scan directory (optionally filtered by category)
    let scan_dir: PathBuf = match &args.category {
        Some(category) => {
            let cat_dir = dataset_path.join(category);
            if !cat_dir.exists() {
                eprintln!("Error: category '{}' not found at {}", category, cat_dir.display());
                std::process::exit(1);
            }
            cat_dir
        }
        None => dataset_path.clone(),
    };

    // Collect .sol files
    let mut sol_files = collect_sol_files(&scan_dir);
    sol_files.sort();

    if sol_files.is_empty() {
        eprintln!("No .sol files found in {}", scan_dir.display());
        std::process::exit(1);
    }

    println!("Found {} .sol files in {}", sol_files.len(), scan_dir.display());

    // Run evaluation
    let result = evaluate::evaluate_dataset(&sol_files, &args.solc_version);

    // Print report
    report::print_report(&result, args.verbose);
}
