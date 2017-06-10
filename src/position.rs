use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Position of token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    filepath: Option<Rc<PathBuf>>,
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
        self.filepath.as_ref().map(|p| p.as_ref())
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
        self.filepath = Some(Rc::new(path.as_ref().to_path_buf()));
    }

    /// Step a position.
    pub(crate) fn step(&mut self, witdh: usize) {
        self.offset += witdh;
        self.column += witdh;
    }

    /// Step a position as a newline.
    pub(crate) fn new_line(&mut self) {
        self.offset += 1;
        self.line += 1;
        self.column = 1;
    }
}
