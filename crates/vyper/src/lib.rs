//! Vyper smart contract compiler: parses `.vy` sources into SCIR.

pub mod ast;
pub mod irgen;
pub mod parser;

pub use scir;

use extlib::error::Result;

/// Compile a Vyper source file into a SCIR module.
pub fn compile_file(input_file: &str) -> Result<scir::Module> {
    let source_unit = parser::parse_input_file(input_file)?;
    let normalized = ast::normalize::run_passes(&source_unit);
    irgen::lower_source_unit(&normalized)
}

/// Compile Vyper source code (string) into a SCIR module.
pub fn compile_source(source_code: &str) -> Result<scir::Module> {
    let source_unit = parser::parse_vyper_source_code(source_code)?;
    let normalized = ast::normalize::run_passes(&source_unit);
    irgen::lower_source_unit(&normalized)
}

/// Parse a Vyper source file into the internal AST (without SCIR lowering).
pub fn parse_file(input_file: &str) -> Result<ast::SourceUnit> {
    parser::parse_input_file(input_file)
}

/// Parse Vyper source from a JSON AST string.
pub fn parse_json(json_str: &str, file_path: &str) -> Result<ast::SourceUnit> {
    parser::parse_from_json(json_str, file_path)
}
