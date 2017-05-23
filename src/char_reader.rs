use std::char;
use num::Num;

use {Result, ErrorKind};

#[derive(Debug)]
pub struct CharReader<T> {
    chars: T,
    unreads: Vec<char>,
}
impl<T> CharReader<T>
    where T: Iterator<Item = char>
{
    pub fn new(chars: T) -> Self {
        CharReader {
            chars: chars,
            unreads: Vec::new(),
        }
    }
    pub fn unread(&mut self, c: char) {
        self.unreads.push(c)
    }
    pub fn read_while<F>(&mut self, f: F) -> String
        where F: Fn(char) -> bool
    {
        let mut buf = String::new();
        while let Ok(c) = self.peek_char() {
            if !f(c) {
                break;
            }
            self.consume_char();
            buf.push(c);
        }
        buf
    }
    pub fn peek_char(&mut self) -> Result<char> {
        if let Some(c) = self.unreads.last().cloned() {
            Ok(c)
        } else if let Some(c) = self.chars.next() {
            self.unreads.push(c);
            Ok(c)
        } else {
            track_panic!(ErrorKind::UnexpectedEof)
        }
    }
    pub fn read_char_if(&mut self, expects: &str) -> Option<char> {
        self.peek_char()
            .ok()
            .and_then(|c| if expects.contains(c) {
                          self.consume_char();
                          Some(c)
                      } else {
                          None
                      })
    }
    pub fn read_char(&mut self) -> Result<char> {
        if let Some(c) = self.unreads.pop() {
            Ok(c)
        } else {
            let c = track_try!(self.chars.next().ok_or(ErrorKind::UnexpectedEof));
            Ok(c)
        }
    }
    pub fn read_escaped_char(&mut self) -> Result<char> {
        let c = track_try!(self.read_char());
        Ok(match c {
               'b' => 8 as char, // Back Space
               'd' => 127 as char, // Delete
               'e' => 27 as char, // Escape,
               'f' => 12 as char, // Form Feed
               'n' => '\n',
               'r' => '\r',
               's' => ' ',
               't' => '\t',
               'v' => 11 as char, // Vertical Tabulation
               '^' => {
                   let c = track_try!(self.read_char());
                   (c as u32 % 32) as u8 as char
               }
               'x' => {
                   let c = self.read_char()?;
                   let buf = if c == '{' {
                       let buf = self.read_while(|c| c != '}');
                       self.consume_char();
                       buf
                   } else {
                       let mut buf = String::with_capacity(2);
                       buf.push(c);
                       buf.push(track_try!(self.read_char()));
                       buf
                   };
                   let code: u32 = track_try!(Num::from_str_radix(&buf, 16));
                   track_try!(char::from_u32(code).ok_or(ErrorKind::InvalidInput))
               }
               c @ '0'...'7' => {
                   let mut n = c.to_digit(8).unwrap();
                   if let Some(c) = self.read_char_if("012345677") {
                       n = (n * 8) + c.to_digit(8).unwrap();
                   }
                   if let Some(c) = self.read_char_if("012345677") {
                       n = (n * 8) + c.to_digit(8).unwrap();
                   }
                   track_try!(char::from_u32(n).ok_or(ErrorKind::InvalidInput))
               }
               _ => c,
           })
    }
    pub fn consume_char(&mut self) {
        let _ = self.read_char();
    }
    pub fn is_eos(&mut self) -> bool {
        self.peek_char().is_err()
    }
}
