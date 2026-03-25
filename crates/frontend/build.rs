//! Build script: auto-generate per-file compilation tests for Solidity
//! datasets.
//!
//! This scans two dataset directories at build time and emits `.rs` files
//! into `OUT_DIR`, each containing one `#[test]` function per `.sol` file.
//! New `.sol` files are picked up automatically on rebuild.

use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

/// Datasets relative to the workspace root.
const LIBSOLIDITY_DIR: &str = "datasets/solidity/libsolidity";
const SMARTBUGS_DIR: &str = "datasets/solidity/smartbugs-curated";

/// Directories inside a file's parent that should be ignored (artifacts from
/// previous test runs).
const IGNORED_SUBDIRS: &[&str] = &["preprocessed", "parsed", "normalized", "externalSource"];

fn main() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ parent")
        .parent()
        .expect("workspace root")
        .to_path_buf();

    let libsolidity_root = workspace_root.join(LIBSOLIDITY_DIR);
    let smartbugs_root = workspace_root.join(SMARTBUGS_DIR);

    // Tell Cargo to re-run this script when the dataset directories change.
    println!("cargo:rerun-if-changed={}", libsolidity_root.display());
    println!("cargo:rerun-if-changed={}", smartbugs_root.display());

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // -- libsolidity tests ---------------------------------------------------
    if libsolidity_root.exists() {
        let sol_files = collect_sol_files(&libsolidity_root);
        let code = generate_libsolidity_tests(&sol_files, &libsolidity_root, &workspace_root);
        fs::write(out_dir.join("compile_libsolidity_tests.rs"), code)
            .expect("write libsolidity tests");
    } else {
        fs::write(out_dir.join("compile_libsolidity_tests.rs"), "// dataset not found\n")
            .expect("write libsolidity stub");
    }

    // -- smartbugs-curated tests ---------------------------------------------
    if smartbugs_root.exists() {
        let sol_files = collect_sol_files(&smartbugs_root);
        let code = generate_smartbugs_tests(&sol_files, &smartbugs_root, &workspace_root);
        fs::write(out_dir.join("compile_smartbugs_tests.rs"), code)
            .expect("write smartbugs tests");
    } else {
        fs::write(out_dir.join("compile_smartbugs_tests.rs"), "// dataset not found\n")
            .expect("write smartbugs stub");
    }
}

// ─── File collection ─────────────────────────────────────────────────────────

/// Recursively collect `.sol` files, skipping ignored subdirectories.
fn collect_sol_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_sol_files_rec(root, root, &mut files);
    files.sort();
    files
}

fn collect_sol_files_rec(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if IGNORED_SUBDIRS.contains(&dir_name) || dir_name == "logs" {
                continue;
            }
            collect_sol_files_rec(root, &path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("sol") {
            out.push(path);
        }
    }
}

// ─── Test generation for libsolidity ─────────────────────────────────────────

/// Check whether a libsolidity test file is expected to have errors (and should
/// therefore be skipped).  The check mirrors `check_test_file_validity` in the
/// old test_utils.
fn is_error_test(path: &Path) -> bool {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return true,
    };
    for line in io::BufReader::new(file).lines().map_while(Result::ok) {
        if !line.starts_with("//") {
            continue;
        }
        if line.contains("SyntaxError")
            || line.contains("ParserError")
            || line.contains("DeclarationError")
            || line.contains("DocstringParsingError")
            || line.contains("failAfter: Parsed")
            || line.contains("TypeError")
            || line.contains("// Warning")
            || line.contains("// Info")
        {
            return true;
        }
    }
    false
}

/// Files that should be excluded from auto-generated tests because they
/// contain constructs our tooling cannot handle (external source directives,
/// unresolvable imports, or known-invalid Solidity).
const EXCLUDED_FILE_NAMES: &[&str] = &[
    "event_with_variables_of_internal_types.sol",
    // Category E: Multi-file import — requires cross-file compilation support
    "PRBMathSD59x18.sol",
    "PRBMathUD60x18.sol",
    // Category E: Import alias / module path references (T.S.E, M.C)
    "via_import.sol",
    "member_notation_ctor.sol",
    // Category F: Duplicate free constants after import elimination
    "same_constants_different_files.sol",
    // Category G: Function type in conditional expression edge case
    "conditional_with_arguments.sol",
    // Multi-source tests — require cross-file import resolution
    "circular_import_2.sol",
    "circular_reimport.sol",
    "circular_reimport_2.sol",
    "free_different_interger_types.sol",
    "free_function_transitive_import.sol",
    "import_overloaded_function.sol",
    "imported_free_function_via_alias.sol",
    "reimport_imported_function.sol",
    // Module path / import alias tests — require module resolution support
    "library_address_via_module.sol",
    "access_through_module_name.sol",
    "multisource.sol",
    "multisource_module.sol",
    "imported_functions.sol",
    "library_through_module.sol",
    "module_renamed.sol",
    "recursive_import.sol",
    "using_global_all_the_types.sol",
    "using_global_for_global.sol",
    "using_global_invisible.sol",
    "using_global_library.sol",
];

