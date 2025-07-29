//! Module for handling localization.

use std::fs;

/// Data structure to capture line position information of a file.
///
/// Line and column number starts from 0.
pub struct Localizer {
    /// File path.
    pub file_path: String,

    /// Vector containing the byte-based position of each line in the file.
    /// For example, the position of line `i` in the file is `line_pos[i]`.
    lines_pos: Vec<usize>,
}

impl Localizer {
    /// Constructor to construct line-position mapping from a file.
    pub fn new(file_path: String) -> Option<Self> {
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let content = content.as_bytes();
                if content.is_empty() {
                    return None;
                }
                let length = content.len();
                let mut lines_pos = vec![];
                // Record the first line position
                lines_pos.push(0);
                // Record next lines' positions
                for (i, item) in content.iter().enumerate().take(length - 1) {
                    if *item == b'\n' {
                        // Record other newline position
                        lines_pos.push(i + 1)
                    }
                }
                // Record the last line whose position is the file length
                lines_pos.push(length);
                Some(Localizer { file_path, lines_pos })
            }
            Err(_) => None,
        }
    }

    /// Get the byte-based position of a line.
    pub fn get_pos_of_line(&self, line: usize) -> Option<usize> {
        if line >= self.lines_pos.len() {
            None
        } else {
            Some(self.lines_pos[line])
        }
    }

    /// Get the line number of a position in the file.
    pub fn get_line_of_pos(&self, pos: usize) -> Option<usize> {
        // Find the line number
        let last_line = self.lines_pos.len() - 1;
        if pos < self.lines_pos[0] || pos > self.lines_pos[last_line] {
            return None;
        }

        // Binary search to find line number
        let mut start_line = 0;
        let mut end_line = last_line;
        while start_line + 1 < end_line {
            let tmp_line = (start_line + end_line) / 2;
            match self.lines_pos[tmp_line] {
                l if l == pos => return Some(tmp_line),
                l if l > pos => end_line = tmp_line,
                _ => start_line = tmp_line,
            }
        }

        Some(start_line)
    }

    /// Get the column number of a position in the file.
    pub fn get_column_of_pos(&self, pos: usize) -> Option<usize> {
        match self.get_line_of_pos(pos) {
            None => None,
            Some(line) => self.get_pos_of_line(line).map(|line_pos| pos - line_pos),
        }
    }

    /// Get line and column number of a position in the file.
    pub fn get_line_column(&self, pos: usize) -> Option<(usize, usize)> {
        match self.get_line_of_pos(pos) {
            None => None,
            Some(line) => match self.get_pos_of_line(line) {
                None => None,
                Some(line_pos) => {
                    let column = pos - line_pos;
                    Some((line, column))
                }
            },
        }
    }
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use crate::parser::localizer::Localizer;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_line_pos() {
        let mut file = NamedTempFile::new().unwrap();

        let content = "123\n45678\n9abcdef";
        write!(file, "{content}").ok();

        let file_path = file.path().as_os_str().to_os_string().into_string();
        assert!(file_path.is_ok());

        let file_path = file_path.unwrap();
        let line_pos = Localizer::new(file_path).unwrap();

        assert_eq!(line_pos.get_line_of_pos(1).unwrap(), 0);
        assert_eq!(line_pos.get_line_of_pos(3).unwrap(), 0);
        assert_eq!(line_pos.get_line_of_pos(4).unwrap(), 1);
        assert_eq!(line_pos.get_line_of_pos(5).unwrap(), 1);
        assert_eq!(line_pos.get_line_of_pos(12).unwrap(), 2);
        assert_eq!(line_pos.get_pos_of_line(1).unwrap(), 4);
    }
}
