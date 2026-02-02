//! Module for exporting Solidity AST.

use crate::ast::*;
use color_eyre::eyre::{ContextCompat, Result};
use extlib::fail;
use std::{
    ffi::OsStr,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

/// Log directory
pub static LOGS_DIR: &str = "logs";

/// Export multiple Solidity AST source units to file.
pub fn export_source_units(source_units: &[SourceUnit], output_file: &str) -> Result<()> {
    let output_file_path = Path::new(output_file);

    // Create parent directory if it does not exists
    if let Some(parent_dir) = output_file_path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let mut file = match File::create(output_file_path) {
        Ok(file) => file,
        Err(_) => {
            panic!("Failed to export Solidity AST to file: {output_file}")
        }
    };

    let source_unit_strs: Vec<String> = source_units
        .iter()
        .map(|source_unit| format!("{source_unit}"))
        .collect();

    Ok(file.write_all(source_unit_strs.join("\n\n").as_bytes())?)
}

/// Prepare log directory for a source file.
pub fn prepare_logging_directory(input_file: &str) -> Result<(PathBuf, String)> {
    let file_path = Path::new(input_file);
    let file_name = file_path.file_name().context(format!(
        "Failed to prepare logging directory: file name not found: {input_file}"
    ))?;
    let file_dir = file_path.parent().context(format!(
        "Failed to prepare logging directory: parent directory not found for {input_file}"
    ))?;

    let output_dir = file_dir.join(LOGS_DIR).join(file_name);
    if !output_dir.exists() && std::fs::create_dir_all(&output_dir).is_err() {
        fail!("Failed to create output directory: {}", output_dir.display());
    }
    // println!("Output directory: {}", output_dir.display());

    let file_stem = Path::new(&file_name)
        .file_stem()
        .and_then(OsStr::to_str)
        .context("Skip exporting source unit: file stem not found!")?;

    Ok((output_dir, file_stem.to_string()))
}

/// Export a source unit in AST form to an output file.
pub fn export_source_unit(source_unit: &SourceUnit, output_file: &str) -> Result<()> {
    export_source_units(&[source_unit.clone()], output_file)
}

/// Export a source unit in AST form for debugging purpose.
pub fn export_debugging_source_unit(
    source_unit: &SourceUnit,
    source_unit_type: &str,
) -> Result<()> {
    let file_path: String = source_unit.path.clone();
    let (output_dir, file_stem) = prepare_logging_directory(&file_path)?;

    let output_file = &output_dir.join(format!("{}.{}.sol", file_stem, source_unit_type));
    let output_file = output_file
        .to_str()
        .context("Skip exporting source unit: output file not found!")?;

    export_source_units(&[source_unit.clone()], output_file)
}
