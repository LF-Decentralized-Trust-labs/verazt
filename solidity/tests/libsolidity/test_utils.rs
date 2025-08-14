//! Module containing utility functions for unit test.
use extlib::{error::Result, fail};
use regex::Regex;
use solidity::{
    ast::SourceUnit, compile::compile_input_file, normalize, util::export::export_source_unit,
};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

// Hack to print debugging information by `info!` when running tests.
#[cfg(test)]
#[allow(unused_imports)]
use std::{println as info, println as warn};

/// Global variable to control whether to test parsing ASTs by exporting the
/// ASTs to Solidity files and invoke Solc to compile them.
pub const TEST_PARSING_AST: bool = true;

/// Global variable to control whether to test normalizing ASTs by exporting
/// the ASTs to Solidity files and invoke Solc to compile them.
pub const TEST_NORMALIZING_AST: bool = true;

/// Export source file.
fn export_source_file(
    source_path: &str,
    content_lines: &[String],
    output_dir: &str,
) -> Option<String> {
    let output_file_path = Path::new(output_dir).join(source_path);
    if let Some(output_dir) = output_file_path.parent() {
        std::fs::create_dir_all(output_dir).ok();
    }
    let mut output_file = match File::create(&output_file_path) {
        Ok(file) => file,
        Err(_) => panic!("Unable to create file: {}", output_file_path.display()),
    };
    output_file
        .write_all(content_lines.join("\n").as_bytes())
        .ok();

    output_file_path.to_str().map(|s| s.to_string())
}

/// Preprocess a Solidity file collected from Ethereum's Solidity test-suite.
fn preprocess_solidity_file(file_path: PathBuf) -> Result<(String, Vec<String>)> {
    info!("- Preprocess: {}", file_path.display());

    // Split content of the test file into lines to extract embedded source files``
    let input_file = File::open(&file_path)?;
    let file_content_lines = io::BufReader::new(input_file).lines();
    let test_dir = file_path.parent().unwrap_or_else(|| Path::new(""));
    let output_dir = test_dir
        .join("preprocessed")
        .to_str()
        .unwrap_or("")
        .to_string();
    let mut source_name: String = file_path
        .file_name()
        .expect("Test file name should be provided")
        .to_str()
        .expect("Test file name should be provided")
        .to_string();

    let mut content: Vec<String> = vec![];
    let mut output_files: Vec<String> = vec![];

    let source_name_regex = Regex::new(r"==== Source: (.+) ====")?;
    for line in file_content_lines.map_while(Result::ok) {
        // Detect a new source file
        if let Some(captures) = source_name_regex.captures(&line) {
            // Write content of the current source file first
            if !content.is_empty()
                && let Some(out_file) = export_source_file(&source_name, &content, &output_dir)
            {
                output_files.push(out_file);
            }

            // Start to record content of the new source file.
            content = vec![];
            source_name = captures.get(1).map_or("", |m| m.as_str()).to_string();
        } else {
            content.push(line.clone());
        }
    }

    // Write content of the last source file
    if !content.is_empty()
        && let Some(output_file) = export_source_file(&source_name, &content, &output_dir)
    {
        output_files.push(output_file);
    }

    Ok((output_dir, output_files))
}

/// Check whether the test file is a type error Solidity file.
fn check_test_file_validity(file_path: &PathBuf) -> Result<()> {
    let input_file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Ok(()),
    };
    let input_file_lines = io::BufReader::new(input_file).lines();
    for line in input_file_lines.map_while(Result::ok) {
        if !line.starts_with("//") {
            continue;
        }

        if line.contains("SyntaxError") {
            fail!("Syntax Error");
        } else if line.contains("ParserError") {
            fail!("Parser Error");
        } else if line.contains("DeclarationError") {
            fail!("Declaration Error");
        } else if line.contains("DocstringParsingError") {
            fail!("Docstring Parsing Error");
        } else if line.contains("failAfter: Parsed") {
            fail!("Failed After Parsing");
        } else if line.contains("TypeError") {
            fail!("Type Error");
        }
    }
    Ok(())
}

/// Get names of all `.sol` files of a directory.
///
/// This function only returns the file name, but not the path.
fn get_solidity_test_files(dir: &str) -> Vec<String> {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(_) => panic!("Failed to read directory: {dir}"),
    };
    read_dir
        .into_iter()
        .filter_map(|item| match item {
            Ok(item) => match item.file_name().to_str() {
                Some(s) => match Path::new(s).extension().and_then(OsStr::to_str) {
                    Some("sol") => Some(s.to_string()),
                    _ => None,
                },
                None => None,
            },
            Err(_) => None,
        })
        .collect()
}

