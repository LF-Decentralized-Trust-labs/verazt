//! Yul Keywords

/// Yul Keywords
pub const YUL_KEYWORDS: &[&str] = &[
    "break", "case", "code", "continue", "data", "default", "for", "function", "leave", "let",
    "object", "switch",
];

/// Yul keywords that cannot be variable names
pub const YUL_RESERVED_NAMES: &[&str] = &[
    "break", "case", "code", "continue", "default", "for", "function", "leave", "let", "object",
    "switch",
];
