//! Analyzer CLI: the analysis options and the detector and configuration
//! commands exposed by the `verazt` binary.

use crate::config::{DEFAULT_CONFIG_TOML, available_threads};
use crate::{
    AnalysisConfig, AnalysisContext, AnalysisReport, Config, DetectorRegistry, InputLanguage,
    JsonFormatter, MarkdownFormatter, OutputFormat, OutputFormatter, PipelineConfig,
    PipelineEngine, SarifFormatter, SeverityFilter, register_all_detectors,
};
use common::error;
use frontend::solidity::{
    ast::SourceUnit, ast::utils::export::export_debugging_source_unit, parsing::parse_input_file,
};
use std::env::consts::EXE_SUFFIX;
use std::ffi::OsStr;
use std::fs;

/// Default output path of `verazt init-config`.
pub const DEFAULT_CONFIG_FILE: &str = "verazt.toml";

/// Options of an analysis run.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Input smart contract files (.sol or .vy).
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

    /// Input language: solidity, vyper.
    /// Auto-detected from file extension if not specified.
    #[arg(long, default_value = None)]
    pub language: Option<String>,

    /// Configure Vyper compiler version (e.g. "^0.3.9").
    #[arg(long, default_value = None)]
    pub vyper_version: Option<String>,

    /// Print input program.
    #[arg(long, visible_alias = "pip", default_value_t = false)]
    pub print_input_program: bool,

    /// Output format [default: text, or the configuration file's]
    #[arg(long, short, value_enum)]
    pub format: Option<OutputFormat>,

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

    /// Minimum severity to report [default: info, or the configuration
    /// file's]
    #[arg(long, value_enum)]
    pub min_severity: Option<SeverityFilter>,

    /// Automatically install the required compiler version if none is
    /// available. Skips the interactive prompt.
    #[arg(long, default_value_t = false)]
    pub install_compiler: bool,

    /// Enable parallel analysis
    #[arg(long, default_value_t = false)]
    pub parallel: bool,
}

/// Analyze the input files and report the detected bugs.
pub fn run(args: Args) {
    env_logger::try_init().ok();
    error::config();

    if args.input_files.is_empty() {
        eprintln!("No input files specified. Use --help for usage information.");
        std::process::exit(1);
    }
    run_analysis(args);
}

/// Print a table of all registered detectors.
pub fn list_detectors() {
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
        let meta = detector.meta();
        println!(
            "{:<25} {:<35} {:<10} {:<10}",
            meta.id.as_str(),
            meta.name,
            meta.risk_level.as_str(),
            format!("{:?}", meta.confidence).to_lowercase(),
        );
    }

    println!("\nUse 'verazt show-detector <id>' for detailed information.");
}

/// Print the metadata of the detector with the given ID.
pub fn show_detector(id: &str) {
    let mut registry = DetectorRegistry::new();
    register_all_detectors(&mut registry);

    match registry.get(id) {
        Some(detector) => {
            let meta = detector.meta();
            println!("Detector: {}", meta.name);
            println!("ID: {}", meta.id.as_str());
            println!("Severity: {}", meta.risk_level);
            println!("Confidence: {:?}", meta.confidence);
            println!();
            println!("Description:");
            println!("  {}", meta.description);
            println!();
            println!("Recommendation:");
            println!("  {}", meta.recommendation);
            println!();

            let swc_ids = meta.swc_ids;
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

            let cwe_ids = meta.cwe_ids;
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

            let refs = meta.references;
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
            eprintln!("Use 'verazt list-detectors' to see available detectors.");
            std::process::exit(1);
        }
    }
}

