//! `verazt compile` subcommand — parse a smart contract and print its IR
//! layers.

use clap::{Parser, ValueEnum};
use common::error::{Result, create_error};
use common::utils::{print_section_header, print_subsection_header};
use scirs::verify::VerifyError;

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

    /// Print debugging information.
    #[arg(short, long)]
    pub debug: bool,

    /// Print the parsed AST.
    #[arg(long, visible_alias = "pip")]
    pub print_ast: bool,

    /// Print the Source IR (SIR).
    #[arg(long, visible_alias = "pir")]
    pub print_sir: bool,

    /// Print the Core IR (CIR).
    #[arg(long)]
    pub print_cir: bool,

    /// Print the Analysis IR (BIR).
    #[arg(long)]
    pub print_air: bool,

    /// Print the Functional IR (FIR).
    #[arg(long)]
    pub print_fir: bool,
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

/// Print the verification header before running passes.
fn print_verify_header(ir_name: &str) {
    println!("Running {ir_name} Verification passes:\n");
}

/// Report verification results after passes have been printed.
/// Returns Ok(()) if no errors, or Err with a formatted message.
fn report_verify_result(ir_name: &str, result: std::result::Result<(), Vec<VerifyError>>) -> Result<()> {
    println!();
    match result {
        Ok(()) => {
            println!("All {ir_name} verification passes succeed!");
            Ok(())
        }
        Err(errors) => {
            println!("Error Details:");
            // Group errors by pass name, preserving insertion order.
            let mut grouped: Vec<(&str, Vec<&str>)> = Vec::new();
            for e in &errors {
                if let Some((_pass, msgs)) = grouped.iter_mut().find(|(p, _)| *p == e.pass) {
                    msgs.push(&e.message);
                } else {
                    grouped.push((e.pass, vec![&e.message]));
                }
            }
            for (pass, msgs) in &grouped {
                println!();
                println!("{pass}:");
                for (i, msg) in msgs.iter().enumerate() {
                    println!("  {}. {msg}", i + 1);
                }
            }
            println!();
            Err(create_error(format!(
                "{ir_name} verification failed with {} errors",
                errors.len()
            )))
        }
    }
}

/// Run the Solidity compilation pipeline for one file.
fn compile_solidity(file: &str, args: &Args) -> Result<()> {
    use frontend::solidity;

    let base_path = args.base_path.as_deref();
    let include_paths: &[String] = &args.include_path;
    let solc_ver = args.solc_version.as_deref();

    // Step 1: Parse
    let source_units =
        solidity::parsing::parse_input_file(file, base_path, include_paths, solc_ver)?;

    // Step 2: Print AST if requested (before normalization — source-faithful)
    if args.print_ast || args.debug {
        print_section_header("Solidity AST");
        for su in &source_units {
            su.print_highlighted_code();
            println!();
        }
    }

    // Verify AST
    if args.debug {
        print_subsection_header("Solidity AST Verification");
        print_verify_header("AST");
        report_verify_result(
            "AST",
            frontend::solidity::ast::verifier::verify(&source_units, true),
        )?;
    }

    // Step 3: Normalize + lower to SIR (integrated in sir::lower)
    let sir_modules = solidity::lowering::lower_source_units(&source_units)?;

    for sir_module in &sir_modules {
        // Step 4: Print SIR if requested
        if args.print_sir || args.debug {
            print_section_header("SIR");
            sir_module.print_pretty();
        }

        // Verify SIR
        if args.debug {
            print_subsection_header("SIR Verification");
            print_verify_header("SIR");
            report_verify_result("SIR", scirs::sir::verifier::verify(sir_module, true))?;
        }

        // Step 5: Lower SIR → CIR
        let cir_module = scirs::sir::lower::lower_module(sir_module)
            .map_err(|e| create_error(format!("CIR lowering failed: {e}")))?;

        if args.print_cir || args.debug {
            print_section_header("CIR");
            cir_module.print_pretty();
        }

        // Verify CIR
        if args.debug {
            print_subsection_header("CIR Verification");
            print_verify_header("CIR");
            report_verify_result("CIR", scirs::cir::verifier::verify(&cir_module, true))?;
        }

        // Step 6: Lower CIR → BIR
        let air_module = scirs::cir::lower::lower_module(&cir_module)
            .map_err(|e| create_error(format!("BIR lowering failed: {e}")))?;

        // Step 7: Print BIR if requested
        if args.print_air || args.debug {
            print_section_header("BIR");
            print!("{air_module}");
        }

        // Verify BIR
        if args.debug {
            print_subsection_header("BIR Verification");
            print_verify_header("BIR");
            report_verify_result("BIR", scirs::bir::verifier::verify(&air_module, true))?;
        }

        // Step 8: Lower BIR → FIR
        let fir_module = scirs::bir::lower::lower_module(&air_module);

        if args.print_fir || args.debug {
            print_section_header("FIR");
            print!("{fir_module}");
        }

        // Verify FIR
        if args.debug {
            print_subsection_header("FIR Verification");
            print_verify_header("FIR");
            report_verify_result("FIR", scirs::fir::verifier::verify(&fir_module, true))?;
        }
    }

    println!("Successfully compiled {file}");
    Ok(())
}

