//! Module implementing utilities for handling Solc.

use super::tool_name;
use crate::util::export;
use crate::version::{
    self, check_range_constraint, find_compatible_solc_versions, find_solidity_versions,
};
use color_eyre::eyre::{Result, bail};
use node_semver::Version;
use regex::Regex;
use std::{fs::File, io::Write, path::Path, process::Command};

//------------------------------------------------------------------
// Data structure representing JSON AST
//------------------------------------------------------------------

pub struct JsonAst {
    pub json_data: String, // JSON content
    pub file_name: Option<String>,
    pub base_path: Option<String>, // Base path that is used to look for source tree.
}

//------------------------------------------------------------------
// Implementations for JSON AST
//------------------------------------------------------------------

impl JsonAst {
    pub fn new(json_data: &str, input_file: Option<&str>, base_path: Option<&str>) -> Self {
        JsonAst {
            json_data: json_data.to_string(),
            file_name: input_file.map(|s| s.to_string()),
            base_path: base_path.map(|s| s.to_string()),
        }
    }
}

//------------------------------------------------------------------
// Internal functions
//------------------------------------------------------------------

fn get_installed_solc_vers() -> String {
    let cmd_args = " versions".to_string();
    let cmd_output = Command::new(tool_name::SOLC_SELECT_EXE)
        .args(cmd_args.split_whitespace())
        .output();

    match cmd_output {
        Ok(output) => String::from_utf8(output.stdout).unwrap_or_else(|_| "".to_string()),
        Err(err) => panic!("Error when running {}: {}", tool_name::SOLC_SELECT_EXE, err),
    }
}

//------------------------------------------------------------------
// Public functions
//------------------------------------------------------------------

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
    let cmd_output = Command::new(tool_name::SOLC_SELECT_EXE)
        .args(cmd_args.split_whitespace())
        .output();
    if cmd_output.is_err() {
        panic!("Failed to install Solc: {solc_ver}");
    }
}

/// Compile Solidity smart contracts to combined JSON format.
///
/// The two inputs `base_path` and `include_path` are similar to the inputs of
/// Solc.
pub fn compile_solidity_file(
    input_file: &str,
    base_path: Option<&str>,
    include_paths: &[String],
    solc_ver: &str,
) -> Result<JsonAst> {
    info!("Compiling file: {input_file}");
    info!("Input Solc version: {solc_ver:?}");

    let input_file_path = Path::new(input_file);
    if !input_file_path.exists() {
        bail!("Input file does not exist: {}", input_file);
    }

    let base_path = match base_path {
        Some(path) => path,
        None => input_file_path
            .parent()
            .expect("Parent directory of input file should exists")
            .to_str()
            .expect("Parent directory of input file should exists"),
    };

    // Checking Solc version indicated in smart contract source code
    let contract_solc_versions = find_solidity_versions(input_file).map(|v| v.join(", "))?;
    let contract_solc_range = node_semver::Range::parse(&contract_solc_versions)?;
    if !check_range_constraint(&contract_solc_range, ">=0.4.12") {
        bail!("Only support Solidity versions >=0.4.12, but found: {}", &contract_solc_versions);
    }

    let compatible_solc_vers = find_compatible_solc_versions(input_file)
        .unwrap_or_else(|_| panic!("Failed to find Solc version in source code!"));

    // Configure suitable Solc version
    let input_solc_ver = match Version::parse(solc_ver) {
        Ok(ver) => vec![ver],
        Err(err) => bail!(err),
    };
    let common_solc_ver = compatible_solc_vers
        .clone()
        .into_iter()
        .filter(|ver| input_solc_ver.contains(ver))
        .collect::<Vec<Version>>();
    let best_solc_vers = match common_solc_ver.is_empty() {
        false => common_solc_ver,
        true => match compatible_solc_vers.is_empty() {
            false => compatible_solc_vers,
            true => input_solc_ver,
        },
    };

    let mut compilation_errors = vec![];
    for solc_ver in &best_solc_vers {
        debug!("Compiling input contract using Solc: {solc_ver}");
        configure_solc_compiler(solc_ver);

        // Prepare compilation command.
        let solc_path = tool_name::SOLC_EXE;
        let mut args = input_file.to_string();

        // Configure base path and include paths
        if !base_path.is_empty() {
            args += &format!(" --base-path {base_path}");
        }
        for include_path in include_paths {
            args += &format!(" --include-path {include_path}");
        }

        // Solc 0.8.10 and newer don't need the flag `compact-format`
        match version::check_version_constraint(solc_ver, ">=0.4.12 <= 0.8.9") {
            true => args += " --combined-json ast,compact-format",
            false => args += " --combined-json ast",
        }

        // Compile source code to JSON AST
        debug!("Command: SOLC_VERSION={solc_ver} {solc_path} {args}");
        let output = Command::new(solc_path)
            .env("SOLC_VERSION", format!("{solc_ver}"))
            .args(args.split_whitespace())
            .output()?;

        if !output.status.success() {
            let mut msg = format!("\nSolc {solc_ver} failed to compile: {input_file}");

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

                return Ok(JsonAst::new(json_data, Some(input_file), Some(base_path)));
            }
            Err(_) => bail!("Failed to parse JSON of: {}", input_file),
        }
    }

    bail!(
        "All Solc compilers failed to compile: {}\n\n{}",
        input_file,
        compilation_errors.join("\n")
    );
}
