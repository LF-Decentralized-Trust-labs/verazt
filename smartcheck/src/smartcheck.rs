use clap::{Parser, arg, crate_version};
use color_eyre::eyre::{Result, bail};
use log::debug;
use solidity::{
    ast::{self, SourceUnit},
    normalize,
    parser::ast_parser,
    util,
    util::export::export_debugging_source_unit,
    version::find_compatible_solc_versions,
};

#[derive(Parser, Debug)]
#[command(author,
          version = crate_version!(),
          term_width = 80,
          about="Compiler for Solidity JSON AST.",
          long_about=None)]
pub struct Arguments {
    /// Input Solidity files to be compiled.
    pub input_files: Vec<String>,

    /// The root directory of the source tree, if specified.
    #[arg(long, default_value = None)]
    pub base_path: Option<String>,

    /// Additional directory to look for import files.
    ///
    /// This argument can be specified multiple times.
    #[arg(long, default_value = None)]
    pub include_path: Vec<String>,

    /// Print debugging information.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    /// Configure Solidity compiler version.
    #[arg(long, default_value = None)]
    pub solc_version: Option<String>,

    /// Verbosity
    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::ErrorLevel>,
}

/// Main function
fn main() {}
