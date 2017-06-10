/// Position of token.
#[derive(Debug, Clone)]
pub struct Position {
    offset: usize,
    line: usize,
    column: usize,
}
impl Position {
    /// Returns an initial position.
    pub fn new() -> Position {
        Position {
            line: 1,
            column: 1,
            offset: 0,
        }
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
