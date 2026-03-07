//! Vyper smart contract compiler: parses `.vy` sources into SCIR.

pub mod ast;
pub mod irgen;
pub mod parser;

pub use scir;

use common::error::Result;

/// Compile a Vyper source file into a SCIR module.
///
/// `vyper_ver` optionally constrains the compiler version (e.g. `"^0.3.9"`).
pub fn compile_file(input_file: &str, vyper_ver: Option<&str>) -> Result<scir::Module> {
    let source_unit = parser::parse_input_file(input_file, vyper_ver)?;
    let normalized = ast::normalize::run_passes(&source_unit);
    irgen::lower_source_unit(&normalized)
}

/// Compile Vyper source code (string) into a SCIR module.
///
/// `vyper_ver` optionally constrains the compiler version (e.g. `"^0.3.9"`).
pub fn compile_source(source_code: &str, vyper_ver: Option<&str>) -> Result<scir::Module> {
    let source_unit = parser::parse_vyper_source_code(source_code, vyper_ver)?;
    let normalized = ast::normalize::run_passes(&source_unit);
    irgen::lower_source_unit(&normalized)
}

/// Parse a Vyper source file into the internal AST (without SCIR lowering).
pub fn parse_file(input_file: &str, vyper_ver: Option<&str>) -> Result<ast::SourceUnit> {
    parser::parse_input_file(input_file, vyper_ver)
}

/// Parse Vyper source from a JSON AST string.
pub fn parse_json(json_str: &str, file_path: &str) -> Result<ast::SourceUnit> {
    parser::parse_from_json(json_str, file_path)
}

/// Extract the version pragma string from a `.vy` file (e.g. `"^0.3.9"`).
///
/// Reads the file and returns the pragma if found, or `None` if absent.
pub fn extract_pragma(file: &str) -> Result<Option<String>> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| common::error::create_error(format!("Failed to read {file}: {e}")))?;
    Ok(parser::extract_version_pragma(&source))
}

/// Query PyPI and return installable Vyper versions satisfying `pragma`.
///
/// Returns versions sorted newest-first.
pub fn find_installable_versions(pragma: &str) -> Result<Vec<node_semver::Version>> {
    parser::find_installable_vyper_versions(pragma)
}

/// Install and activate a specific Vyper version via `vyper-select`.
pub fn install_version(ver: &node_semver::Version) -> Result<()> {
    parser::configure_vyper_compiler(ver)
}
