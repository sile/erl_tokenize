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
    pub fn read_char(&mut self) -> Result<char> {
        if let Some(c) = self.unreads.pop() {
            Ok(c)
        } else {
            let c = track_try!(self.chars.next().ok_or(ErrorKind::UnexpectedEof));
            Ok(c)
        }
    }
    pub fn consume_char(&mut self) {
        let _ = self.read_char();
    }
    pub fn is_eos(&mut self) -> bool {
        self.peek_char().is_err()
    }
}
