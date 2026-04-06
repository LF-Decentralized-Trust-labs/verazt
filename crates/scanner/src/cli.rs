//! `verazt scan` — fast syntactic security checks

use crate::registry::{ScanRegistry, register_all_detectors};
use crate::engine::{ScanConfig, ScanEngine};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Run fast syntactic security scan checks")]
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

    /// List available scan detectors
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

    // Detect language from extension
    let _language = args.language.as_deref().unwrap_or_else(|| {
        if let Some(first) = args.input_files.first() {
            if first.ends_with(".vy") {
                "vyper"
            } else {
                "solidity"
            }
        } else {
            "solidity"
        }
    });

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

    // Create registry and filter
    let mut registry = ScanRegistry::new();
    register_all_detectors(&mut registry);

    let mut detectors = registry.into_detectors();

    // Filter by enable/disable
    if let Some(ref enable_str) = args.enable {
        let enabled: Vec<&str> = enable_str.split(',').map(|s| s.trim()).collect();
        detectors.retain(|d| enabled.contains(&d.id()));
    }
    if let Some(ref disable_str) = args.disable {
        let disabled: Vec<&str> = disable_str.split(',').map(|s| s.trim()).collect();
        detectors.retain(|d| !disabled.contains(&d.id()));
    }

    let engine = ScanEngine::new(ScanConfig::default(), detectors);
    let report = engine.run(&all_modules);

    // Output
    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&report.bugs).unwrap_or_default();
            println!("{}", json);
        }
        _ => {
            if report.bugs.is_empty() {
                println!("No issues found ({} detectors run in {:.2?}).",
                    report.detectors_run, report.duration);
            } else {
                println!(
                    "Found {} issue(s) ({} detectors run in {:.2?}):\n",
                    report.bugs.len(),
                    report.detectors_run,
                    report.duration
                );
                for bug in &report.bugs {
                    println!("{}", bug.format_with_snippet());
                }
            }
        }
    }
}

fn parse_and_lower(
    input_file: &str,
    args: &Args,
) -> Result<Vec<scirs::sir::Module>, String> {
    let is_vyper = args.language.as_deref() == Some("vyper")
        || input_file.ends_with(".vy");

    if is_vyper {
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
    let mut registry = ScanRegistry::new();
    register_all_detectors(&mut registry);

    println!("Scan Detectors — EVM ({}):", registry.len());
    println!("=====================================\n");

    // Group by level
    let module_dets: Vec<_> = registry.all().iter()
        .filter(|d| d.level() == crate::DetectionLevel::Module)
        .collect();
    let contract_dets: Vec<_> = registry.all().iter()
        .filter(|d| d.level() == crate::DetectionLevel::Contract)
        .collect();
    let function_dets: Vec<_> = registry.all().iter()
        .filter(|d| d.level() == crate::DetectionLevel::Function)
        .collect();

    println!("MODULE ({}):", module_dets.len());
    for d in &module_dets {
        println!("  {:<25} {:<30} {:?}   {:?}",
            d.id(), d.name(), d.bug_kind(), d.risk_level());
    }

    println!("\nCONTRACT ({}):", contract_dets.len());
    for d in &contract_dets {
        println!("  {:<25} {:<30} {:?}   {:?}",
            d.id(), d.name(), d.bug_kind(), d.risk_level());
    }

    println!("\nFUNCTION ({}):", function_dets.len());
    for d in &function_dets {
        println!("  {:<25} {:<30} {:?}   {:?}",
            d.id(), d.name(), d.bug_kind(), d.risk_level());
    }
}
