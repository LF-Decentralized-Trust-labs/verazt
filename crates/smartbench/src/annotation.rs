//! Ground-truth annotation parser for the SmartBugs dataset.
//!
//! Annotations in SmartBugs-curated dataset follow the format:
//! ```solidity
//! // <yes> <report> BUG_NAME
//! vulnerable_code_line;
//! ```

use bugs::bug::BugCategory;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// An annotated bug from the ground-truth dataset.
#[derive(Debug, Clone)]
pub struct AnnotatedBug {
    /// Path to the Solidity file containing the annotation.
    pub file_path: PathBuf,
    /// Line number of the annotation comment.
    pub line_number: usize,
    /// Line number of the vulnerable code (line_number + 1).
    pub bug_line: usize,
    /// Raw bug name from annotation (e.g., "REENTRANCY").
    pub bug_name: String,
    /// Parsed bug category.
    pub category: BugCategory,
}

/// Parse annotations from a single .sol file.
///
/// Looks for lines matching `// <yes> <report> BUG_NAME` and creates
/// `AnnotatedBug` entries for each match.
pub fn parse_annotations(file_path: &Path) -> Vec<AnnotatedBug> {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let re = Regex::new(r"//\s*<yes>\s*<report>\s+(.+)").unwrap();
    let mut annotations = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        if let Some(caps) = re.captures(line) {
            let bug_name = caps[1].trim().to_string();
            let line_number = idx + 1; // 1-indexed

            if let Some(category) = BugCategory::from_annotation(&bug_name) {
                annotations.push(AnnotatedBug {
                    file_path: file_path.to_path_buf(),
                    line_number,
                    bug_line: line_number + 1,
                    bug_name,
                    category,
                });
            }
        }
    }

    annotations
}

/// Recursively scan a dataset directory for .sol files and parse annotations.
pub fn scan_dataset(dataset_dir: &Path) -> Vec<AnnotatedBug> {
    let mut all_annotations = Vec::new();

    for entry in WalkDir::new(dataset_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sol") {
            let annotations = parse_annotations(path);
            all_annotations.extend(annotations);
        }
    }

    all_annotations
}

/// Get all .sol files in a dataset directory.
pub fn collect_sol_files(dataset_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dataset_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sol") {
            files.push(path.to_path_buf());
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_annotations_simple() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"pragma solidity ^0.4.2;
contract SimpleDAO {{
  function withdraw(uint amount) {{
    if (credit[msg.sender]>= amount) {{
      // <yes> <report> REENTRANCY
      bool res = msg.sender.call.value(amount)();
    }}
  }}
}}"#
        )
        .unwrap();

        let annotations = parse_annotations(file.path());
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].bug_name, "REENTRANCY");
        assert_eq!(annotations[0].category, BugCategory::Reentrancy);
        assert_eq!(annotations[0].line_number, 5);
        assert_eq!(annotations[0].bug_line, 6);
    }

    #[test]
    fn test_parse_annotations_multiple() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"// <yes> <report> ACCESS_CONTROL
line1;
// <yes> <report> ACCESS_CONTROL
line2;"#
        )
        .unwrap();

        let annotations = parse_annotations(file.path());
        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].category, BugCategory::AccessControl);
        assert_eq!(annotations[1].category, BugCategory::AccessControl);
    }

    #[test]
    fn test_parse_annotations_other_style() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"// <yes> <report> OTHER - uninitialized storage
line1;"#
        )
        .unwrap();

        let annotations = parse_annotations(file.path());
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].category, BugCategory::Other);
    }

    #[test]
    fn test_parse_annotations_unchecked_ll_calls() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"        // <yes> <report> UNCHECKED_LL_CALLS
        line1;"#
        )
        .unwrap();

        let annotations = parse_annotations(file.path());
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].category, BugCategory::UncheckedLowLevelCalls);
    }

    #[test]
    fn test_parse_annotations_no_match() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"pragma solidity ^0.4.2;
contract Foo {{
  function bar() {{}}
}}"#
        )
        .unwrap();

        let annotations = parse_annotations(file.path());
        assert!(annotations.is_empty());
    }

    #[test]
    fn test_scan_dataset() {
        let dir = tempfile::tempdir().unwrap();

        // Create a .sol file with annotations
        let sol_path = dir.path().join("test.sol");
        fs::write(&sol_path, "// <yes> <report> REENTRANCY\nvulnerable_line;\n").unwrap();

        // Create a non-sol file (should be ignored)
        let txt_path = dir.path().join("readme.txt");
        fs::write(&txt_path, "// <yes> <report> REENTRANCY\n").unwrap();

        let annotations = scan_dataset(dir.path());
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].category, BugCategory::Reentrancy);
    }

    #[test]
    fn test_collect_sol_files() {
        let dir = tempfile::tempdir().unwrap();

        fs::write(dir.path().join("a.sol"), "// sol").unwrap();
        fs::write(dir.path().join("b.sol"), "// sol").unwrap();
        fs::write(dir.path().join("c.txt"), "// txt").unwrap();

        let files = collect_sol_files(dir.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_parse_all_annotation_types() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"// <yes> <report> REENTRANCY
line;
// <yes> <report> ARITHMETIC
line;
// <yes> <report> ACCESS_CONTROL
line;
// <yes> <report> UNCHECKED_LL_CALLS
line;
// <yes> <report> DENIAL_OF_SERVICE
line;
// <yes> <report> BAD_RANDOMNESS
line;
// <yes> <report> FRONT_RUNNING
line;
// <yes> <report> TIME_MANIPULATION
line;
// <yes> <report> SHORT_ADDRESSES
line;"#
        )
        .unwrap();

        let annotations = parse_annotations(file.path());
        assert_eq!(annotations.len(), 9);

        assert_eq!(annotations[0].category, BugCategory::Reentrancy);
        assert_eq!(annotations[1].category, BugCategory::Arithmetic);
        assert_eq!(annotations[2].category, BugCategory::AccessControl);
        assert_eq!(annotations[3].category, BugCategory::UncheckedLowLevelCalls);
        assert_eq!(annotations[4].category, BugCategory::DenialOfService);
        assert_eq!(annotations[5].category, BugCategory::BadRandomness);
        assert_eq!(annotations[6].category, BugCategory::FrontRunning);
        assert_eq!(annotations[7].category, BugCategory::TimeManipulation);
        assert_eq!(annotations[8].category, BugCategory::ShortAddresses);
    }
}
