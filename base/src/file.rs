//! Module containing utility functions to handle files.

use crate::{
    error::{create_error, report_error, Result},
    fail,
};
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
        Err(err) => return Err(create_error(err)),
    };
    let output_file_path = output_dir_path.join(file_name);
    let mut output_file = File::create(&output_file_path)?;
    match output_file.write_all(file_content.as_bytes()) {
        Ok(_) => match output_file_path.to_str() {
            Some(path) => Ok(path.to_string()),
            None => Err(create_error("Output file path not found!")),
        },
        Err(err) => Err(create_error(err)),
    }
}

/// Save multiple strings to multiple temporary files.
///
/// Return the output file path.
pub fn save_to_temporary_files(
    source_code_list: &[(&str, &str)],
) -> Result<Vec<String>> {
    let output_dir_path = match tempfile::tempdir() {
        Ok(dir) => dir.keep(),
        Err(err) => return Err(create_error(err)),
    };

    let mut output_files: Vec<String> = vec![];

    for file_name_content in source_code_list.iter() {
        let (file_name, file_content) = file_name_content;

        let output_file_path = output_dir_path.join(file_name);
        let mut output_file = File::create(&output_file_path)?;
        match output_file.write_all(file_content.as_bytes()) {
            Ok(_) => match output_file_path.to_str() {
                Some(path) => output_files.push(path.to_string()),
                None => {
                    return Err(create_error("Output file path not found!"))
                }
            },
            Err(err) => return Err(create_error(err)),
        }
    }

    Ok(output_files)
}
