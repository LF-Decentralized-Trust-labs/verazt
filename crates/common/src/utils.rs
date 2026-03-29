use std::path::Path;

/// Print a formatted section header to stdout.
pub fn print_header(title: &str) {
    let ruler = "=".repeat(75);
    println!();
    println!("{ruler}");
    println!("*** {title} ***");
    println!("{ruler}");
    println!();
}

/// Print a formatted subsection header to stdout.
pub fn print_subheader(title: &str) {
    let ruler = "-".repeat(51);
    println!();
    println!("{ruler}");
    println!("*** {title} ***");
    println!("{ruler}");
    println!();
}

/// Format a path to be relative to the current working directory, if possible.
pub fn format_relative_path(path: &Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(stripped) = path.strip_prefix(&cwd) {
            return format!("{}", stripped.display());
        }
    }
    format!("{}", path.display())
}
