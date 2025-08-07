use crate::{ast, parser::ast_parser, version::find_compatible_solc_versions};
use color_eyre::eyre::{Result, bail};

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
    let solc_ver = match solc_ver {
        Some(ver) => ver.to_string(),
        None => {
            let vers = find_compatible_solc_versions(input_file)?;
            if !vers.is_empty() {
                let ver = vers[0].to_string();
                debug!("Choosing the first compatible version: {ver}");
                ver
            } else {
                bail!("No compatible Solc versions found!")
            }
        }
    };
    ast_parser::parse_solidity_file(input_file, base_path, include_paths, &solc_ver)
}
