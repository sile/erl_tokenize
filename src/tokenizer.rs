use {Result, ErrorKind, Token};
use char_reader::CharReader;
use tokens;

#[derive(Debug)]
pub struct Tokenizer<T>
    where T: Iterator<Item = char>
{
    reader: CharReader<T>,
}
impl<T> Tokenizer<T>
    where T: Iterator<Item = char>
{
    pub fn new(chars: T) -> Self {
        Tokenizer { reader: CharReader::new(chars) }
    }

    fn scan_token(&mut self) -> Result<Token> {
        let c = track_try!(self.reader.peek_char());
        Ok(match c {
               ' ' | '\t' | '\r' | '\n' => self.scan_whitespace(),
               'a'...'z' => unimplemented!(),
               'A'...'Z' => unimplemented!(),
               '0'...'9' => unimplemented!(),
               '$' => unimplemented!(),
               '"' => unimplemented!(),
               '\'' => unimplemented!(),
               '%' => self.scan_comment(),
               _ => unimplemented!(),
           })
    }
    fn scan_whitespace(&mut self) -> Token {
        let whitespace = match self.reader.read_char() {
            Ok(' ') => tokens::Whitespace::Space,
            Ok('\t') => tokens::Whitespace::Tab,
            Ok('\r') => tokens::Whitespace::Return,
            Ok('\n') => tokens::Whitespace::Newline,
            _ => unreachable!(),
        };
        Token::from(whitespace)
    }
    fn scan_comment(&mut self) -> Token {
        let line = self.reader.read_while(|c| c != '\n');
        Token::from(tokens::Comment(line))
    }
}
impl<T> Iterator for Tokenizer<T>
    where T: Iterator<Item = char>
{
    type Item = Result<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.is_eos() {
            None
        } else {
            Some(track!(self.scan_token()))
        }
    }
}
