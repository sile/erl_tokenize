use std::num::NonZeroUsize;

/// Position within a source string.
///
/// Tracks the byte offset from the start of the source and the line and
/// column of the underlying UTF-8 character. Columns advance in bytes
/// rather than characters, so a line that contains multi-byte characters
/// can have columns beyond the visual character count.
///
/// Line and column are typed as [`NonZeroUsize`] because both are
/// 1-based (there is no line 0 or column 0).
///
/// `Position` intentionally carries no file-path information. Callers
/// manage the association between a source string and its identifier
/// (path, URL, buffer name) themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    offset: usize,
    line: NonZeroUsize,
    column: NonZeroUsize,
}

impl Position {
    /// Returns the initial position at the start of a source
    /// (offset 0, line 1, column 1).
    pub const fn new() -> Position {
        Position {
            line: NonZeroUsize::MIN,
            column: NonZeroUsize::MIN,
            offset: 0,
        }
    }

    /// Returns the byte offset from the start of the source.
    ///
    /// [`scan_token`](crate::scan_token) and [`Token::text`](crate::Token::text) use this offset
    /// to slice the source that the caller passes in.
    pub const fn offset(self) -> usize {
        self.offset
    }

    /// Returns the 1-based line number.
    ///
    /// Lines are separated by LF (`\n`). CR (`\r`) is not treated as a
    /// line break.
    pub const fn line(self) -> NonZeroUsize {
        self.line
    }

    /// Returns the 1-based column number.
    ///
    /// Columns are counted in bytes. Column 1 is the start of a line;
    /// crossing a LF resets column to 1.
    pub const fn column(self) -> NonZeroUsize {
        self.column
    }

    /// Advance by `width` bytes without crossing a line break.
    ///
    /// Only callers that already know the range does not contain LF
    /// should use this; [`step_by_text`](Self::step_by_text) is safe for
    /// arbitrary text.
    pub(crate) fn step_by_width(mut self, width: usize) -> Position {
        self.offset += width;
        self.column = self.column.saturating_add(width);
        self
    }

    /// Advance by `text`, updating line and column across any embedded
    /// line breaks.
    pub(crate) fn step_by_text(mut self, mut text: &str) -> Position {
        while let Some(i) = text.find('\n') {
            self.offset += i + 1;
            self.line = self.line.saturating_add(1);
            self.column = NonZeroUsize::MIN;
            text = &text[i + 1..];
        }
        self.offset += text.len();
        self.column = self.column.saturating_add(text.len());
        self
    }

    /// Advance by a single character, respecting LF as a line break.
    pub(crate) fn step_by_char(mut self, c: char) -> Position {
        let n = c.len_utf8();
        self.offset += n;
        if c == '\n' {
            self.line = self.line.saturating_add(1);
            self.column = NonZeroUsize::MIN;
        } else {
            self.column = self.column.saturating_add(n);
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
    /// Formats the position as `line:column`. Offset and file identifier
    /// are not printed.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
