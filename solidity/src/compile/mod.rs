pub mod json_ast;
pub mod typ;
pub mod version;
pub mod yul;

pub use json_ast::ast_parser;
pub use typ::type_parser;
pub use version::version_parser;

use crate::{
    ast::{self, SourceUnit, utils::export},
    ast::utils::version::{
        check_range_constraint, check_version_constraint, find_compatible_solc_versions,
        find_pragma_solidity_versions,
    },
};
use self::json_ast::{AstParser, JsonAst};
use extlib::{error::Result, fail};
use node_semver::Version;
use regex::Regex;
use std::{fs::File, io::Write, path::Path, process::Command};

// Tool names
const SOLC: &str = "solc";
const SOLC_SELECT: &str = "solc-select";

/// Save a string to a temporary file of a given name.
///
/// Return the output file path.
fn save_to_temporary_file(file_content: &str, file_name: &str) -> Result<String> {
    let output_dir_path = match tempfile::tempdir() {
        Ok(dir) => dir.keep(),
        Err(err) => fail!(err),
    };
    let output_file_path = output_dir_path.join(file_name);
    let mut output_file = File::create(&output_file_path)?;
    match output_file.write_all(file_content.as_bytes()) {
        Ok(_) => match output_file_path.to_str() {
            Some(path) => Ok(path.to_string()),
            None => fail!("Output file path not found!"),
        },
        Err(err) => fail!(err),
    }
}

/// Save multiple strings to multiple temporary files.
///
/// Return the output file path.
fn save_to_temporary_files(source_code_list: &[(&str, &str)]) -> Result<Vec<String>> {
    let output_dir_path = match tempfile::tempdir() {
        Ok(dir) => dir.keep(),
        Err(err) => fail!(err),
    };

    let mut output_files: Vec<String> = vec![];

    for file_name_content in source_code_list.iter() {
        let (file_name, file_content) = file_name_content;

        let output_file_path = output_dir_path.join(file_name);
        let mut output_file = File::create(&output_file_path)?;
        match output_file.write_all(file_content.as_bytes()) {
            Ok(_) => match output_file_path.to_str() {
                Some(path) => output_files.push(path.to_string()),
                None => fail!("Output file path not found!"),
            },
            Err(err) => fail!(err),
        }
    }

    Ok(output_files)
}

fn get_installed_solc_vers() -> String {
    let cmd_args = " versions".to_string();
    let cmd_output = Command::new(SOLC_SELECT)
        .args(cmd_args.split_whitespace())
        .output();

    match cmd_output {
        Ok(output) => String::from_utf8(output.stdout).unwrap_or_else(|_| "".to_string()),
        Err(err) => panic!("Error when running {SOLC_SELECT}: {err}"),
    }
}

pub fn configure_solc_compiler(solc_ver: &Version) {
    // Check settings of solc-select
    let installed_solcs = get_installed_solc_vers();
    let solc_ver_regex = Regex::new(r"(\d+\.\d+\.\d+)").expect("Version regex should be valid");

    for mat in solc_ver_regex.find_iter(&installed_solcs) {
        if let Ok(ver) = Version::parse(mat.as_str())
            && ver.eq(solc_ver)
        {
            return;
        }
    }

    debug!("Installing Solc {}...", &solc_ver);
    let cmd_args = " install ".to_string() + &solc_ver.to_string();
    let cmd_output = Command::new(SOLC_SELECT)
        .args(cmd_args.split_whitespace())
        .output();
    if cmd_output.is_err() {
        panic!("Failed to install Solc: {solc_ver}");
    }
}

