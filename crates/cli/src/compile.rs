//! `verazt compile` subcommand — parse a smart contract and print its IR layers.

use clap::{Parser, ValueEnum};
use common::error::{Result, create_error};

/// Supported source languages.
#[derive(Clone, Debug, ValueEnum)]
pub enum Language {
    Solidity,
    Vyper,
}

/// Arguments for the `compile` subcommand.
#[derive(Parser, Debug)]
#[command(about = "Compile a smart contract and print its IR representations")]
pub struct Args {
    /// Input smart contract file(s) (.sol or .vy).
    pub input_files: Vec<String>,

    /// Explicit language override (solidity | vyper). Auto-detected from extension if omitted.
    #[arg(long)]
    pub language: Option<Language>,

    // Solidity-specific options (ignored for Vyper)
    #[arg(long)]
    pub base_path: Option<String>,
    #[arg(long)]
    pub include_path: Vec<String>,
    #[arg(long)]
    pub solc_version: Option<String>,

    /// Print the parsed AST.
    #[arg(long, visible_alias = "pip")]
    pub print_ast: bool,

    /// Print the Source IR (SIR).
    #[arg(long, visible_alias = "pir")]
    pub print_sir: bool,

    /// Print the Analysis IR (AIR).
    #[arg(long)]
    pub print_air: bool,

    /// Print the Verification IR (VIR).
    #[arg(long)]
    pub print_vir: bool,
}

/// Detect the language from the file extension.
fn detect_language(file: &str) -> Result<Language> {
    if file.ends_with(".sol") {
        Ok(Language::Solidity)
    } else if file.ends_with(".vy") {
        Ok(Language::Vyper)
    } else {
        Err(create_error(format!(
            "Cannot detect language for file '{file}'. Use --language to specify it explicitly."
        )))
    }
}

/// Run the compile subcommand.
pub fn run(args: Args) -> Result<()> {
    if args.input_files.is_empty() {
        return Err(create_error("No input files provided.".to_string()));
    }

    for file in &args.input_files {
        let lang = match &args.language {
            Some(l) => match l {
                Language::Solidity => Language::Solidity,
                Language::Vyper => Language::Vyper,
            },
            None => detect_language(file)?,
        };

        match lang {
            Language::Solidity => compile_solidity(file, &args)?,
            Language::Vyper => compile_vyper(file, &args)?,
        }
    }

    Ok(())
}

/// Run the Solidity compilation pipeline for one file.
fn compile_solidity(file: &str, args: &Args) -> Result<()> {
    use langs::solidity;

    let base_path = args.base_path.as_deref();
    let include_paths: &[String] = &args.include_path;
    let solc_ver = args.solc_version.as_deref();

    // Step 1: Parse
    let source_units =
        solidity::parser::parse_input_file(file, base_path, include_paths, solc_ver)?;

    // Step 2: Normalize
    let normalized = solidity::ast::normalize::run_passes(&source_units);

    // Step 3: Print AST if requested
    if args.print_ast {
        println!("=== Parsed AST for {file} ===");
        for su in &normalized {
            su.print_highlighted_code();
            println!();
        }
    }

    for su in &normalized {
        // Step 4: Lower to SIR
        let sir_module = solidity::irgen::lower_source_unit(su)?;

        // Step 5: Print SIR if requested
        if args.print_sir {
            println!("=== SIR for {file} ===");
            sir_module.print_pretty();
        }

        // Step 6: Lower to AIR
        let air_module = mlir::air::lower::lower_module(&sir_module)
            .map_err(|e| create_error(format!("AIR lowering failed: {e}")))?;

        // Step 7: Print AIR if requested
        if args.print_air {
            println!("=== AIR for {file} ===");
            println!("{air_module}");
        }
    }

    // Step 8: VIR (not yet implemented)
    if args.print_vir {
        return Err(create_error(
            "VIR lowering is not yet implemented.".to_string(),
        ));
    }

    println!("Successfully compiled {file}");
    Ok(())
}

/// Run the Vyper compilation pipeline for one file.
fn compile_vyper(file: &str, args: &Args) -> Result<()> {
    use langs::vyper;

    // Step 1: Parse
    let source_unit = vyper::parser::parse_input_file(file, None)?;

    // Step 2: Normalize
    let normalized = vyper::ast::normalize::run_passes(&source_unit);

    // Step 3: Print AST if requested
    if args.print_ast {
        println!("=== Parsed AST for {file} ===");
        println!("{normalized:#?}");
        println!();
    }

    // Step 4: Lower to SIR
    let sir_module = vyper::irgen::lower_source_unit(&normalized)?;

    // Step 5: Print SIR if requested
    if args.print_sir {
        println!("=== SIR for {file} ===");
        sir_module.print_pretty();
    }

    // Step 6: Lower to AIR
    let air_module = mlir::air::lower::lower_module(&sir_module)
        .map_err(|e| create_error(format!("AIR lowering failed: {e}")))?;

    // Step 7: Print AIR if requested
    if args.print_air {
        println!("=== AIR for {file} ===");
        println!("{air_module}");
    }

    // Step 8: VIR (not yet implemented)
    if args.print_vir {
        return Err(create_error(
            "VIR lowering is not yet implemented.".to_string(),
        ));
    }

    println!("Successfully compiled {file}");
    Ok(())
}
