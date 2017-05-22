use std::iter::Peekable;

use {Result, ErrorKind};
use types::Location;

#[derive(Debug)]
pub struct CharReader<T>
    where T: Iterator<Item = char>
{
    chars: Peekable<T>,
    current: Location,
}
impl<T> CharReader<T>
    where T: Iterator<Item = char>
{
    pub fn new(chars: T) -> Self {
        CharReader {
            chars: chars.peekable(),
            current: Location { line: 1, column: 1 },
        }
    }
    pub fn current_location(&self) -> Location {
        self.current
    }
    pub fn read_while<F>(&mut self, f: F) -> String
        where F: Fn(char) -> bool
    {
        let mut buf = String::new();
        while let Some(&c) = self.chars.peek() {
            if !f(c) {
                break;
            }
            self.consume_char();
            buf.push(c);
        }
        buf
    }
    pub fn peek_char(&mut self) -> Result<char> {
        let c = track_try!(self.chars.peek().cloned().ok_or(ErrorKind::UnexpectedEof));
        Ok(c)
    }
    pub fn read_char(&mut self) -> Result<char> {
        let c = track_try!(self.chars.next().ok_or(ErrorKind::UnexpectedEof));
        match c {
            '\n' => {
                self.current.line += 1;
                self.current.column = 1;
            }
            '\t' => {
                // TODO: customize
                self.current.column += 4;
            }
            _ => {
                self.current.column += 1;
            }
        }
        Ok(c)
    }
    pub fn consume_char(&mut self) {
        let _ = self.read_char();
    }
    pub fn is_eos(&mut self) -> bool {
        self.chars.peek().is_none()
    }
}
