use std::iter::Peekable;

use {Result, ErrorKind, Token};
use types::Location;

#[derive(Debug)]
pub struct Tokenizer<T>
    where T: Iterator<Item = char>
{
    chars: Peekable<T>,
    current: Location,
}
impl<T> Tokenizer<T>
    where T: Iterator<Item = char>
{
    pub fn new(chars: T) -> Self {
        Tokenizer {
            chars: chars.peekable(),
            current: Location { line: 0, column: 0 },
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        let c = track_try!(self.chars.peek().ok_or(ErrorKind::UnexpectedEof));
        match *c {
            ' ' | '\t' | '\r' | '\n' => unimplemented!(),
            'a'...'z' => unimplemented!(),
            'A'...'Z' => unimplemented!(),
            '0'...'9' => unimplemented!(),
            '$' => unimplemented!(),
            '"' => unimplemented!(),
            '\'' => unimplemented!(),
            '%' => unimplemented!(),
            _ => unimplemented!(),
        }
    }
    fn is_eos(&mut self) -> bool {
        self.chars.peek().is_none()
    }
}
impl<T> Iterator for Tokenizer<T>
    where T: Iterator<Item = char>
{
    type Item = Result<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_eos() {
            None
        } else {
            Some(track!(self.next_token()))
        }
    }
}
