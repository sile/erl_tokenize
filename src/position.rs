/// Position within a source string.
///
/// Tracks the byte offset from the start of the source and the 1-based
/// line and column of the underlying UTF-8 character. `Position` is
/// intentionally free of any file-path information: callers manage the
/// association between a source string and its identifier (path, URL,
/// buffer name) themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    offset: usize,
    line: usize,
    column: usize,
}

impl Position {
    /// Returns an initial position (offset 0, line 1, column 1).
    pub const fn new() -> Position {
        Position {
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Returns the byte offset from the start of the source.
    pub const fn offset(self) -> usize {
        self.offset
    }

    /// Returns the 1-based line number.
    pub const fn line(self) -> usize {
        self.line
    }

    /// Returns the 1-based column number.
    pub const fn column(self) -> usize {
        self.column
    }

    /// Advance by `width` bytes without crossing a line break.
    ///
    /// Only callers that already know the range does not contain LF
    /// should use this; [`step_by_text`](Self::step_by_text) is safe for
    /// arbitrary text.
    pub(crate) fn step_by_width(mut self, width: usize) -> Position {
        self.offset += width;
        self.column += width;
        self
    }

    /// Advance by `text`, updating line and column across any embedded
    /// line breaks.
    pub(crate) fn step_by_text(mut self, mut text: &str) -> Position {
        while let Some(i) = text.find('\n') {
            self.offset += i + 1;
            self.line += 1;
            self.column = 1;
            text = &text[i + 1..];
        }
        self.offset += text.len();
        self.column += text.len();
        self
    }

    /// Advance by a single character, respecting LF as a line break.
    pub(crate) fn step_by_char(mut self, c: char) -> Position {
        let n = c.len_utf8();
        self.offset += n;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += n;
        }
        self
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
