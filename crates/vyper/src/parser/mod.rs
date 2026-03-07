//! Vyper compiler invocation and parsing.

pub mod json_ast_parser;

pub use json_ast_parser::AstParser;

use crate::ast::source_unit::SourceUnit;
use common::{error::Result, fail};
use node_semver::{Range, Version};
use regex::Regex;
use std::io::Write;
use std::path::Path;
use std::process::Command;

const VYPER: &str = "vyper";
const VYPER_SELECT: &str = "vyper-select";

// ============================================================================
// Version management helpers
// ============================================================================

/// List installed Vyper versions via `vyper-select versions`.
fn get_installed_vyper_vers() -> Vec<String> {
    let output = Command::new(VYPER_SELECT).arg("versions").output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    // Accept lines that look like semver versions
                    if Version::parse(trimmed).is_ok() {
                        Some(trimmed.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        }
        Err(e) => {
            log::warn!("Failed to run `{VYPER_SELECT} versions`: {e}");
            vec![]
        }
    }
}

/// Ensure a specific Vyper version is installed and activated via
/// `vyper-select`.
pub fn configure_vyper_compiler(ver: &Version) -> Result<()> {
    let installed = get_installed_vyper_vers();
    let ver_str = ver.to_string();

    // Install if not already present
    if !installed.iter().any(|v| v == &ver_str) {
        log::debug!("Installing Vyper {ver}...");
        let status = Command::new(VYPER_SELECT)
            .args(["install", &ver_str])
            .status();
        match status {
            Ok(s) if !s.success() => fail!("vyper-select install {ver} exited with: {s}"),
            Err(e) => fail!("Failed to run vyper-select install {ver}: {e}"),
            _ => {}
        }
    }

    // Activate
    log::debug!("Activating Vyper {ver}...");
    let status = Command::new(VYPER_SELECT).args(["use", &ver_str]).status();
    match status {
        Ok(s) if !s.success() => fail!("vyper-select use {ver} exited with: {s}"),
        Err(e) => fail!("Failed to run vyper-select use {ver}: {e}"),
        _ => {}
    }

    Ok(())
}

/// Find compatible installed Vyper versions given an optional pragma range.
///
/// Returns versions sorted newest-first. Falls back to all installed versions
/// if no pragma is provided.
pub fn find_compatible_vyper_versions(pragma_ver: &Option<String>) -> Result<Vec<Version>> {
    let installed = get_installed_vyper_vers();
    let mut versions: Vec<Version> = installed
        .iter()
        .filter_map(|v| Version::parse(v).ok())
        .collect();

    // Sort newest first
    versions.sort_by(|a, b| b.cmp(a));

    if let Some(pragma) = pragma_ver {
        if let Ok(range) = Range::parse(pragma) {
            let filtered: Vec<Version> = versions
                .into_iter()
                .filter(|v| range.satisfies(v))
                .collect();
            if filtered.is_empty() {
                fail!("No installed Vyper version satisfies pragma: {pragma}");
            }
            return Ok(filtered);
        } else {
            log::warn!("Could not parse Vyper pragma '{pragma}' as semver range, ignoring.");
        }
    }

    if versions.is_empty() {
        fail!(
            "No Vyper versions installed. \
             Install one via: vyper-select install <version>"
        );
    }

    Ok(versions)
}

/// Query PyPI for all published `vyper` versions satisfying `pragma`.
/// Returns versions sorted newest-first.
pub fn find_installable_vyper_versions(pragma: &str) -> Result<Vec<Version>> {
    let response: serde_json::Value = ureq::get("https://pypi.org/pypi/vyper/json")
        .call()
        .map_err(|e| common::error::create_error(format!("Failed to query PyPI: {e}")))?
        .into_json()
        .map_err(|e| common::error::create_error(format!("Failed to parse PyPI response: {e}")))?;
    let range = Range::parse(pragma)
        .map_err(|e| common::error::create_error(format!("Invalid pragma '{pragma}': {e}")))?;
    let mut versions: Vec<Version> = response["releases"]
        .as_object()
        .unwrap_or(&serde_json::Map::new())
        .keys()
        .filter_map(|k| Version::parse(k).ok())
        .filter(|v| range.satisfies(v))
        .collect();
    versions.sort_by(|a, b| b.cmp(a));
    Ok(versions)
}

/// Check whether a semver range intersects with a constraint string.
fn check_range_constraint(range: &Range, constraint: &str) -> bool {
    if let Ok(constraint_range) = Range::parse(constraint) {
        // A simple heuristic: check a set of representative versions.
        // We test all minor versions from 0.2.0 to 0.4.20.
        for major in 0..=0u64 {
            for minor in 2..=4u64 {
                for patch in 0..=20u64 {
                    let v = Version { major, minor, patch, build: vec![], pre_release: vec![] };
                    if range.satisfies(&v) && constraint_range.satisfies(&v) {
                        return true;
                    }
                }
            }
        }
        false
    } else {
        false
    }
}

// ============================================================================
// Parsing entry points
// ============================================================================

