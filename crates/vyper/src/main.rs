//! CLI binary for compiling Vyper sources into SCIR.

use clap::{Parser, crate_version};
use common::error;
use vyper::{ast::normalize, parser::parse_input_file};

#[derive(Parser, Debug)]
#[command(
    author,
    version = crate_version!(),
    term_width = 80,
    about = "Compiler for Vyper smart contracts to SCIR.",
    long_about = None
)]
pub struct Arguments {
    /// Input Vyper files to be compiled.
    pub input_files: Vec<String>,

    /// Print debugging information.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    /// Print input program (parsed AST).
    #[arg(long, visible_alias = "pip", default_value_t = false)]
    pub print_input_program: bool,

    /// Print normalized program.
    #[arg(long, visible_alias = "pnp", default_value_t = false)]
    pub print_normalized_program: bool,

    /// Print intermediate representation (SCIR).
    #[arg(long, visible_alias = "pir", default_value_t = false)]
    pub print_intermediate_representation: bool,

    /// Verbosity
    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::ErrorLevel>,
}

fn main() {
    error::config();
    env_logger::init();

    let args = Arguments::parse();

    for file in &args.input_files {
        // Parse
        let source_unit = match parse_input_file(file, None) {
            Ok(su) => su,
            Err(err) => {
                eprintln!("Error parsing {file}: {err}");
                std::process::exit(1);
            }
        };

        if args.print_input_program {
            println!("=== Parsed AST for {file} ===");
            println!("{source_unit:#?}");
            println!();
        }

        // Normalize
        let normalized = normalize::run_passes(&source_unit);

        if args.print_normalized_program {
            println!("=== Normalized AST for {file} ===");
            println!("{normalized:#?}");
            println!();
        }

        // Lower to SCIR
        let module = match vyper::irgen::lower_source_unit(&normalized) {
            Ok(m) => m,
            Err(err) => {
                eprintln!("Error lowering {file} to SCIR: {err}");
                std::process::exit(1);
            }
        };

        if args.print_intermediate_representation {
            module.print_pretty();
        }

        println!("Successfully compiled {file}");
    }
}