/// Write the default configuration file to `output`.
pub fn init_config(output: &str) {
    match fs::write(output, DEFAULT_CONFIG_TOML) {
        Ok(_) => {
            println!("Configuration file created: {}", output);
        }
        Err(e) => {
            eprintln!("Failed to create configuration file: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_analysis(args: Args) {
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
        config.num_threads = available_threads();
    }

    if let Some(enable) = &args.enable {
        config.detectors.enabled = enable.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Some(disable) = &args.disable {
        config.detectors.disabled = disable.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Some(format) = args.format {
        config.output_format = format;
    }

    if let Some(min_severity) = args.min_severity {
        config.min_severity = min_severity;
    }

    // Parse input files
    let solc_ver = args.solc_version.as_deref();
    let vyper_ver = args.vyper_version.as_deref();
    let base_path = args.base_path.as_deref();
    let include_paths: &[String] = &args.include_path;

    // Detect input language
    let input_language = detect_language(&args.input_files, args.language.as_deref());

    let mut sir_units: Vec<scirs::sir::Module> = Vec::new();
    let mut files_analyzed: Vec<String> = Vec::new();

    for file in &args.input_files {
        if args.debug {
            let rel_file = common::utils::format_relative_path(std::path::Path::new(file));
            eprintln!("\nCompiling: {}", rel_file);
        }

        match input_language {
            InputLanguage::Solidity => {
                let source_units = match parse_input_file(file, base_path, include_paths, solc_ver)
                {
                    Ok(source_units) => source_units,
                    Err(err) => {
                        // Try auto-install recovery
                        match try_install_and_compile_solidity(
                            file,
                            base_path,
                            include_paths,
                            solc_ver,
                            args.install_compiler,
                        ) {
                            Some(units) => units,
                            None => {
                                eprintln!("Error compiling {}: {}", file, err);
                                continue;
                            }
                        }
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

                // Lower AST to SIR
                match frontend::solidity::lowering::lower_source_units(&source_units) {
                    Ok(modules) => {
                        sir_units.extend(modules);
                    }
                    Err(err) => {
                        eprintln!("Error lowering {}: {}", file, err);
                        continue;
                    }
                }
            }
            InputLanguage::Vyper => match frontend::vyper::compile_file(file, vyper_ver) {
                Ok(module) => {
                    sir_units.push(module);
                }
                Err(err) => {
                    // Try auto-install recovery
                    match try_install_and_compile_vyper(file, vyper_ver, args.install_compiler) {
                        Some(module) => {
                            sir_units.push(module);
                        }
                        None => {
                            eprintln!("Error compiling {}: {}", file, err);
                            continue;
                        }
                    }
                }
            },
            _ => {
                eprintln!(
                    "Language {:?} is not yet supported by the scanner CLI.",
                    input_language
                );
                continue;
            }
        }

        files_analyzed.push(file.clone());
    }

    if files_analyzed.is_empty() {
        eprintln!("No source files were successfully compiled.");
        std::process::exit(1);
    }

    // Create analysis context
    let analysis_config = AnalysisConfig { input_language };
    let mut context = AnalysisContext::new(sir_units, analysis_config);

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
            if config.num_threads > 1 {
                config.num_threads
            } else {
                1
            }
        );
    }

    let mut result = engine.run(&mut context);
    let failures = result.failures();
    result.bugs.retain(|bug| config.should_report_severity(&bug.risk_level));

    // Create report
    let lang_str = match input_language {
        InputLanguage::Vyper => "vyper",
        InputLanguage::Solidity => "solidity",
        InputLanguage::MoveSui => "move_sui",
        InputLanguage::MoveAptos => "move_aptos",
        InputLanguage::Solana => "solana",
    };
    let report = AnalysisReport::with_language(
        result.bugs,
        files_analyzed,
        result.total_duration,
        lang_str,
    );

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

    for failure in &failures {
        eprintln!("Error: {failure}");
    }

    // Exit with error code if the analysis was incomplete or high severity
    // issues were found
    if !failures.is_empty() || report.has_high_severity() {
        std::process::exit(1);
    }
}

fn format_header(title: &str) -> String {
    let ruler = "=".repeat(75);
    format!("\n{}\n*** {} ***\n{}\n\n", ruler, title, ruler)
}

fn format_text_output(report: &AnalysisReport) -> String {
    let mut output = String::new();

    if report.bugs.is_empty() {
        output.push_str(&format_header("Detected Bugs"));
        output.push_str("✅ No issues found!\n");
    } else {
        output.push_str(&format_header("Detected Bugs"));

        for (i, bug) in report.bugs.iter().enumerate() {
            output.push_str(&format!(
                "🐛 Issue {}: {} ({}) [{}]\n\n",
                i + 1,
                bug.name,
                bug.category,
                bug.detector_id
            ));

            let snippet_file = bug.loc.file.as_deref().unwrap_or("");
            let display_file =
                common::utils::format_relative_path(std::path::Path::new(snippet_file));

            let loc_str = if bug.loc.start_col == 0 || bug.loc.end_col == 0 {
                format!("{}:{}", display_file, bug.loc.start_line)
            } else if bug.loc.start_line == bug.loc.end_line {
                format!(
                    "{}:{}:{}-{}",
                    display_file, bug.loc.start_line, bug.loc.start_col, bug.loc.end_col
                )
            } else {
                format!(
                    "{}:{}:{}-{}:{}",
                    display_file,
                    bug.loc.start_line,
                    bug.loc.start_col,
                    bug.loc.end_line,
                    bug.loc.end_col
                )
            };

            output.push_str(&format!("---> {}\n", loc_str));

            if let Some(snippet) = common::snippet::extract_snippet(
                snippet_file,
                bug.loc.start_line,
                bug.loc.end_line,
                bug.loc.start_col,
                bug.loc.end_col,
                1,
            ) {
                output.push_str(&snippet);
            } else {
                output.push_str("<source code line not available>\n");
            }

            output.push('\n');
            let desc = bug.description.as_deref().unwrap_or("None");
            output.push_str(&format!("Description: {}\n\n", desc));
            output.push_str(&format!("Severity: {}\n\n", bug.risk_level));
            if let Some(ref remedy) = bug.remediation {
                output.push_str(&format!("Remediation: {}\n\n", remedy));
            }
            output.push_str(&format!("Location: {}\n\n", loc_str));
        }
    }

    output.push_str(&format_header("Summary"));
    output.push_str(&format!(
        "Files analyzed: {}\n\
         Duration: {:.2}s\n\n\
         - Critical: {}\n\
         - High: {}\n\
         - Medium: {}\n\
         - Low: {}\n\
         - Info: {}\n\
         - Total: {}\n\n",
        report.files_analyzed.len(),
        report.duration.as_secs_f64(),
        report.stats.bugs_by_severity.critical,
        report.stats.bugs_by_severity.high,
        report.stats.bugs_by_severity.medium,
        report.stats.bugs_by_severity.low,
        report.stats.bugs_by_severity.info,
        report.total_bugs(),
    ));

    output
}

/// Detect the input language from CLI override or file extensions.
///
/// All input files must share the same detected language.
fn detect_language(files: &[String], override_lang: Option<&str>) -> InputLanguage {
    if let Some(lang) = override_lang {
        return match lang.to_lowercase().as_str() {
            "vyper" | "vy" => InputLanguage::Vyper,
            _ => InputLanguage::Solidity,
        };
    }

    // Infer from first file extension
    if let Some(first) = files.first() {
        if first.ends_with(".vy") {
            return InputLanguage::Vyper;
        }
    }

    InputLanguage::Solidity
}

// ============================================================================
// Compiler auto-install helpers
// ============================================================================

/// Read a yes/no answer from stderr/stdin. Returns true for "y" or "Y".
fn prompt_yes_no(msg: &str) -> bool {
    use std::io::{self, Write};
    eprint!("{msg}");
    io::stderr().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    matches!(input.trim(), "y" | "Y")
}

/// Returns true if `tool` is found in PATH.
///
/// Looks the executable up instead of running `tool --version`, which
/// `solc-select` rejects with a non-zero exit status.
fn is_tool_installed(tool: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| is_in_path(tool, &path))
}

/// Returns true if an executable named `tool` exists in a directory of the
/// PATH-formatted list `path`.
fn is_in_path(tool: &str, path: &OsStr) -> bool {
    let exe_name = format!("{tool}{EXE_SUFFIX}");
    std::env::split_paths(path).any(|dir| dir.join(&exe_name).is_file())
}

/// Install a pip package via `pip3` or `pip`, whichever is available.
/// Returns true if installation succeeded.
fn install_pip_package(package: &str) -> bool {
    for pip in &["pip3", "pip"] {
        if let Ok(status) = std::process::Command::new(pip)
            .args(["install", package])
            .status()
        {
            if status.success() {
                return true;
            }
        }
    }
    false
}

/// Ensure `tool` (a pip-installable `pip_package`) is present in PATH.
/// Prompts the user before installing, or installs silently when `auto` is
/// true. Returns true if the tool is (now) available.
fn ensure_select_installed(tool: &str, pip_package: &str, auto: bool) -> bool {
    if is_tool_installed(tool) {
        return true;
    }

    if auto {
        eprintln!("'{tool}' is not installed. Installing via pip...");
    } else {
        eprintln!("'{tool}' is not installed.");
        if !prompt_yes_no(&format!("Install {tool} via pip now? [y/N] ")) {
            eprintln!("Cannot proceed without {tool}.");
            return false;
        }
    }

    if install_pip_package(pip_package) {
        eprintln!("{tool} installed successfully.");
        true
    } else {
        eprintln!("Failed to install {tool} via pip. Please install it manually:");
        eprintln!("  pip install {pip_package}");
        false
    }
}

/// Try to install a compatible Vyper compiler and re-compile the file.
fn try_install_and_compile_vyper(
    file: &str,
    vyper_ver: Option<&str>,
    auto: bool,
) -> Option<scirs::sir::Module> {
    // Step 0: Ensure vyper-select itself is present. It is not published on
    // PyPI, so unlike solc-select it cannot be installed with pip.
    if !is_tool_installed("vyper-select") {
        eprintln!("'vyper-select' is not installed. Install it and add it to PATH.");
        return None;
    }

    let pragma = frontend::vyper::extract_pragma(file).ok()??;

    let candidates = frontend::vyper::find_installable_versions(&pragma).ok()?;
    if candidates.is_empty() {
        eprintln!("No published Vyper version satisfies pragma '{pragma}'.");
        return None;
    }
    let best = &candidates[0];

    if !auto {
        eprintln!("No installed Vyper version satisfies pragma '{pragma}'.");
        eprintln!("The latest compatible version is {best}.");
        if !prompt_yes_no(&format!("Install Vyper {best} now? [y/N] ")) {
            return None;
        }
    }

    eprintln!("Installing Vyper {best}...");
    if let Err(e) = frontend::vyper::install_version(best) {
        eprintln!("Failed to install Vyper {best}: {e}");
        return None;
    }
    eprintln!("Vyper {best} installed successfully.");

    frontend::vyper::compile_file(file, vyper_ver).ok()
}

/// Try to install a compatible solc compiler and re-compile the file.
fn try_install_and_compile_solidity(
    file: &str,
    base_path: Option<&str>,
    include_paths: &[String],
    solc_ver: Option<&str>,
    auto: bool,
) -> Option<Vec<SourceUnit>> {
    // Step 0: Ensure solc-select itself is present.
    if !ensure_select_installed("solc-select", "solc-select", auto) {
        return None;
    }

    let pragma = frontend::solidity::extract_pragma(file).ok()??;

    let candidates = frontend::solidity::find_installable_versions(&pragma).ok()?;
    if candidates.is_empty() {
        eprintln!("No published solc version satisfies pragma '{pragma}'.");
        return None;
    }
    let best = &candidates[0];

    if !auto {
        eprintln!("No installed solc version satisfies pragma '{pragma}'.");
        eprintln!("The latest compatible version is {best}.");
        if !prompt_yes_no(&format!("Install solc {best} now? [y/N] ")) {
            return None;
        }
    }

    eprintln!("Installing solc {best}...");
    if let Err(e) = frontend::solidity::install_version(best) {
        eprintln!("Failed to install solc {best}: {e}");
        return None;
    }
    eprintln!("solc {best} installed successfully.");

    parse_input_file(file, base_path, include_paths, solc_ver).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_path_finds_tool_without_running_it() {
        // An empty file cannot run, so only a lookup can find it, as it must
        // find `solc-select`, which fails on `--version`.
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(format!("fake-select{EXE_SUFFIX}")), "").unwrap();
        let path = std::env::join_paths([dir.path()]).unwrap();

        assert!(is_in_path("fake-select", &path));
        assert!(!is_in_path("missing-select", &path));
    }
}
