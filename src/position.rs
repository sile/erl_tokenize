use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Position of token.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Position {
    filepath: Option<Arc<PathBuf>>,
    offset: usize,
    line: usize,
    column: usize,
}
impl Position {
    /// Returns an initial position.
    pub fn new() -> Position {
        Position {
            filepath: None,
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Returns the file path where this token is located.
    pub fn filepath(&self) -> Option<&PathBuf> {
        self.filepath.as_ref().map(AsRef::as_ref)
    }

    /// Returns an offset from the beginning of the buffer.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns a line number from the beginning of the buffer (1-based).
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns a column number from the beginning of the line (1-based).
    pub fn column(&self) -> usize {
        self.column
    }

    /// Sets the file path where this token is located.
    pub(crate) fn set_filepath<P: AsRef<Path>>(&mut self, path: P) {
        self.filepath = Some(Arc::new(path.as_ref().to_path_buf()));
    }

    /// Steps a position by the given width.
    pub(crate) fn step_by_width(mut self, witdh: usize) -> Position {
        self.offset += witdh;
        self.column += witdh;
        self
    }

    /// Steps a position by the given text.
    pub(crate) fn step_by_text(mut self, mut text: &str) -> Position {
        while let Some(i) = text.find('\n') {
            self.offset += i + 1;
            self.line += 1;
            self.column = 1;
            let len = text.len();
            text = unsafe { text.get_unchecked(i + 1..len) };
        }
        self.offset += text.len();
        self.column += text.len();
        self
    }

    pub(crate) fn step_by_char(mut self, c: char) -> Position {
        if c == '\n' {
            self.offset += 1;
            self.line += 1;
            self.column = 1;
        } else {
            self.offset += 1;
            self.column += 1;
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
        write!(
            f,
            "{}:{}:{}",
            self.filepath
                .as_ref()
                .and_then(|f| f.to_str())
                .unwrap_or("<unknown>"),
            self.line,
            self.column
        )
    }
}

impl std::ops::Add<usize> for Position {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        self.step_by_width(rhs)
    }
}

/// This trait allows to get a (half-open) range where the subject is located.
pub trait PositionRange {
    /// Returns the (inclusive) start position of this.
    fn start_position(&self) -> Position;

    /// Returns the (exclusive) end position of this.
    fn end_position(&self) -> Position;
}
impl<T: PositionRange> PositionRange for Box<T> {
    fn start_position(&self) -> Position {
        (**self).start_position()
    }

    fn end_position(&self) -> Position {
        (**self).end_position()
    }
}