/// Compile input file to source units in AST format.
///
/// The two inputs `base_path` and `include_path` are similar to the inputs of
/// Solc. Auto detect Solc version if not provided
pub fn compile_input_file(
    input_file: &str,
    base_path: Option<&str>,
    include_paths: &[String],
    solc_ver: Option<&str>,
) -> Result<Vec<ast::SourceUnit>> {
    println!("Compiling input file: {input_file}");
    let input_file_path = Path::new(input_file);
    if !input_file_path.exists() {
        fail!("Input file does not exist: {}", input_file);
    }

    // Checking Solc version indicated in smart contract source code
    let pragma_solc_ver = match find_pragma_solidity_versions(input_file) {
        Ok(vers) if !vers.is_empty() => Some(vers.join(", ")),
        _ => None,
    };
    if let Some(ver) = &pragma_solc_ver {
        let pragma_solc_range = node_semver::Range::parse(ver)
            .or_else(|_| fail!("Failed to parse pragma version: '{}'", ver))?;
        if !check_range_constraint(&pragma_solc_range, ">=0.4.12") {
            fail!("Only support Solidity versions >=0.4.12, but found: {}", ver);
        }
    }

    let compatible_solc_vers = find_compatible_solc_versions(&pragma_solc_ver)
        .unwrap_or_else(|_| panic!("Failed to find Solc version in source code!"));

    // Configure suitable Solc version
    let input_solc_range = match solc_ver {
        None => None,
        Some(v) => Some(
            node_semver::Range::parse(v)
                .or_else(|_| fail!("Failed to parse Solc version constraint: '{}'", v))?,
        ),
    };

    let common_solc_ver = compatible_solc_vers
        .clone()
        .into_iter()
        .filter(|ver| match &input_solc_range {
            None => false,
            Some(range) => range.satisfies(ver),
        })
        .collect::<Vec<Version>>();
    let best_solc_vers = match common_solc_ver.is_empty() {
        false => common_solc_ver,
        true => match compatible_solc_vers.is_empty() {
            false => compatible_solc_vers,
            true => fail!("Unable to find a Solc version to compile: {input_file}"),
        },
    };

    let mut compilation_errors = vec![];
    for solc_ver in &best_solc_vers {
        debug!("Compiling input contract using Solc: {solc_ver}");
        configure_solc_compiler(solc_ver);

        // Prepare compilation command.
        let mut args = input_file.to_string();

        // Configure base path and include paths
        if let Some(path) = base_path
            && check_version_constraint(solc_ver, ">=0.7.0")
        {
            args += &format!(" --base-path {path}");
        }
        if !include_paths.is_empty() && check_version_constraint(solc_ver, ">=0.8.8") {
            for include_path in include_paths {
                args += &format!(" --include-path {include_path}");
            }
        }

        // Solc 0.8.10 and newer don't need the flag `compact-format`
        match check_version_constraint(solc_ver, ">=0.4.12 <= 0.8.9") {
            true => args += " --combined-json ast,compact-format",
            false => args += " --combined-json ast",
        }

        // Compile source code to JSON AST
        debug!("Command: SOLC_VERSION={solc_ver} solc {args}");
        let output = Command::new(SOLC)
            .env("SOLC_VERSION", format!("{solc_ver}"))
            .args(args.split_whitespace())
            .output()?;

        if !output.status.success() {
            let mut msg = format!("\n{SOLC} {solc_ver} failed to compile: {input_file}");

            if let Ok(err) = String::from_utf8(output.stderr.to_vec()) {
                let mut err_msg = err.as_str().trim();
                if err_msg.starts_with("Error: ") {
                    err_msg = &err_msg["Error: ".len()..];
                }
                msg += &format!("\n\nCompilation error: {err_msg}\n");
                compilation_errors.push(msg.to_string());
            };
            debug!("{msg}");
            continue;
        }

        match std::str::from_utf8(&output.stdout) {
            Ok(json_data) => {
                let (output_dir, file_stem) = export::prepare_logging_directory(input_file)?;
                let output_file_path = output_dir.join(format!("{file_stem}.sol.json"));
                println!("Export JSON output to: {}", output_file_path.display());
                let mut file = File::create(output_file_path)?;
                file.write_all(json_data.as_bytes())?;
                let json_ast = JsonAst::new(json_data, Some(input_file), base_path);
                let mut parser = AstParser::new(&json_ast);
                match parser.parse_solidity_json() {
                    Ok(source_units) => return Ok(source_units),
                    Err(err) => fail!(err),
                }
            }
            Err(_) => fail!("Failed to parse JSON of: {}", input_file),
        }
    }

    fail!(
        "All Solc compilers failed to compile: {}\n\n{}",
        input_file,
        compilation_errors.join("\n")
    )
}

/// Function to parse a Solidity source code string to internal AST.
///
/// `solc_ver` is the Solidity version, empty string means unknown version.
pub fn compile_solidity_source_code(source_code: &str, solc_ver: &str) -> Result<Vec<SourceUnit>> {
    // Save the source code to a temporarily Solidity file
    let solidity_file = match save_to_temporary_file(source_code, "contract.sol") {
        Ok(filename) => filename,
        Err(_) => fail!("Failed to save input contract to file"),
    };

    // Parse the Solidity file to internal AST.
    compile_input_file(&solidity_file, None, &[], Some(solc_ver))
}

/// Function to parse a list of Solidity source code strings to internal AST.
///
/// `source_code_list` is a list of source code string and file name pairs.
pub fn compile_solidity_source_code_list(
    source_code_list: &[(&str, &str)],
    solc_ver: &str,
) -> Result<Vec<SourceUnit>> {
    // Save the source code to a temporarily Solidity file
    let solidity_files = match save_to_temporary_files(source_code_list) {
        Ok(files) => files,
        Err(_) => fail!("Failed to save input contract to files"),
    };
    // Parse Solidity files to internal AST.
    let mut output_sunits: Vec<SourceUnit> = vec![];
    for input_file in solidity_files {
        let input_path = Path::new(&input_file);
        let base_path = input_path.parent().and_then(|p| p.to_str());
        let sunits = compile_input_file(&input_file, base_path, &[], Some(solc_ver))?;
        sunits.iter().for_each(|sunit| {
            if !output_sunits.iter().any(|sunit2| sunit.path == sunit2.path) {
                output_sunits.push(sunit.clone())
            }
        })
    }
    // Return result.
    Ok(output_sunits)
}
