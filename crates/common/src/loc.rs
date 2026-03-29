//! Unified source location type shared across all crates.

use serde::{Deserialize, Serialize};

/// A source span representing a range in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Loc {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    /// The source file this location refers to (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

impl Loc {
    /// Create a new source location with line and column info.
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Loc { start_line, start_col, end_line, end_col, file: None }
    }

    /// Attach a source file path to this location.
    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    /// Returns `false` if `start_line == 0` (invalid / dummy location).
    pub fn is_valid(&self) -> bool {
        self.start_line != 0
    }
}

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref file) = self.file {
            write!(f, "{}:{}:{}", file, self.start_line, self.start_col)
        } else {
            write!(f, "{}:{}-{}:{}", self.start_line, self.start_col, self.end_line, self.end_col)
        }
    }
}
