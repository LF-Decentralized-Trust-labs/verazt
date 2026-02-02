use clap::{Parser, arg, crate_version};

#[derive(Parser, Debug)]
#[command(author,
          version = crate_version!(),
          term_width = 80,
          about="Smartproof: smart contract formal verification system.",
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
