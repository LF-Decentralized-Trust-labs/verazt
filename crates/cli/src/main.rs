use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "verazt", about = "Verazt Smart Contract Analyzer and Verifier", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze smart contracts for vulnerabilities
    #[command(trailing_var_arg = true, allow_hyphen_values = true)]
    Analyze {
        args: Vec<String>,
    },
    /// Verify smart contracts properties
    #[command(trailing_var_arg = true, allow_hyphen_values = true)]
    Verify {
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    
    // We insert the subcommand name back in at the front so that 
    // the target module can parse it using try_parse_from.
    match cli.command {
        Commands::Analyze { args } => {
            let mut all_args = vec!["verazt analyze".to_string()];
            all_args.extend(args);
            analyze::cli_entry::run(all_args);
        }
        Commands::Verify { args } => {
            let mut all_args = vec!["verazt verify".to_string()];
            all_args.extend(args);
            verify::cli_entry::run(all_args);
        }
    }
}