/// Parse a `.vy` file into a Vyper AST source unit.
///
/// `vyper_ver` optionally constrains the compiler version (e.g. `"^0.3.9"`).
pub fn parse_input_file(input_file: &str, vyper_ver: Option<&str>) -> Result<SourceUnit> {
    println!("Compiling Vyper input file: {input_file}");
    let input_path = Path::new(input_file);
    if !input_path.exists() {
        fail!("Input file does not exist: {input_file}");
    }

    // Read source code to check version pragma
    let source_code = std::fs::read_to_string(input_file)
        .map_err(|e| common::error::create_error(format!("Failed to read input file: {e}")))?;

    let pragma = extract_version_pragma(&source_code);
    if let Some(ref ver) = pragma {
        log::debug!("Detected Vyper version pragma: {ver}");

        // Vyper B — validate minimum version
        if let Ok(pragma_range) = Range::parse(ver) {
            if !check_range_constraint(&pragma_range, ">=0.2.0") {
                fail!("Only Vyper versions >= 0.2.0 are supported, but found pragma: {ver}");
            }
        }
    }

    // Resolve compiler version
    resolve_and_configure_compiler(&pragma, vyper_ver)?;

    // Invoke vyper compiler to get JSON AST
    let json_ast = invoke_vyper_compiler(input_file)?;

    // Parse the JSON AST
    AstParser::parse(&json_ast, input_file)
}

/// Parse Vyper source code (string) into a Vyper AST source unit.
///
/// `vyper_ver` optionally constrains the compiler version (e.g. `"^0.3.9"`).
pub fn parse_vyper_source_code(source_code: &str, vyper_ver: Option<&str>) -> Result<SourceUnit> {
    // Write source code to a temporary file
    let temp_dir = tempfile::tempdir()
        .map_err(|e| common::error::create_error(format!("Failed to create temp dir: {e}")))?;
    let temp_file = temp_dir.path().join("temp_contract.vy");
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| common::error::create_error(format!("Failed to create temp file: {e}")))?;
    file.write_all(source_code.as_bytes())
        .map_err(|e| common::error::create_error(format!("Failed to write temp file: {e}")))?;

    let temp_path = temp_file.to_string_lossy().to_string();

    // Check pragma in source
    let pragma = extract_version_pragma(source_code);

    // Resolve compiler version
    resolve_and_configure_compiler(&pragma, vyper_ver)?;

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

// ============================================================================
// Internal helpers
// ============================================================================

/// Resolve the best Vyper version from pragma + optional CLI override, then
/// configure `vyper-select` to use it.
fn resolve_and_configure_compiler(pragma: &Option<String>, vyper_ver: Option<&str>) -> Result<()> {
    // Build the effective pragma range (intersection of pragma and CLI
    // override).
    let effective_pragma = match (pragma, vyper_ver) {
        (Some(p), Some(cli)) => Some(format!("{p} {cli}")),
        (Some(p), None) => Some(p.clone()),
        (None, Some(cli)) => Some(cli.to_string()),
        (None, None) => None,
    };

    let candidates = find_compatible_vyper_versions(&effective_pragma)?;

    if let Some(best) = candidates.first() {
        configure_vyper_compiler(best)?;
    }

    Ok(())
}

/// Invoke the Vyper compiler and return the JSON AST.
fn invoke_vyper_compiler(input_file: &str) -> Result<String> {
    log::debug!("Running: {VYPER} -f ast {input_file}");

    let output = Command::new(VYPER)
        .args(["-f", "ast", input_file])
        .output()
        .map_err(|e| {
            common::error::create_error(format!(
                "Failed to run `{VYPER}`. Is Vyper installed? Error: {e}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        fail!("Vyper compilation failed for {input_file}:\n{stderr}");
    }

    let json_str = String::from_utf8(output.stdout)
        .map_err(|e| common::error::create_error(format!("Invalid UTF-8 in vyper output: {e}")))?;

    if json_str.trim().is_empty() {
        fail!("Vyper compiler returned empty output for {input_file}");
    }

    Ok(json_str)
}

/// Extract the version pragma from Vyper source code.
///
/// Recognises both `# @version …` (Vyper ≤ 0.3.x) and `# pragma version …`
/// (Vyper ≥ 0.4.0).
pub fn extract_version_pragma(source: &str) -> Option<String> {
    let re = Regex::new(r"#\s*(?:@version|pragma\s+version)\s+(.+)").ok()?;
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
    fn test_extract_version_pragma_new_style() {
        let source = "# pragma version ^0.4.0\n\nowner: address";
        let version = extract_version_pragma(source);
        assert_eq!(version, Some("^0.4.0".to_string()));
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

    #[test]
    fn test_check_range_constraint() {
        let range = Range::parse("^0.3.9").unwrap();
        assert!(check_range_constraint(&range, ">=0.2.0"));

        let old_range = Range::parse("^0.1.0").unwrap();
        assert!(!check_range_constraint(&old_range, ">=0.2.0"));
    }
}
