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
        let c = track_try!(self.reader.read_char());
        let symbol = match c {
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
            '<' | '>' | '/' | '=' | ':' | '|' | '-' | '+' => self.scan_multi_char_symbol(c),
            c => track_panic!(ErrorKind::InvalidInput, "Illegal character: {:?}", c),
        };
        Ok(Token::from(symbol))
    }
    fn scan_multi_char_symbol(&mut self, c0: char) -> tokens::Symbol {
        use tokens::Symbol;
        let c1 = self.reader.read_char().ok();
        match (c0, c1) {
            ('<', Some('-')) => Symbol::LeftAllow,
            ('<', Some('=')) => Symbol::DoubleLeftAllow,
            ('<', Some('<')) => Symbol::DoubleLeftAngle,
            ('<', _) => {
                c1.map(|c| self.reader.unread(c));
                Symbol::Less
            }
            ('>', Some('>')) => Symbol::DoubleRightAngle,
            ('>', Some('=')) => Symbol::GreaterEq,
            ('>', _) => {
                c1.map(|c| self.reader.unread(c));
                Symbol::Greater
            }
            ('/', Some('=')) => Symbol::NotEq,
            ('/', _) => {
                c1.map(|c| self.reader.unread(c));
                Symbol::Slash
            }
            (':', Some('=')) => Symbol::MapMatch,
            (':', _) => {
                c1.map(|c| self.reader.unread(c));
                Symbol::Colon
            }
            ('|', Some('|')) => Symbol::DoubleVerticalBar,
            ('|', _) => {
                c1.map(|c| self.reader.unread(c));
                Symbol::VerticalBar
            }
            ('-', Some('-')) => Symbol::MinusMinus,
            ('-', Some('>')) => Symbol::RightAllow,
            ('-', _) => {
                c1.map(|c| self.reader.unread(c));
                Symbol::Hyphen
            }
            ('+', Some('+')) => Symbol::PlusPlus,
            ('+', _) => {
                c1.map(|c| self.reader.unread(c));
                Symbol::Plus
            }
            ('=', Some('<')) => Symbol::LessEq,
            ('=', Some('>')) => Symbol::DoubleRightAllow,
            ('=', Some('=')) => Symbol::Eq,
            ('=', Some(':')) => {
                if self.reader.peek_char().ok() == Some('=') {
                    self.reader.consume_char();
                    Symbol::ExactEq
                } else {
                    self.reader.unread(':');
                    Symbol::Match
                }
            }
            ('=', Some('/')) => {
                if self.reader.peek_char().ok() == Some('=') {
                    self.reader.consume_char();
                    Symbol::ExactEq
                } else {
                    self.reader.unread('/');
                    Symbol::Match
                }
            }
            ('=', _) => {
                c1.map(|c| self.reader.unread(c));
                Symbol::Match
            }
            _ => unreachable!(),
        }
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
