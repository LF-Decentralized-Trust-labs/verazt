use bat::PrettyPrinter;

/// Print Smart contract IR with highlighted syntax to terminal.
pub fn print_ir(source_code: &str) {
    PrettyPrinter::new()
        .theme("Visual Studio Dark+")
        .line_numbers(true)
        .input_from_bytes(source_code.as_bytes())
        .language("solidity")
        .print()
        .unwrap_or_default();
    print!("");
}
