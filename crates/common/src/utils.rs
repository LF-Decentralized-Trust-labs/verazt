/// Print a formatted section header to stdout.
pub fn print_section_header(title: &str) {
    let ruler = "=".repeat(60);
    println!();
    println!("{ruler}");
    println!("*** {title} ***");
    println!("{ruler}");
    println!();
}
