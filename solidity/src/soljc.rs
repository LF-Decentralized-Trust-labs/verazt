//! Compiler for compiling Solidity AST in JSON format.

use clap::{Parser, crate_version};
use solidity::{
    ast::SourceUnit, compile::compile_input_file, normalize,
    util::export::export_debugging_source_unit,
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

    /// Normalize AST.
    #[arg(long, default_value_t = false)]
    pub normalize: bool,

    /// Transform AST ot IR.
    #[arg(long, default_value_t = false)]
    pub transform: bool,

    /// Configure Solidity compiler version.
    #[arg(long, default_value = None)]
    pub solc_version: Option<String>,

    /// Print input program.
    #[arg(long, visible_alias = "pip", default_value_t = false)]
    pub print_input_program: bool,

    /// Print normalized program.
    #[arg(long, visible_alias = "pnp", default_value_t = false)]
    pub print_normalized_program: bool,

    /// Print intermediate representation program.
    #[arg(long, visible_alias = "pir", default_value_t = false)]
    pub print_intermediate_representation: bool,

    /// Profile time statistics.
    #[arg(long, default_value_t = false)]
    pub profile_time: bool,

    /// Verbosity
    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::ErrorLevel>,
}

/// Main function
fn main() {
    // Use `color_eyre`'s error handling.
    let _ = color_eyre::install();

    // // Pre-configuration
    // panic::override_panic_message("Run with `RUST_BACKTRACE=1` and `-D` to
    // display backtrace.");

    // Parse command line arguments
    let args = Arguments::parse();

    // Enable profiling execution time.
    if args.profile_time {
        time_graph::enable_data_collection(true);
    }

    pretty_env_logger::env_logger::Builder::new()
        .format_timestamp(None)
        .format_target(false) // use .format_target(true) to print path/module
        .filter_level(args.verbose.log_level_filter())
        .init();

    // Parse input files
    let solc_ver = args.solc_version.as_deref();
    let base_path = args.base_path.as_deref();
    let include_paths: &[String] = &args.include_path;

    let input_source_units: Vec<SourceUnit> = args
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

    // Normalize AST
    let normalized_source_units = normalize::normalize_source_units(&input_source_units);

    if args.print_normalized_program {
        println!("Source unit AST after normalization:");
        for source_unit in &normalized_source_units {
            source_unit.print_highlighted_code();
        }
    }

    // Export normalized source units
    for source_unit in &normalized_source_units {
        if let Err(err) = export_debugging_source_unit(source_unit, "normalized") {
            panic!("{}", err);
        }
    }

    // // Transform AST to IR:
    // let mut transformed_source_units = vec![];
    // for source_unit in &normalized_source_units {
    //     let source_unit = match transform::transform_source_unit(source_unit)
    // {         Ok(sunit) => sunit,
    //         Err(err) => panic!("{}", err),
    //     };

    //     if args.print_intermediate_representation {
    //         print_long_double_separator_line!();
    //         println!("Source unit SAST:");
    //         source_unit.print_pretty();
    //     }

    //     // // Export transformed source units
    //     // if let Err(err) =
    // ir::export_debugging_source_unit(source_unit.borrow(), "sast") {
    //     //     panic!("{}", err);
    //     // }

    //     transformed_source_units.push(source_unit);
    // }

    // // Print execution time of some profiled functions
    // if args.profile_time {
    //     let graph = time_graph::get_full_graph();
    //     println!("\n\nExecution time:\n{}", graph.as_short_table());
    // }
}
