//! Module export utilities — writing CIR modules to disk.

use crate::Module;
use color_eyre::eyre::Result;
use std::fs;
use std::path::Path;

/// Export a CIR module to a file.
pub fn export_module(module: &Module, output_dir: &Path) -> Result<String> {
    let content = format!("{module}");
    let name = module.name().unwrap_or_else(|| "module".to_string());
    let filename = format!("{name}.cir");
    let path = output_dir.join(&filename);
    fs::create_dir_all(output_dir)?;
    fs::write(&path, &content)?;
    Ok(filename)
}
