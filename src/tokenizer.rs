use num::Num;

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
               'a'...'z' => self.scan_atom_or_keyword(),
               'A'...'Z' | '_' => self.scan_variable(),
               '0'...'9' => track_try!(self.scan_number()),
               '$' => track_try!(self.scan_character()),
               '"' => track_try!(self.scan_string()),
               '\'' => track_try!(self.scan_quoted_atom()),
               '%' => self.scan_comment(),
               _ => track_try!(self.scan_symbol()),
           })
    }
    fn scan_character(&mut self) -> Result<Token> {
        self.reader.consume_char();
        let mut c = track_try!(self.reader.read_char());
        if c == '\\' {
            c = track_try!(self.reader.read_escaped_char());
        }
        Ok(Token::from(tokens::Char(c)))
    }
    fn scan_string(&mut self) -> Result<Token> {
        // See: http://erlang.org/doc/reference_manual/data_types.html#id76742
        self.reader.consume_char();
        let mut buf = String::new();
        loop {
            let c = match track_try!(self.reader.read_char()) {
                '\\' => track_try!(self.reader.read_escaped_char()),
                '"' => break,
                c => c,
            };
            buf.push(c);
        }
        Ok(Token::from(tokens::Str(buf)))
    }
    fn scan_variable(&mut self) -> Token {
        fn is_var_char(c: char) -> bool {
            match c {
                'a'...'z' | 'A'...'Z' | '@' | '_' | '0'...'9' => true,
                _ => false,
            }
        }
        let var = self.reader.read_while(is_var_char);
        Token::from(tokens::Var(var))
    }
    fn scan_number(&mut self) -> Result<Token> {
        // See: http://erlang.org/doc/reference_manual/data_types.html#id65900
        let mut buf = String::new();
        while let Ok(c) = self.reader.peek_char() {
            match c {
                '0'...'9' => {
                    self.reader.consume_char();
                    buf.push(c);
                }
                '.' => {
                    self.reader.consume_char();
                    buf.push('.');
                    return track!(self.scan_float(buf));
                }
                '#' => {
                    self.reader.consume_char();
                    let radix = track_try!(buf.parse());
                    track_assert!(1 < radix && radix < 37,
                                  ErrorKind::InvalidInput,
                                  "Illegal Radix: {:?}",
                                  buf);
                    return track!(self.scan_integer(radix));
                }
                _ => break,
            }
        }
        let n = track_try!(buf.parse());
        Ok(Token::from(tokens::Int(n)))
    }
    fn scan_integer(&mut self, radix: u32) -> Result<Token> {
        let buf = self.reader.read_while(|c| c.is_digit(radix));
        let n = track_try!(Num::from_str_radix(&buf, radix));
        Ok(Token::from(tokens::Int(n)))
    }
    fn scan_float(&mut self, mut buf: String) -> Result<Token> {
        buf.push_str(&self.reader.read_while(|c| c.is_digit(10)));
        if self.reader.read_char_if("eE").is_some() {
            buf.push('e');
            if let Some(sign) = self.reader.read_char_if("-+") {
                buf.push(sign);
            }
            buf.push_str(&self.reader.read_while(|c| c.is_digit(10)));
        }
        let n = track_try!(buf.parse());
        Ok(Token::from(tokens::Float(n)))
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
    fn scan_atom_or_keyword(&mut self) -> Token {
        fn is_atom_non_leading_char(c: char) -> bool {
            match c {
                'a'...'z' | 'A'...'Z' | '@' | '_' | '0'...'9' => true,
                _ => c.is_alphabetic(),
            }
        }
        let name = self.reader.read_while(is_atom_non_leading_char);
        if let Some(k) = tokens::Keyword::from_str(&name) {
            Token::from(k)
        } else {
            Token::from(tokens::Atom(name))
        }
    }
    fn scan_quoted_atom(&mut self) -> Result<Token> {
        self.reader.consume_char();
        let mut buf = String::new();
        loop {
            let c = match track_try!(self.reader.read_char()) {
                '\\' => track_try!(self.reader.read_escaped_char()),
                '\'' => break,
                c => c,
            };
            buf.push(c);
        }
        Ok(Token::from(tokens::Atom(buf)))
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
