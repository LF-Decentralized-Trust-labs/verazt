//! `verazt scan` — quick security scan with plain-text or JSON output
//!
//! Runs the same detector pipeline as `verazt analyze`, without its report
//! formatting and configuration file.

use crate::config::InputLanguage;
use crate::context::{AnalysisConfig, AnalysisContext};
use crate::detectors::{DetectorRegistry, register_all_detectors};
use crate::pipeline::{PipelineConfig, PipelineEngine};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Run a quick security scan")]
pub struct Args {
    /// Input smart contract files
    pub input_files: Vec<String>,

    /// Input language override (solidity, vyper)
    #[arg(long)]
    pub language: Option<String>,

    // Solidity options
    #[arg(long)]
    pub base_path: Option<String>,
    #[arg(long)]
    pub include_path: Vec<String>,
    #[arg(long)]
    pub solc_version: Option<String>,

    /// Output format: text, json
    #[arg(long, short, default_value = "text")]
    pub format: String,

    /// List of detector IDs to enable (comma-separated)
    #[arg(long)]
    pub enable: Option<String>,

    /// List of detector IDs to disable (comma-separated)
    #[arg(long)]
    pub disable: Option<String>,

    /// Enable parallel execution
    #[arg(long)]
    pub parallel: bool,

    /// List available detectors
    #[arg(long)]
    pub list_detectors: bool,
}

pub fn run<I, T>(args_iter: I)
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let args = Args::parse_from(args_iter);

    if args.list_detectors {
        print_detectors();
        return;
    }

    if args.input_files.is_empty() {
        eprintln!("Error: no input files specified");
        std::process::exit(1);
    }

    // Parse and lower to SIR
    let mut all_modules = Vec::new();
    for input_file in &args.input_files {
        match parse_and_lower(input_file, &args) {
            Ok(modules) => all_modules.extend(modules),
            Err(e) => {
                eprintln!("Error processing '{}': {}", input_file, e);
                std::process::exit(1);
            }
        }
    }

    let input_language = if is_vyper(&args, &args.input_files[0]) {
        InputLanguage::Vyper
    } else {
        InputLanguage::Solidity
    };
    let analysis_config = AnalysisConfig { input_language };
    let mut context = AnalysisContext::new(all_modules, analysis_config);

    let engine = PipelineEngine::new(PipelineConfig {
        parallel: args.parallel,
        enabled: split_ids(args.enable.as_deref()),
        disabled: split_ids(args.disable.as_deref()),
        ..PipelineConfig::default()
    });
    let result = engine.run(&mut context);
    let detectors_run = result.detector_stats.len();

    // Output
    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&result.bugs).unwrap_or_default();
            println!("{}", json);
        }
        _ => {
            if result.bugs.is_empty() {
                println!(
                    "No issues found ({} detectors run in {:.2?}).",
                    detectors_run, result.total_duration
                );
            } else {
                println!(
                    "Found {} issue(s) ({} detectors run in {:.2?}):\n",
                    result.bugs.len(),
                    detectors_run,
                    result.total_duration
                );
                for bug in &result.bugs {
                    println!("{}", bug.format_with_snippet());
                }
            }
        }
    }

    // The bugs printed above are incomplete if any phase failed
    let failures = result.failures();
    for failure in &failures {
        eprintln!("Error: {failure}");
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
}

/// Split a comma-separated detector list (`--enable` / `--disable`).
fn split_ids(list: Option<&str>) -> Vec<String> {
    list.map(|s| s.split(',').map(|id| id.trim().to_string()).collect())
        .unwrap_or_default()
}

fn is_vyper(args: &Args, input_file: &str) -> bool {
    args.language.as_deref() == Some("vyper") || input_file.ends_with(".vy")
}

fn parse_and_lower(
    input_file: &str,
    args: &Args,
) -> Result<Vec<scirs::sir::Module>, String> {
    if is_vyper(args, input_file) {
        // Vyper path
        let vyper_ver = args.solc_version.as_deref();
        let module = frontend::vyper::compile_file(input_file, vyper_ver)
            .map_err(|e| format!("Vyper compile error: {}", e))?;
        Ok(vec![module])
    } else {
        // Solidity path
        let base_path = args.base_path.as_deref();
        let source_units = frontend::solidity::parsing::parse_input_file(
            input_file,
            base_path,
            &args.include_path,
            args.solc_version.as_deref(),
        )
        .map_err(|e| format!("Parse error: {}", e))?;
        let modules = frontend::solidity::lowering::lower_source_units(&source_units)
            .map_err(|e| format!("Lowering error: {}", e))?;
        Ok(modules)
    }
}

fn print_detectors() {
    let mut registry = DetectorRegistry::new();
    register_all_detectors(&mut registry);
    let mut detectors: Vec<_> = registry.all().collect();
    detectors.sort_by_key(|d| d.meta().id.as_str());

    println!("Detectors ({}):", detectors.len());
    println!("=====================================\n");
    for d in detectors {
        let meta = d.meta();
        println!(
            "  {:<25} {:<30} {:?}   {:?}",
            meta.id.as_str(),
            meta.name,
            meta.bug_kind,
            meta.risk_level
        );
    }
}
