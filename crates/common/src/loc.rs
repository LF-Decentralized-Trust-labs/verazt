//! Unified source location type shared across all crates.

/// A source span representing a range in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Loc {
    pub start: u32,
    pub end: u32,
}

impl Loc {
    pub fn new(start: u32, end: u32) -> Self {
        Loc { start, end }
    }
}

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
