//! `verazt compile` subcommand — parse a smart contract and print its IR
//! layers.

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

    /// Explicit language override (solidity | vyper). Auto-detected from
    /// extension if omitted.
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

    /// Print the Analysis IR (BIR).
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
    use frontend::solidity;

    let base_path = args.base_path.as_deref();
    let include_paths: &[String] = &args.include_path;
    let solc_ver = args.solc_version.as_deref();

    // Step 1: Parse
    let source_units =
        solidity::parser::parse_input_file(file, base_path, include_paths, solc_ver)?;

    // Step 2: Print AST if requested (before normalization — source-faithful)
    if args.print_ast {
        println!("=== Parsed AST for {file} ===");
        for su in &source_units {
            su.print_highlighted_code();
            println!();
        }
    }

    // Step 3: Normalize + lower to SIR (integrated in sir::lower)
    let sir_modules = solidity::lower::lower_source_units(&source_units)?;

    for sir_module in &sir_modules {
        // Step 4: Print SIR if requested
        if args.print_sir {
            println!("=== SIR for {file} ===");
            sir_module.print_pretty();
        }

        // Step 5: Lower SIR → CIR
        let cir_module = scirs::cir::lower::lower_module(sir_module)
            .map_err(|e| create_error(format!("CIR lowering failed: {e}")))?;

        // Step 6: Lower CIR → BIR
        let air_module = scirs::bir::lower::lower_module(&cir_module)
            .map_err(|e| create_error(format!("BIR lowering failed: {e}")))?;

        // Step 7: Print BIR if requested
        if args.print_air {
            println!("=== BIR for {file} ===");
            println!("{air_module}");
        }
    }

    // Step 8: VIR (not yet implemented)
    if args.print_vir {
        return Err(create_error("VIR lowering is not yet implemented.".to_string()));
    }

    println!("Successfully compiled {file}");
    Ok(())
}

/// Run the Vyper compilation pipeline for one file.
fn compile_vyper(file: &str, args: &Args) -> Result<()> {
    use frontend::vyper;

    // Step 1: Parse
    let source_unit = vyper::parser::parse_input_file(file, None)?;

    // Step 2: Print AST if requested (before normalization — source-faithful)
    if args.print_ast {
        println!("=== Parsed AST for {file} ===");
        println!("{source_unit:#?}");
        println!();
    }

    // Step 3: Normalize + lower to SIR (integrated in sir::lower)
    let sir_module = vyper::lower::lower_source_unit_normalized(&source_unit)?;

    // Step 4: Print SIR if requested
    if args.print_sir {
        println!("=== SIR for {file} ===");
        sir_module.print_pretty();
    }

    // Step 5: Lower SIR → CIR
    let cir_module = scirs::cir::lower::lower_module(&sir_module)
        .map_err(|e| create_error(format!("CIR lowering failed: {e}")))?;

    // Step 6: Lower CIR → BIR
    let air_module = scirs::bir::lower::lower_module(&cir_module)
        .map_err(|e| create_error(format!("BIR lowering failed: {e}")))?;

    // Step 7: Print BIR if requested
    if args.print_air {
        println!("=== BIR for {file} ===");
        println!("{air_module}");
    }

    // Step 8: VIR (not yet implemented)
    if args.print_vir {
        return Err(create_error("VIR lowering is not yet implemented.".to_string()));
    }

    println!("Successfully compiled {file}");
    Ok(())
}
