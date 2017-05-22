use {Result, ErrorKind, Token};
use char_reader::CharReader;
use tokens;

#[derive(Debug)]
pub struct Tokenizer<T> {
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
               _ => track_try!(self.scan_symbol()),
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
        self.reader.consume_char();
        let line = self.reader.read_while(|c| c != '\n');
        Token::from(tokens::Comment(line))
    }
    fn scan_symbol(&mut self) -> Result<Token> {
        use tokens::Symbol;
        let symbol = match track_try!(self.reader.read_char()) {
            '[' => Symbol::OpenSquare,
            ']' => Symbol::CloseSquare,
            '(' => Symbol::OpenParen,
            ')' => Symbol::CloseParen,
            '{' => Symbol::OpenBrace,
            '}' => Symbol::CloseBrace,
            '#' => Symbol::Sharp,
            '.' => Symbol::Dot,
            ',' => Symbol::Comma,
            ';' => Symbol::Semicolon,
            '?' => Symbol::Question,
            '!' => Symbol::Not,
            '*' => Symbol::Multiply,
            '<' => {
                match self.reader.read_char_if("-=<") {
                    Some('-') => Symbol::LeftAllow,
                    Some('=') => Symbol::DoubleLeftAllow,
                    Some('<') => Symbol::DoubleLeftAngle,
                    _ => Symbol::Less,
                }
            }
            '>' => {
                match self.reader.read_char_if(">=") {
                    Some('>') => Symbol::DoubleRightAngle,
                    Some('=') => Symbol::GreaterEq,
                    _ => Symbol::Greater,
                }
            }
            '/' => {
                if self.reader.read_char_if("=").is_some() {
                    Symbol::NotEq
                } else {
                    Symbol::Slash
                }
            }
            '=' => {
                match self.reader.read_char_if("<>=:/") {
                    Some('<') => Symbol::LessEq,
                    Some('>') => Symbol::DoubleRightAllow,
                    Some('=') => Symbol::Eq,
                    Some(':') => {
                        if self.reader.read_char_if("=").is_some() {
                            Symbol::ExactEq
                        } else {
                            self.reader.unread(':');
                            Symbol::Match
                        }
                    }
                    Some('/') => {
                        if self.reader.read_char_if("=").is_some() {
                            Symbol::ExactNotEq
                        } else {
                            self.reader.unread('/');
                            Symbol::Match
                        }
                    }
                    _ => Symbol::Match,
                }
            }
            ':' => {
                match self.reader.read_char_if("=") {
                    Some('=') => Symbol::MapMatch,
                    _ => Symbol::Colon,
                }
            }
            '|' => {
                match self.reader.read_char_if("|") {
                    Some('|') => Symbol::DoubleVerticalBar,
                    _ => Symbol::VerticalBar,
                }
            }
            '-' => {
                match self.reader.read_char_if("->") {
                    Some('-') => Symbol::MinusMinus,
                    Some('>') => Symbol::RightAllow,
                    _ => Symbol::Hyphen,
                }
            }
            '+' => {
                match self.reader.read_char_if("+") {
                    Some('+') => Symbol::PlusPlus,
                    _ => Symbol::Plus,
                }
            }
            c => track_panic!(ErrorKind::InvalidInput, "Illegal character: {:?}", c),
        };
        Ok(Token::from(symbol))
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
