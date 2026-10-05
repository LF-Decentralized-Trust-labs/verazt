mod compile;

use analyzer::cli::DEFAULT_CONFIG_FILE;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "verazt",
    about = "Verazt Smart Contract Analyzer",
    version,
    args_conflicts_with_subcommands = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Running `verazt <files>` without a subcommand analyzes the files.
    #[command(flatten)]
    analyze: analyzer::cli::Args,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze smart contracts for bugs and security vulnerabilities (default)
    Analyze(analyzer::cli::Args),
    /// Compile a smart contract and print its IR representations
    Compile(compile::Args),
    /// Generate a default configuration file
    InitConfig {
        /// Output file
        #[arg(default_value = DEFAULT_CONFIG_FILE)]
        output: String,
    },
    /// List available detectors
    ListDetectors,
    /// Show detector information
    ShowDetector {
        /// Detector ID
        id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => analyzer::cli::run(cli.analyze),
        Some(Commands::Analyze(args)) => analyzer::cli::run(args),
        Some(Commands::Compile(args)) => {
            if let Err(err) = compile::run(args) {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
        Some(Commands::InitConfig { output }) => analyzer::cli::init_config(&output),
        Some(Commands::ListDetectors) => analyzer::cli::list_detectors(),
        Some(Commands::ShowDetector { id }) => analyzer::cli::show_detector(&id),
    }
}
