mod compile;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "verazt",
    about = "Verazt Smart Contract Analyzer and Verifier",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compile a smart contract and print its IR representations
    Compile(compile::Args),
    /// Scan smart contracts for bugs and security vulnerabilities
    #[command(trailing_var_arg = true, allow_hyphen_values = true)]
    Scanner { args: Vec<String> },
    /// Verify smart contracts properties
    #[command(trailing_var_arg = true, allow_hyphen_values = true)]
    Verifier { args: Vec<String> },
}

fn main() {
    let cli = Cli::parse();

    // We insert the subcommand name back in at the front so that
    // the target module can parse it using try_parse_from.
    match cli.command {
        Commands::Compile(args) => {
            if let Err(err) = compile::run(args) {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
        Commands::Scanner { args } => {
            let mut all_args = vec!["verazt check".to_string()];
            all_args.extend(args);
            scanner::cli::run(all_args);
        }
        Commands::Verifier { args } => {
            let mut all_args = vec!["verazt verify".to_string()];
            all_args.extend(args);
            verifier::cli::run(all_args);
        }
    }
}
