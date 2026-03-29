/// Extract a code snippet around the given range, with context lines and column
/// markers.
///
/// Returns `None` if the file cannot be read or the line numbers are invalid (0
/// or out of range). Lines are 1-indexed. Columns are 1-indexed.
///
/// For single-line ranges, a `^^^` marker is placed from `start_col` to
/// `end_col`. For multi-line ranges, a single `^` marker is placed at the start
/// column on the first line and at the end column on the last line.
pub fn extract_snippet(
    file_path: &str,
    start_line: usize,
    end_line: usize,
    start_col: usize,
    end_col: usize,
    context: usize,
) -> Option<String> {
    let content = std::fs::read_to_string(file_path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if start_line == 0 || start_line > lines.len() {
        return None;
    }
    let end_line = end_line.max(start_line).min(lines.len());

    let start_print = start_line.saturating_sub(context + 1);
    let end_print = (end_line + context).min(lines.len());
    let mut result = String::new();

    let single_line = start_line == end_line;
    let has_col = start_col > 0 && end_col > 0;

    let skip_start = start_line + 2;
    let skip_end = end_line.saturating_sub(1);
    let has_truncation = skip_start + 1 < skip_end;
    let mut truncated = false;

    for i in start_print..end_print {
        let current_line = i + 1;

        if has_truncation && current_line >= skip_start && current_line < skip_end {
            if !truncated {
                result.push_str("   ... |\n");
                truncated = true;
            }
            continue;
        }

        let in_range = current_line >= start_line && current_line <= end_line;
        let prefix = if in_range { "> " } else { "  " };
        result.push_str(&format!("{}{:4} | {}\n", prefix, current_line, lines[i]));

        if has_col && single_line && current_line == start_line {
            // Single-line: underline from start_col to end_col
            let marker_start = start_col.saturating_sub(1);
            let marker_len = if end_col >= start_col {
                end_col - start_col + 1
            } else {
                1
            };
            let padding = " ".repeat(marker_start);
            let carets = "^".repeat(marker_len);
            result.push_str(&format!("    .. | {}{}\n", padding, carets));
        }
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_snippet_basic() {
        let dir = std::env::temp_dir().join("verazt_snippet_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.sol");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "pragma solidity ^0.8.0;").unwrap();
        writeln!(f, "contract Foo {{").unwrap();
        writeln!(f, "    uint x;").unwrap();
        writeln!(f, "}}").unwrap();

        let snippet = extract_snippet(path.to_str().unwrap(), 2, 2, 0, 0, 1).unwrap();
        assert!(snippet.contains(">    2 | contract Foo {"));
        assert!(snippet.contains("     1 | pragma solidity ^0.8.0;"));
        assert!(snippet.contains("     3 |     uint x;"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_snippet_invalid_line() {
        let dir = std::env::temp_dir().join("verazt_snippet_test2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test2.sol");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "line1").unwrap();

        assert!(extract_snippet(path.to_str().unwrap(), 0, 0, 0, 0, 1).is_none());
        assert!(extract_snippet(path.to_str().unwrap(), 5, 5, 0, 0, 1).is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_snippet_nonexistent_file() {
        assert!(extract_snippet("/tmp/nonexistent_verazt_test.sol", 1, 1, 0, 0, 1).is_none());
    }

    #[test]
    fn test_extract_snippet_single_line_range() {
        let dir = std::env::temp_dir().join("verazt_snippet_test_single");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_single.sol");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "// comment").unwrap();
        writeln!(f, "    function IamMissing()").unwrap();
        writeln!(f, "        public").unwrap();

        // Highlight columns 5-24 on line 2: "function IamMissing()"
        let snippet = extract_snippet(path.to_str().unwrap(), 2, 2, 5, 24, 1).unwrap();
        assert!(snippet.contains(">    2 |     function IamMissing()"));
        assert!(snippet.contains("    .. |     ^^^^^^^^^^^^^^^^^^^^"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_snippet_multi_line_range() {
        let dir = std::env::temp_dir().join("verazt_snippet_test_multi");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_multi.sol");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "// comment").unwrap();
        writeln!(f, "    function IamMissing() {{").unwrap();
        writeln!(f, "        public").unwrap();
        writeln!(f, "    }}").unwrap();
        writeln!(f, "// end").unwrap();

        // Highlight from line 2 col 5 to line 4 col 5
        let snippet = extract_snippet(path.to_str().unwrap(), 2, 4, 5, 5, 1).unwrap();
        // First line of range gets > prefix
        assert!(snippet.contains(">    2 |     function IamMissing() {"));
        // All lines in range get > prefix
        assert!(snippet.contains(">    3 |         public"));
        assert!(snippet.contains(">    4 |     }"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_extract_snippet_long_range_truncation() {
        let dir = std::env::temp_dir().join("verazt_snippet_test_trunc");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_trunc.sol");
        let mut f = std::fs::File::create(&path).unwrap();
        for i in 1..=10 {
            writeln!(f, "line {}", i).unwrap();
        }

        // Feature: 10 lines, highlight from 2 to 9
        // Print context: 1 context line -> start_print = 1, end_print = 10
        // Truncation: start_line=2, end_line=9.
        // skip_start = 2 + 2 = 4. skip_end = 9 - 1 = 8.
        // It will skip lines 4, 5, 6, 7.
        // Should print lines 1, 2, 3, then "   ... |\n", then 8, 9, 10.
        let snippet = extract_snippet(path.to_str().unwrap(), 2, 9, 0, 0, 1).unwrap();
        
        // Assert we have standard formatting
        assert!(snippet.contains("     1 | line 1"));
        assert!(snippet.contains(">    2 | line 2"));
        assert!(snippet.contains(">    3 | line 3"));
        assert!(snippet.contains("   ... |"));
        assert!(!snippet.contains("line 5")); // truncated
        assert!(snippet.contains(">    8 | line 8"));
        assert!(snippet.contains(">    9 | line 9"));
        assert!(snippet.contains("    10 | line 10"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
