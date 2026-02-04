use clap::{Parser, crate_version};
use extlib::error;
use solidity::{
    ast::utils::export::export_debugging_source_unit,
    ast::SourceUnit,
    compile::compile_input_file,
    // passes,
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

    /// Print input program.
    #[arg(long, visible_alias = "pip", default_value_t = false)]
    pub print_input_program: bool,

    /// Verbosity
    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::ErrorLevel>,
}

/// Main function
fn main() {
    env_logger::init();
    error::config();

    // Parse command line arguments
    let args = Arguments::parse();

    // Parse input files
    let solc_ver = args.solc_version.as_deref();
    let base_path = args.base_path.as_deref();
    let include_paths: &[String] = &args.include_path;

    let _input_source_units: Vec<SourceUnit> = args
        .input_files
        .iter()
        .flat_map(|file| {
            let source_units = match compile_input_file(file, base_path, include_paths, solc_ver) {
                Ok(source_units) => source_units,
                Err(err) => panic!("{}", err),
            };
            if args.print_input_program {
                println!("Source units after parsing:");
            }
            for source_unit in &source_units {
                if args.print_input_program {
                    source_unit.print_highlighted_code();
                    println!();
                }
                if let Err(err) = export_debugging_source_unit(source_unit, "parsed") {
                    panic!("{}", err);
                }
            }
            source_units
        })
        .collect();
}