/// Check whether a test file should be excluded from test generation.
fn should_exclude_test(path: &Path) -> bool {
    // Check excluded file names
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if EXCLUDED_FILE_NAMES.contains(&name) {
            return true;
        }
    }

    // Exclude SMT-checker tests entirely — they use experimental pragmas and
    // SMT-specific output expectations that our tooling cannot handle.
    if path.to_string_lossy().contains("smt-checker-tests") {
        return true;
    }

    // Check file content for ExternalSource directives or external imports
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return true,
    };

    for line in content.lines() {
        // Files using ==== ExternalSource: ... ==== directives
        if line.contains("==== ExternalSource:") {
            return true;
        }
        // Files importing from external directories or URLs
        if line.starts_with("import") && (line.contains("http://") || line.contains("https://")) {
            return true;
        }
    }

    // Exclude tests relying on `using X for *` until eliminate_using is complete.
    if content.contains("using ") && content.contains("for *") {
        return true;
    }

    false
}

fn generate_libsolidity_tests(
    files: &[PathBuf],
    dataset_root: &Path,
    workspace_root: &Path,
) -> String {
    let mut buf = String::new();
    writeln!(buf, "// Auto-generated libsolidity compilation tests.").unwrap();
    writeln!(buf, "// DO NOT EDIT — generated by build.rs\n").unwrap();
    writeln!(buf, "#[allow(non_snake_case)]").unwrap();
    writeln!(buf, "use test_utils::test_compiling_solidity_file_standalone;\n").unwrap();

    // Group by top-level sub-dataset (e.g. "syntax-tests", "semantic-tests")
    let mut grouped: BTreeMap<String, Vec<(&PathBuf, String)>> = BTreeMap::new();
    for path in files {
        if is_error_test(path) || should_exclude_test(path) {
            continue;
        }
        let rel = path.strip_prefix(dataset_root).unwrap();
        let sub_dataset = rel
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("root")
            .to_string();
        let test_name = make_test_name(&sub_dataset, rel);
        grouped
            .entry(sub_dataset.clone())
            .or_default()
            .push((path, test_name));
    }

    for (sub_dataset, tests) in &grouped {
        let mod_name = sanitize_ident(sub_dataset);
        writeln!(buf, "#[allow(non_snake_case)]").unwrap();
        writeln!(buf, "mod {mod_name} {{").unwrap();
        writeln!(buf, "    use super::*;\n").unwrap();
        for (path, test_name) in tests {
            let abs = workspace_root
                .join(LIBSOLIDITY_DIR)
                .join(path.strip_prefix(dataset_root).unwrap());
            let abs_str = abs.display().to_string().replace('\\', "/");
            writeln!(buf, "    #[test]").unwrap();
            writeln!(buf, "    fn {test_name}() {{").unwrap();
            writeln!(
                buf,
                "        test_compiling_solidity_file_standalone(\"{abs_str}\", \"0.8.19\");"
            )
            .unwrap();
            writeln!(buf, "    }}\n").unwrap();
        }
        writeln!(buf, "}}\n").unwrap();
    }

    buf
}

// ─── Test generation for smartbugs-curated ───────────────────────────────────

