//! Vyper compiler invocation and parsing.

pub mod json_ast_parser;

pub use json_ast_parser::AstParser;

use crate::ast::source_unit::SourceUnit;
use extlib::{error::Result, fail};
use regex::Regex;
use std::io::Write;
use std::path::Path;
use std::process::Command;

const VYPER: &str = "vyper";

/// Parse a `.vy` file into a Vyper AST source unit.
pub fn parse_input_file(input_file: &str) -> Result<SourceUnit> {
    println!("Compiling Vyper input file: {input_file}");
    let input_path = Path::new(input_file);
    if !input_path.exists() {
        fail!("Input file does not exist: {input_file}");
    }

    // Read source code to check version pragma
    let source_code = std::fs::read_to_string(input_file)
        .map_err(|e| extlib::error::create_error(format!("Failed to read input file: {e}")))?;

    if let Some(version) = extract_version_pragma(&source_code) {
        log::debug!("Detected Vyper version pragma: {version}");
    }

    // Invoke vyper compiler to get JSON AST
    let json_ast = invoke_vyper_compiler(input_file)?;

    // Parse the JSON AST
    AstParser::parse(&json_ast, input_file)
}

/// Parse Vyper source code (string) into a Vyper AST source unit.
pub fn parse_vyper_source_code(source_code: &str) -> Result<SourceUnit> {
    // Write source code to a temporary file
    let temp_dir = tempfile::tempdir()
        .map_err(|e| extlib::error::create_error(format!("Failed to create temp dir: {e}")))?;
    let temp_file = temp_dir.path().join("temp_contract.vy");
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| extlib::error::create_error(format!("Failed to create temp file: {e}")))?;
    file.write_all(source_code.as_bytes())
        .map_err(|e| extlib::error::create_error(format!("Failed to write temp file: {e}")))?;

    let temp_path = temp_file.to_string_lossy().to_string();

    // Invoke vyper compiler
    let json_ast = invoke_vyper_compiler(&temp_path)?;

    // Parse the JSON AST
    AstParser::parse(&json_ast, "<source>")
}

/// Parse Vyper source code from a JSON AST string (for testing without
/// compiler).
pub fn parse_from_json(json_str: &str, file_path: &str) -> Result<SourceUnit> {
    AstParser::parse(json_str, file_path)
}

/// Invoke the Vyper compiler and return the JSON AST.
fn invoke_vyper_compiler(input_file: &str) -> Result<String> {
    log::debug!("Running: {VYPER} -f ast {input_file}");

    let output = Command::new(VYPER)
        .args(["-f", "ast", input_file])
        .output()
        .map_err(|e| {
            extlib::error::create_error(format!(
                "Failed to run `{VYPER}`. Is Vyper installed? Error: {e}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        fail!("Vyper compilation failed for {input_file}:\n{stderr}");
    }

    let json_str = String::from_utf8(output.stdout)
        .map_err(|e| extlib::error::create_error(format!("Invalid UTF-8 in vyper output: {e}")))?;

    if json_str.trim().is_empty() {
        fail!("Vyper compiler returned empty output for {input_file}");
    }

    Ok(json_str)
}

/// Extract the version pragma from Vyper source code.
fn extract_version_pragma(source: &str) -> Option<String> {
    let re = Regex::new(r"#\s*@version\s+(.+)").ok()?;
    re.captures(source)
        .map(|caps| caps.get(1).unwrap().as_str().trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_pragma() {
        let source = "# @version ^0.3.9\n\nowner: address";
        let version = extract_version_pragma(source);
        assert_eq!(version, Some("^0.3.9".to_string()));
    }

    #[test]
    fn test_extract_version_pragma_missing() {
        let source = "owner: address\n";
        let version = extract_version_pragma(source);
        assert_eq!(version, None);
    }

    #[test]
    fn test_parse_from_json() {
        let json = r#"{
            "ast_type": "Module",
            "body": [],
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 0
        }"#;
        let su = parse_from_json(json, "test.vy").unwrap();
        assert_eq!(su.path, "test.vy");
        assert!(su.body.is_empty());
    }
}
