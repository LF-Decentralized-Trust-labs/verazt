use crate::ir::*;
use color_eyre::eyre::Result;
use std::{fs::File, io::Write, path::Path};

pub fn export_source_units(source_units: &[SourceUnit], output_file: &str) -> Result<()> {
    let output_file_path = Path::new(output_file);
    let mut file = match File::create(output_file_path) {
        Ok(file) => file,
        Err(_) => {
            panic!("Failed to export Solidity IR to file: {output_file}")
        }
    };

    let source_unit_strs: Vec<String> = source_units
        .iter()
        .map(|source_unit| format!("{source_unit}"))
        .collect();

    Ok(file.write_all(source_unit_strs.join("\n\n").as_bytes())?)
}

/// Export a source unit in IR form for debugging purpose.
pub fn export_source_unit(source_unit: &SourceUnit, output_file: &str) -> Result<()> {
    export_source_units(&[source_unit.clone()], output_file)
}

/// Export a source unit in IR form for debugging purpose.
pub fn export_debugging_source_unit(
    _source_unit: &SourceUnit,
    _source_unit_type: &str,
) -> Result<()> {
    // let file_path: String = source_unit.path.clone();
    // let (output_dir, file_stem) =
    // ast::export::prepare_logging_directory(&file_path)?; let output_path =
    // &output_dir.join(format!("{}.{}.sol", file_stem, source_unit_type));
    // let output_file = output_path
    //     .to_str()
    //     .context("Skip exporting source unit: output file not found!")?;

    // debug!("Export {} source unit to: {}", source_unit_type, output_file);

    // export_source_units(&[source_unit.clone()], output_file)

    todo!("Export IR");
}