/// Run the Vyper compilation pipeline for one file.
fn compile_vyper(file: &str, args: &Args) -> Result<()> {
    use frontend::vyper;

    // Step 1: Parse
    let source_unit = vyper::parsing::parse_input_file(file, None)?;

    // Step 2: Print AST if requested (before normalization — source-faithful)
    if args.print_ast || args.debug {
        print_section_header("Vyper AST");
        let ast_str = format!("{source_unit:#?}");
        println!("{}", ast_str.replace("    ", "  "));
        println!();
    }

    // Step 3: Normalize + lower to SIR (integrated in sir::lower)
    let sir_module = vyper::lowering::lower_source_unit_normalized(&source_unit)?;

    // Step 4: Print SIR if requested
    if args.print_sir || args.debug {
        print_section_header("SIR");
        sir_module.print_pretty();
    }

    // Verify SIR
    if args.debug {
        print_subsection_header("SIR Verification");
        print_verify_header("SIR");
        report_verify_result("SIR", scirs::sir::verifier::verify(&sir_module, true))?;
    }

    // Step 5: Lower SIR → CIR
    let cir_module = scirs::sir::lower::lower_module(&sir_module)
        .map_err(|e| create_error(format!("CIR lowering failed: {e}")))?;

    if args.print_cir || args.debug {
        print_section_header("CIR");
        cir_module.print_pretty();
    }

    // Verify CIR
    if args.debug {
        print_subsection_header("CIR Verification");
        print_verify_header("CIR");
        report_verify_result("CIR", scirs::cir::verifier::verify(&cir_module, true))?;
    }

    // Step 6: Lower CIR → BIR
    let air_module = scirs::cir::lower::lower_module(&cir_module)
        .map_err(|e| create_error(format!("BIR lowering failed: {e}")))?;

    // Step 7: Print BIR if requested
    if args.print_air || args.debug {
        print_section_header("BIR");
        print!("{air_module}");
    }

    // Verify BIR
    if args.debug {
        print_subsection_header("BIR Verification");
        print_verify_header("BIR");
        report_verify_result("BIR", scirs::bir::verifier::verify(&air_module, true))?;
    }

    // Step 8: Lower BIR → FIR
    let fir_module = scirs::bir::lower::lower_module(&air_module);

    if args.print_fir || args.debug {
        print_section_header("FIR");
        print!("{fir_module}");
    }

    // Verify FIR
    if args.debug {
        print_subsection_header("FIR Verification");
        print_verify_header("FIR");
        report_verify_result("FIR", scirs::fir::verifier::verify(&fir_module, true))?;
    }

    println!("Successfully compiled {file}");
    Ok(())
}
