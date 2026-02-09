//! SmartHunt - AST-based Smart Contract Bug Detection CLI
//!
//! This is the main entry point for the SmartHunt tool.

use clap::{Parser, Subcommand, crate_version};
use extlib::error;
use smarthunt::{
    AnalysisConfig,
    AnalysisContext,
    AnalysisReport,
    Config,
    DetectorRegistry,
    JsonFormatter,
    MarkdownFormatter,
    OutputFormat,
    OutputFormatter,
    PipelineConfig,
    PipelineEngine,
    SarifFormatter,
    SeverityFilter,
    register_all_detectors,
};
use solidity::{
    ast::SourceUnit, ast::utils::export::export_debugging_source_unit, parser::parse_input_file,
};
use std::fs;

#[derive(Parser, Debug)]
#[command(
    author,
    version = crate_version!(),
    term_width = 80,
    about = "SmartHunt - AST-based Smart Contract Bug Detection",
    long_about = None
)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Input Solidity files to be compiled.
    pub input_files: Vec<String>,

    /// The root directory of the source tree, if specified.
    #[arg(long, default_value = None)]
    pub base_path: Option<String>,

    /// Additional directory to look for import files.
    #[arg(long, default_value = None)]
    pub include_path: Vec<String>,

    /// Print debugging information.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    /// Configure Solidity compiler version.
    #[arg(long, default_value = None)]
    pub solc_version: Option<String>,

    /// Print input program.
    #[arg(long, visible_alias = "pip", default_value_t = false)]
    pub print_input_program: bool,

    /// Output format: json, markdown, sarif, text
    #[arg(long, short, default_value = "text")]
    pub format: String,

    /// Output file (default: stdout)
    #[arg(long, short)]
    pub output: Option<String>,

    /// Configuration file path
    #[arg(long, short)]
    pub config: Option<String>,

    /// List of detector IDs to enable (comma-separated)
    #[arg(long)]
    pub enable: Option<String>,

    /// List of detector IDs to disable (comma-separated)
    #[arg(long)]
    pub disable: Option<String>,

    /// Minimum severity to report: info, low, medium, high, critical
    #[arg(long, default_value = "info")]
    pub min_severity: String,

    /// Enable parallel analysis
    #[arg(long, default_value_t = false)]
    pub parallel: bool,

    /// Verbosity
    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::ErrorLevel>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Analyze smart contracts for vulnerabilities
    Analyze {
        /// Input files to analyze
        files: Vec<String>,
    },
    /// List available detectors
    ListDetectors,
    /// Show detector information
    ShowDetector {
        /// Detector ID
        id: String,
    },
    /// Generate a default configuration file
    InitConfig {
        /// Output file
        #[arg(default_value = "smarthunt.toml")]
        output: String,
    },
}

/// Main function
fn main() {
    env_logger::init();
    error::config();

    // Parse command line arguments
    let mut args = Arguments::parse();

    // Handle subcommands
    if let Some(command) = args.command.clone() {
        match command {
            Command::ListDetectors => {
                list_detectors();
                return;
            }
            Command::ShowDetector { id } => {
                show_detector(&id);
                return;
            }
            Command::InitConfig { output } => {
                init_config(&output);
                return;
            }
            Command::Analyze { files } => {
                args.input_files = files;
                run_analysis(args);
                return;
            }
        }
    }

    // Default: run analysis on input files
    if !args.input_files.is_empty() {
        run_analysis(args);
    } else {
        eprintln!("No input files specified. Use --help for usage information.");
        std::process::exit(1);
    }
}

fn list_detectors() {
    let mut registry = DetectorRegistry::new();
    register_all_detectors(&mut registry);
    println!("Available Detectors ({}):", registry.len());
    println!("========================\n");

    let detectors = registry.all().collect::<Vec<_>>();
    let mut sorted_detectors = detectors.clone();
    sorted_detectors.sort_by(|a, b| a.name().cmp(&b.name()));

    println!("{:<25} {:<35} {:<10} {:<10}", "ID", "Name", "Severity", "Confidence");
    println!("{}", "-".repeat(85));

    for detector in sorted_detectors {
        println!(
            "{:<25} {:<35} {:<10} {:<10}",
            detector.id().as_str(),
            detector.name(),
            detector.risk_level().as_str(),
            format!("{:?}", detector.confidence()).to_lowercase(),
        );
    }

    println!("\nUse 'smarthunt show-detector <id>' for detailed information.");
}

