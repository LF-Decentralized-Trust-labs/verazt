//! Module containing utility functions to handle files.

use color_eyre::eyre::{Result, bail};
use std::{fs::File, io::Write};

/// Save a string to a temporary file of a given name.
///
/// Return the output file path.
pub fn save_to_temporary_file(
    file_content: &str,
    file_name: &str,
) -> Result<String> {
    let output_dir_path = match tempfile::tempdir() {
        Ok(dir) => dir.keep(),
        Err(err) => bail!(err),
    };
    let output_file_path = output_dir_path.join(file_name);
    let mut output_file = File::create(&output_file_path)?;
    match output_file.write_all(file_content.as_bytes()) {
        Ok(_) => match output_file_path.to_str() {
            Some(path) => Ok(path.to_string()),
            None => bail!("Output file path not found!"),
        },
        Err(err) => bail!(err),
    }
}

/// Save multiple strings to multiple temporary files.
///
/// Return the output file path.
pub fn save_to_temporary_files(
    file_names_and_contents: &[(&str, &str)],
) -> Result<Vec<String>> {
    let output_dir_path = match tempfile::tempdir() {
        Ok(dir) => dir.keep(),
        Err(err) => bail!(err),
    };

    let mut output_files: Vec<String> = vec![];

    for file_name_content in file_names_and_contents.iter() {
        let (file_name, file_content) = file_name_content;

        let output_file_path = output_dir_path.join(file_name);
        let mut output_file = File::create(&output_file_path)?;
        match output_file.write_all(file_content.as_bytes()) {
            Ok(_) => match output_file_path.to_str() {
                Some(path) => output_files.push(path.to_string()),
                None => bail!("Output file path not found!"),
            },
            Err(err) => bail!(err),
        }
    }

    Ok(output_files)
}
