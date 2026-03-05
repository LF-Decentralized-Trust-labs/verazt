//! Pretty-printing for CIR using bat syntax highlighting.

use bat::PrettyPrinter;

/// Print CIR source text with syntax highlighting (Solidity-like).
pub fn print_ir(source: &str) {
    PrettyPrinter::new()
        .language("solidity")
        .input_from_bytes(source.as_bytes())
        .print()
        .unwrap();
}