fn generate_smartbugs_tests(
    files: &[PathBuf],
    dataset_root: &Path,
    workspace_root: &Path,
) -> String {
    let mut buf = String::new();
    writeln!(buf, "// Auto-generated smartbugs-curated compilation tests.").unwrap();
    writeln!(buf, "// DO NOT EDIT — generated by build.rs\n").unwrap();
    writeln!(buf, "#[allow(non_snake_case)]").unwrap();
    writeln!(buf, "use test_utils::test_compiling_solidity_file_standalone;\n").unwrap();

    // Group by category dir
    let mut grouped: BTreeMap<String, Vec<(&PathBuf, String)>> = BTreeMap::new();
    for path in files {
        let rel = path.strip_prefix(dataset_root).unwrap();
        let category = rel
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("root")
            .to_string();
        let test_name = make_test_name_simple(rel);
        grouped
            .entry(category.clone())
            .or_default()
            .push((path, test_name));
    }

    for (category, tests) in &grouped {
        let mod_name = sanitize_ident(category);
        writeln!(buf, "#[allow(non_snake_case)]").unwrap();
        writeln!(buf, "mod {mod_name} {{").unwrap();
        writeln!(buf, "    use super::*;\n").unwrap();
        for (path, test_name) in tests {
            let abs = workspace_root
                .join(SMARTBUGS_DIR)
                .join(path.strip_prefix(dataset_root).unwrap());
            let abs_str = abs.display().to_string().replace('\\', "/");
            // Smartbugs contracts are often old Solidity (<0.5)
            writeln!(buf, "    #[test]").unwrap();
            writeln!(buf, "    fn {test_name}() {{").unwrap();
            writeln!(
                buf,
                "        test_compiling_solidity_file_standalone(\"{abs_str}\", \"0.4.26\");"
            )
            .unwrap();
            writeln!(buf, "    }}\n").unwrap();
        }
        writeln!(buf, "}}\n").unwrap();
    }

    buf
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Turn a relative path like `abstract/foo_bar.sol` into a valid Rust test
/// identifier like `abstract__foo_bar`.
fn make_test_name(_sub_dataset: &str, rel: &Path) -> String {
    // Skip the first component (sub-dataset name) as it's used for the module.
    let rest: PathBuf = rel.components().skip(1).collect();
    let s = rest
        .to_str()
        .unwrap_or("unknown")
        .trim_end_matches(".sol")
        .replace(['/', '\\'], "__")
        .replace(
            [
                '-', '.', ' ', '(', ')', '+', '=', ',', '\'', '!', '@', '#', '$', '%', '^', '&',
                '*', '~', '`', '[', ']', '{', '}', '|', ':', ';', '<', '>', '?',
            ],
            "_",
        );
    sanitize_ident(&s)
}

fn make_test_name_simple(rel: &Path) -> String {
    let s = rel
        .to_str()
        .unwrap_or("unknown")
        .trim_end_matches(".sol")
        .replace(['/', '\\'], "__")
        .replace(
            [
                '-', '.', ' ', '(', ')', '+', '=', ',', '\'', '!', '@', '#', '$', '%', '^', '&',
                '*', '~', '`', '[', ']', '{', '}', '|', ':', ';', '<', '>', '?',
            ],
            "_",
        );
    // Skip the first segment (category) since it's used as module name
    let parts: Vec<&str> = s.splitn(2, "__").collect();
    if parts.len() > 1 {
        sanitize_ident(parts[1])
    } else {
        sanitize_ident(&s)
    }
}

/// Ensure the identifier is a valid Rust ident: starts with a letter or
/// underscore, contains only alphanumeric and underscores.
fn sanitize_ident(s: &str) -> String {
    let s = s.replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_");
    // Collapse repeated underscores
    let mut result = String::with_capacity(s.len());
    let mut prev_underscore = false;
    for c in s.chars() {
        if c == '_' {
            if !prev_underscore {
                result.push('_');
            }
            prev_underscore = true;
        } else {
            result.push(c);
            prev_underscore = false;
        }
    }
    // Remove trailing underscores
    let result = result.trim_end_matches('_').to_string();
    // Prefix with underscore if starts with digit
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{result}")
    } else if result.is_empty() {
        "_unnamed".to_string()
    } else {
        // Handle Rust keywords
        match result.as_str() {
            "abstract" | "as" | "async" | "await" | "become" | "box" | "break" | "const"
            | "continue" | "crate" | "do" | "dyn" | "else" | "enum" | "extern" | "false"
            | "final" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "macro"
            | "match" | "mod" | "move" | "mut" | "override" | "priv" | "pub" | "ref"
            | "return" | "self" | "Self" | "static" | "struct" | "super" | "trait" | "true"
            | "try" | "type" | "typeof" | "unsafe" | "unsized" | "use" | "virtual" | "where"
            | "while" | "yield" => format!("{result}_"),
            _ => result,
        }
    }
}
