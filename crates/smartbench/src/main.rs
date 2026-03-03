//! SmartBench CLI - Benchmark analysis tools against the SmartBugs dataset.

use clap::{Args, Parser};
use smartbench::runner::{ToolConfig, create_runner};
use smartbench::{generate_report, match_file, print_report, report, run_batch, scan_dataset};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "SmartBench - Benchmark smart contract analysis tools",
    long_about = "Benchmark analysis tools against the SmartBugs-curated dataset.\n\
                  Parses ground-truth annotations, runs the tool, and reports accuracy."
)]
struct Cli {
    #[command(flatten)]
    tool: ToolArgs,

    /// Path to dataset directory.
    #[arg(long)]
    dataset: PathBuf,

    /// Output format: text or json.
    #[arg(long, default_value = "text")]
    format: String,

    /// Output file (default: stdout).
    #[arg(long, short)]
    output: Option<PathBuf>,

    /// Filter to a specific bug category.
    #[arg(long)]
    category: Option<String>,

    /// Per-file timeout in seconds (Docker mode).
    #[arg(long)]
    timeout: Option<u64>,

    /// Memory limit per container, e.g., "4g" (Docker mode).
    #[arg(long)]
    mem_limit: Option<String>,

    /// CPU limit per container (Docker mode).
    #[arg(long)]
    cpus: Option<f64>,

    /// Show per-file details.
    #[arg(long)]
    verbose: bool,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct ToolArgs {
    /// Path to local analysis tool binary.
    #[arg(long)]
    tool_path: Option<PathBuf>,

    /// Docker image for the analysis tool.
    #[arg(long)]
    docker_image: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Build ToolConfig
    let tool_config = if let Some(ref path) = cli.tool.tool_path {
        ToolConfig::Binary { path: path.clone() }
    } else if let Some(ref image) = cli.tool.docker_image {
        ToolConfig::Docker {
            image: image.clone(),
            timeout_secs: cli.timeout,
            memory_limit: cli.mem_limit.clone(),
            cpus: cli.cpus,
        }
    } else {
        eprintln!("Error: must provide either --tool-path or --docker-image");
        std::process::exit(1);
    };

    let tool_desc = match &tool_config {
        ToolConfig::Binary { path } => format!("binary: {}", path.display()),
        ToolConfig::Docker { image, .. } => format!("docker: {}", image),
    };

    // Scan dataset for annotations
    let annotations = scan_dataset(&cli.dataset);
    if annotations.is_empty() {
        eprintln!("Warning: no annotations found in {}", cli.dataset.display());
    }

    // Optionally filter by category
    let annotations = if let Some(ref cat_name) = cli.category {
        let cat = bugs::bug::BugCategory::from_annotation(cat_name).unwrap_or_else(|| {
            eprintln!("Unknown category: {}", cat_name);
            std::process::exit(1);
        });
        annotations
            .into_iter()
            .filter(|a| a.category == cat)
            .collect()
    } else {
        annotations
    };

    // Create runner and run on all .sol files
    let runner = create_runner(&tool_config);
    let tool_results = run_batch(runner.as_ref(), &cli.dataset);

    // Count errors
    let files_with_errors = tool_results.iter().filter(|r| !r.success).count();

    // Group annotations by file
    let mut annotations_by_file: HashMap<PathBuf, Vec<_>> = HashMap::new();
    for ann in &annotations {
        annotations_by_file
            .entry(ann.file_path.clone())
            .or_default()
            .push(ann.clone());
    }

    // Match ground truth vs detections
    let mut match_results = Vec::new();
    for tool_result in &tool_results {
        let file_annotations = annotations_by_file
            .get(&tool_result.file_path)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let result = match_file(&tool_result.file_path, file_annotations, &tool_result.findings);
        match_results.push(result);
    }

    // Generate report
    let bench_report = generate_report(&match_results, files_with_errors);

    // Output
    match cli.format.as_str() {
        "json" => {
            let json = report::report_to_json(&bench_report);
            output_string(&json, cli.output.as_deref());
        }
        _ => {
            // Capture stdout for text output
            print_report(&bench_report, &tool_desc, &cli.dataset.display().to_string());

            if cli.verbose {
                println!("\nPer-file Details:");
                println!("{}", "-".repeat(80));
                for file in &bench_report.per_file {
                    if file.true_positives > 0
                        || file.false_negatives > 0
                        || file.false_positives > 0
                    {
                        println!(
                            "  {}: TP={} FN={} FP={}",
                            file.file_path,
                            file.true_positives,
                            file.false_negatives,
                            file.false_positives,
                        );
                    }
                }
            }
        }
    }
}

fn output_string(content: &str, output_path: Option<&std::path::Path>) {
    match output_path {
        Some(path) => {
            if let Err(e) = std::fs::write(path, content) {
                eprintln!("Failed to write output: {}", e);
                std::process::exit(1);
            }
            eprintln!("Report written to: {}", path.display());
        }
        None => {
            println!("{}", content);
        }
    }
}
