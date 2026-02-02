use clap::{Parser, crate_version};
use yul::parser;

/// Data structure representing CLI arguments.
///
/// See more settings at:
/// <https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html>
#[derive(Parser, Debug)]
#[command(author,
          version = crate_version!(),
          term_width = 80,
          about="Compiler for smart contract Yul intermediate code.",
          long_about=None)]
pub struct Arguments {
    /// Input file to be analyzed.
    pub input_file: String,

    /// Linting configuration file
    #[arg(long = "config", default_value = None)]
    pub config_file: Option<String>,

    /// Print debugging information.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}

/// Main function
fn main() {
    // Parse command line arguments
    let args = Arguments::parse();

    // Parse input file
    let input_file = args.input_file;
    println!("Input file: {}", &input_file);
    let source_unit = match parser::parse_input_file(&input_file) {
        Ok(source_unit) => source_unit,
        Err(err) => panic!("{}", err),
    };
    println!("Yul AST:\n{source_unit}");
}
