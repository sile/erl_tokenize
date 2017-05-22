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
        loop {
            match c {
                ' ' | '\t' | '\r' | '\n' => self.reader.consume_char(),
                'a'...'z' => unimplemented!(),
                'A'...'Z' => unimplemented!(),
                '0'...'9' => unimplemented!(),
                '$' => unimplemented!(),
                '"' => unimplemented!(),
                '\'' => unimplemented!(),
                '%' => return self.scan_comment(),
                _ => unimplemented!(),
            }
        }
    }
    fn scan_comment(&mut self) -> Result<Token> {
        let location = self.reader.current_location();
        let line = self.reader.read_while(|c| c != '\n');
        let _ = self.reader.consume_char();
        Ok(Token::new(location, tokens::Comment(line)))
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
