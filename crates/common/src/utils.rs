/// Print a formatted section header to stdout.
pub fn print_section_header(title: &str) {
    let ruler = "=".repeat(75);
    println!();
    println!("{ruler}");
    println!("*** {title} ***");
    println!("{ruler}");
    println!();
}

/// Print a formatted subsection header to stdout.
pub fn print_subsection_header(title: &str) {
    let ruler = "-".repeat(51);
    println!();
    println!("{ruler}");
    println!("*** {title} ***");
    println!("{ruler}");
    println!();
}
