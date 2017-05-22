use std::iter::Peekable;

use {Result, ErrorKind};

#[derive(Debug)]
pub struct CharReader<T>
    where T: Iterator<Item = char>
{
    chars: Peekable<T>,
}
impl<T> CharReader<T>
    where T: Iterator<Item = char>
{
    pub fn new(chars: T) -> Self {
        CharReader { chars: chars.peekable() }
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
        Ok(c)
    }
    pub fn consume_char(&mut self) {
        let _ = self.chars.next();
    }
    pub fn is_eos(&mut self) -> bool {
        self.chars.peek().is_none()
    }
}