fn show_detector(id: &str) {
    let mut registry = DetectorRegistry::new();
    register_all_detectors(&mut registry);

    match registry.get(id) {
        Some(detector) => {
            println!("Detector: {}", detector.name());
            println!("ID: {}", detector.id().as_str());
            println!("Severity: {}", detector.risk_level());
            println!("Confidence: {:?}", detector.confidence());
            println!();
            println!("Description:");
            println!("  {}", detector.description());
            println!();
            println!("Recommendation:");
            println!("  {}", detector.recommendation());
            println!();

            let swc_ids = detector.swc_ids();
            if !swc_ids.is_empty() {
                println!(
                    "SWC IDs: {}",
                    swc_ids
                        .iter()
                        .map(|id| format!("SWC-{}", id))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            let cwe_ids = detector.cwe_ids();
            if !cwe_ids.is_empty() {
                println!(
                    "CWE IDs: {}",
                    cwe_ids
                        .iter()
                        .map(|id| format!("CWE-{}", id))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            let refs = detector.references();
            if !refs.is_empty() {
                println!();
                println!("References:");
                for r in refs {
                    println!("  - {}", r);
                }
            }
        }
        None => {
            eprintln!("Detector '{}' not found.", id);
            eprintln!("Use 'smarthunt list-detectors' to see available detectors.");
            std::process::exit(1);
        }
    }
}

fn init_config(output: &str) {
    let default_config = r#"# SmartHunt Configuration File

[analysis]
# Enable parallel analysis
parallel = true
# Maximum number of worker threads (0 = auto-detect)
max_workers = 0

[detectors]
# Enable vulnerability detection
vulnerabilities = true
# Enable refactoring suggestions
refactoring = true
# Enable optimization hints
optimization = true

# Explicitly enable specific detectors (empty = all enabled)
# enabled = ["reentrancy", "tx-origin"]

# Explicitly disable specific detectors
# disabled = []

[output]
# Output format: "text", "json", "markdown", "sarif"
format = "text"
# Minimum severity to report: "info", "low", "medium", "high", "critical"
min_severity = "info"

[ignore]
# Patterns to ignore in files
patterns = [
    "// smarthunt-disable",
    "// slither-disable",
]

# Files to ignore
files = [
    "test/**",
    "node_modules/**",
]

# Directories to ignore
directories = [
    "lib",
    "node_modules",
]
"#;

    match fs::write(output, default_config) {
        Ok(_) => {
            println!("Configuration file created: {}", output);
        }
        Err(e) => {
            eprintln!("Failed to create configuration file: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_analysis(args: Arguments) {
    // Load configuration
    let mut config = if let Some(config_path) = &args.config {
        Config::from_file(std::path::Path::new(config_path)).unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        })
    } else {
        Config::default()
    };

    // Apply CLI overrides
    if args.parallel {
        config.num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
    }

    if let Some(enable) = &args.enable {
        config.detectors.enabled = enable.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Some(disable) = &args.disable {
        config.detectors.disabled = disable.split(',').map(|s| s.trim().to_string()).collect();
    }

    config.output_format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        "markdown" | "md" => OutputFormat::Markdown,
        "sarif" => OutputFormat::Sarif,
        _ => OutputFormat::Text,
    };

    config.min_severity = match args.min_severity.as_str() {
        "critical" => SeverityFilter::Critical,
        "high" => SeverityFilter::High,
        "medium" => SeverityFilter::Medium,
        "low" => SeverityFilter::Low,
        _ => SeverityFilter::Informational,
    };

    // Parse input files
    let solc_ver = args.solc_version.as_deref();
    let base_path = args.base_path.as_deref();
    let include_paths: &[String] = &args.include_path;

    let mut all_source_units: Vec<SourceUnit> = Vec::new();
    let mut files_analyzed: Vec<String> = Vec::new();

    for file in &args.input_files {
        if args.debug {
            eprintln!("Compiling: {}", file);
        }

        let source_units = match parse_input_file(file, base_path, include_paths, solc_ver) {
            Ok(source_units) => source_units,
            Err(err) => {
                eprintln!("Error compiling {}: {}", file, err);
                continue;
            }
        };

        if args.print_input_program {
            println!("Source units after parsing:");
        }

        for source_unit in &source_units {
            if args.print_input_program {
                source_unit.print_highlighted_code();
                println!();
            }
            if args.debug {
                if let Err(err) = export_debugging_source_unit(source_unit, "parsed") {
                    eprintln!("Warning: {}", err);
                }
            }
        }

        files_analyzed.push(file.clone());
        all_source_units.extend(source_units);
    }

    if all_source_units.is_empty() {
        eprintln!("No source files were successfully compiled.");
        std::process::exit(1);
    }

    // Create analysis context
    let mut context = AnalysisContext::new(all_source_units, AnalysisConfig::default());

    // Create and run the pipeline
    let engine = PipelineEngine::new(PipelineConfig {
        parallel: config.num_threads > 1,
        num_threads: config.num_threads,
        enabled: config.detectors.enabled.clone(),
        disabled: config.detectors.disabled.clone(),
    });

    if args.debug {
        eprintln!(
            "Running pipeline ({} threads)...",
            if config.num_threads > 1 { config.num_threads } else { 1 }
        );
    }

    let result = engine.run(&mut context);

    // Create report
    let report = AnalysisReport::new(result.bugs, files_analyzed, result.total_duration);

    // Format output
    let output = match config.output_format {
        OutputFormat::Json => {
            let formatter = JsonFormatter::new(true);
            formatter.format(&report)
        }
        OutputFormat::Markdown => {
            let formatter = MarkdownFormatter::new();
            formatter.format(&report)
        }
        OutputFormat::Sarif => {
            let formatter = SarifFormatter::new(true);
            formatter.format(&report)
        }
        OutputFormat::Text => format_text_output(&report),
    };

    // Write output
    match &args.output {
        Some(path) => {
            if let Err(e) = fs::write(path, &output) {
                eprintln!("Failed to write output: {}", e);
                std::process::exit(1);
            }
            eprintln!("Report written to: {}", path);
        }
        None => {
            println!("{}", output);
        }
    }

    // Exit with error code if high severity issues found
    if report.has_high_severity() {
        std::process::exit(1);
    }
}

fn format_text_output(report: &AnalysisReport) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "SmartHunt Analysis Report\n\
         =========================\n\n\
         Files analyzed: {}\n\
         Duration: {:.2}s\n\n",
        report.files_analyzed.len(),
        report.duration.as_secs_f64()
    ));

    output.push_str(&format!(
        "Summary:\n\
         - Critical: {}\n\
         - High: {}\n\
         - Medium: {}\n\
         - Low: {}\n\
         - Info: {}\n\
         - Total: {}\n\n",
        report.stats.bugs_by_severity.critical,
        report.stats.bugs_by_severity.high,
        report.stats.bugs_by_severity.medium,
        report.stats.bugs_by_severity.low,
        report.stats.bugs_by_severity.info,
        report.total_bugs(),
    ));

    if report.bugs.is_empty() {
        output.push_str("✅ No issues found!\n");
    } else {
        output.push_str("Findings:\n");
        output.push_str("---------\n\n");

        for (i, bug) in report.bugs.iter().enumerate() {
            output.push_str(&format!("{}. [{}] {}\n", i + 1, bug.risk_level, bug.name));

            output
                .push_str(&format!("   Location: {}:{}\n", bug.loc.start_line, bug.loc.start_col));

            if let Some(desc) = &bug.description {
                output.push_str(&format!("   {}\n", desc));
            }

            output.push('\n');
        }
    }

    output
}
