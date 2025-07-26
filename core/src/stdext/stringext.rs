/// Trait to extend String utilities.
pub trait StringExt<'a> {
    /// Indent all lines of a string.
    fn indent(&self, indent: usize) -> Self;
}

impl<'a> StringExt<'a> for String {
    fn indent(&self, indent: usize) -> Self {
        self.lines()
            .map(|line| {
                if line.is_empty() {
                    line.to_string()
                } else {
                    format!("{}{}", " ".repeat(indent), line)
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