/// Test compiling a list of related Solidity files which can import each other.
/// The test is done by performing the following steps:
///
/// 1. Compile each file by Solc to JSON AST and parse the JSON AST to internal
///    AST.
/// 2. Export all the internal ASTs back to Solidity files.
/// 3. Compile the exported Solidity files by Solc and check if it is
///    successful.
fn test_compiling_solidity_file(
    input_files: &[String],
    preprocessed_dir: &str,
    root_test_dir: &Path,
    solc_ver: &str,
) -> bool {
    let mut parsed_source_units: Vec<SourceUnit> = vec![];
    for input_file in input_files {
        match compile_input_file(input_file, Some(preprocessed_dir), &[], Some(solc_ver)) {
            Ok(source_units) => {
                // Check if source units are compiled successfully.
                assert!(
                    !source_units.is_empty(),
                    "Test export and compile source unit: no source unit is: {input_file}"
                );

                // Do not include duplicated source unit.
                for source_unit in source_units {
                    if parsed_source_units
                        .iter()
                        .all(|sunit| !&sunit.path.eq(&source_unit.path))
                    {
                        parsed_source_units.push(source_unit)
                    }
                }
            }
            Err(err) => panic!("Failed to compile: {input_file}\n\nError: {err}"),
        };
    }

    info!("- Test dir: {root_test_dir:?}");
    info!("- Preprocessed dir: {preprocessed_dir:?}");

    // Test compiling all parsed source units
    if TEST_PARSING_AST {
        info!("\n================== TEST PARSING AST ==================");
        let parsed_dir_path: PathBuf = root_test_dir.join("parsed");
        let parsed_dir = parsed_dir_path.to_str().unwrap_or("");

        let mut exported_files = vec![];
        for source_unit in parsed_source_units.iter() {
            // Prepare output file name
            info!("- Source file: {}", source_unit.path);
            let output_file = source_unit.path.replace(preprocessed_dir, parsed_dir);

            info!("- Export the parsed AST to: {output_file}");
            export_source_unit(source_unit, &output_file).ok();

            assert!(
                Path::new(&output_file).exists(),
                "Soljc failed export the original AST to: {output_file}"
            );
            exported_files.push(output_file);
        }

        // Now compile all the exported files to test if they are valid Solidity files.
        for file in exported_files {
            info!("- Test compilation: {file}");
            if let Err(err) = compile_input_file(&file, Some(parsed_dir), &[], Some(solc_ver)) {
                panic!("Failed to compile: {file}\n\nError: {err}");
            }
        }
    }

    // Test compiling all normalized source units
    if TEST_NORMALIZING_AST {
        info!("\n================== TEST NORMALIZING AST ==================");
        let normalized_dir_path: PathBuf = root_test_dir.join("normalized");
        let normalized_dir = normalized_dir_path.to_str().unwrap_or("");

        let normalized_source_units = normalize::normalize_source_units(&parsed_source_units);

        // Test compiling all normalized source units
        let mut exported_files = vec![];
        for source_unit in normalized_source_units.iter() {
            // Prepare output file name
            let output_file = source_unit.path.replace(preprocessed_dir, normalized_dir);

            info!("- Export normalized AST to: {output_file}");
            export_source_unit(source_unit, &output_file).ok();

            assert!(
                Path::new(&output_file).exists(),
                "Soljc failed export the normalized AST to: {output_file}"
            );

            exported_files.push(output_file);
        }

        for file in exported_files {
            // Compile the normalized Solidity file by Solc again
            info!("- Test compilation: {file}");
            if let Err(err) = compile_input_file(&file, Some(normalized_dir), &[], Some(solc_ver))
            {
                panic!("Failed to compile: {file}\n\nError: {err}");
            }
        }
    }

    true
}

/// Test parsing Solidity files in a directory
///
/// `failed_tests`: list of test cases that are supposed to be failed
#[allow(dead_code)]
pub fn test_compiling_solidity_dir(input_dir: &str, skipped_tests: Vec<&str>, solc_version: &str) {
    // Use `color_eyre` to print error messages in a more readable way.
    let _ = color_eyre::install();

    // Create a temporarily directory as the output test directory
    let output_path = match tempfile::tempdir() {
        Ok(temp_dir) => temp_dir.keep(),
        Err(_) => panic!("Failed to create the output test directory!"),
    };
    let root_output_dir = match output_path.as_os_str().to_str() {
        Some(dir) => dir,
        None => panic!("Failed to create the output test directory!"),
    };

    let mut successful_tests = vec![];
    for test_file in get_solidity_test_files(input_dir) {
        let test_file_path = Path::new(input_dir).join(Path::new(&test_file));
        info!("Test file: {}\n", &test_file_path.display());

        if let Err(err) = check_test_file_validity(&test_file_path) {
            println!("Ignore error test file: {err}");
            continue;
        }

        if skipped_tests.contains(&test_file.as_str()) {
            println!("Skipping test! To be handle later!");
            continue;
        }

        // Create a dedicated output directory for each test file.
        let root_test_dir = match Path::new(&test_file).file_name() {
            Some(file_name) => Path::new(root_output_dir).join(file_name),
            None => Path::new(root_output_dir).to_path_buf(),
        };

        if !root_test_dir.exists() && std::fs::create_dir_all(&root_test_dir).is_err() {
            panic!("Failed to create output test directory: {}", root_test_dir.display());
        }

        // Copy files to the temporary directory
        let input_test_file = root_test_dir.join(Path::new(&test_file));
        match fs::copy(&test_file_path, &input_test_file) {
            Ok(_) => (),
            Err(err) => panic!(
                "Failed to copy test file `{:?}` to `{:?}`.\nError: {}",
                &test_file_path, &input_test_file, err
            ),
        }

        // Test files in the temporary directory
        if let Ok((preprocessed_dir, test_files)) = preprocess_solidity_file(input_test_file) {
            info!("- Start to compile: {}", test_files.join(", "));
            if !test_compiling_solidity_file(
                &test_files,
                &preprocessed_dir,
                &root_test_dir,
                solc_version,
            ) {
                successful_tests.push(test_file);
            }
        }
    }

    successful_tests.dedup();
    println!("Successful tests: {successful_tests:?}");
}
