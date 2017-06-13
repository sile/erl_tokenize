use std::fmt;

use {Result, ErrorKind, Position, PositionRange, HiddenToken, LexicalToken};
use tokens::{AtomToken, CharToken, FloatToken, IntegerToken, KeywordToken, StringToken,
             SymbolToken, VariableToken, CommentToken, WhitespaceToken};

/// Token.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum Token {
    Atom(AtomToken),
    Char(CharToken),
    Comment(CommentToken),
    Float(FloatToken),
    Integer(IntegerToken),
    Keyword(KeywordToken),
    String(StringToken),
    Symbol(SymbolToken),
    Variable(VariableToken),
    Whitespace(WhitespaceToken),
}
impl Token {
    /// Tries to convert from any prefixes of the text to a token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::{Token, Position};
    /// use erl_tokenize::values::Symbol;
    ///
    /// let pos = Position::new();
    ///
    /// // Atom
    /// let token = Token::from_text("foo", pos.clone()).unwrap();
    /// assert_eq!(token.as_atom_token().map(|t| t.value()), Some("foo"));
    ///
    /// // Symbol
    /// let token = Token::from_text("[foo]", pos.clone()).unwrap();
    /// assert_eq!(token.as_symbol_token().map(|t| t.value()), Some(Symbol::OpenSquare));
    /// ```
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let head = track_try!(text.chars().nth(0).ok_or(ErrorKind::UnexpectedEos));
        match head {
            ' ' | '\t' | '\r' | '\n' | '\u{A0}' => {
                track!(WhitespaceToken::from_text(text, pos)).map(Token::from)
            }
            'A'...'Z' | '_' => track!(VariableToken::from_text(text, pos)).map(Token::from),
            '0'...'9' => {
                let maybe_float = if let Some(i) = text.find(|c: char| !c.is_digit(10)) {
                    text.as_bytes()[i] == b'.' &&
                    text.as_bytes()
                        .get(i + 1)
                        .map_or(false, |c| (*c as char).is_digit(10))
                } else {
                    false
                };
                if maybe_float {
                    track!(FloatToken::from_text(text, pos)).map(Token::from)
                } else {
                    track!(IntegerToken::from_text(text, pos)).map(Token::from)
                }
            }
            '$' => track!(CharToken::from_text(text, pos)).map(Token::from),
            '"' => track!(StringToken::from_text(text, pos)).map(Token::from),
            '\'' => track!(AtomToken::from_text(text, pos)).map(Token::from),
            '%' => track!(CommentToken::from_text(text, pos)).map(Token::from),
            _ => {
                if head.is_alphabetic() {
                    let atom = track_try!(AtomToken::from_text(text, pos.clone()));
                    if let Ok(keyword) = KeywordToken::from_text(atom.text(), pos) {
                        Ok(Token::from(keyword))
                    } else {
                        Ok(Token::from(atom))
                    }
                } else {
                    track!(SymbolToken::from_text(text, pos)).map(Token::from)
                }
            }
        }
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::{Token, Position};
    ///
    /// let pos = Position::new();
    ///
    /// // Comment
    /// assert_eq!(Token::from_text("% foo", pos.clone()).unwrap().text(), "% foo");
    ///
    /// // Char
    /// assert_eq!(Token::from_text(r#"$\t"#, pos.clone()).unwrap().text(), r#"$\t"#);
    /// ```
    pub fn text(&self) -> &str {
        match *self {
            Token::Atom(ref t) => t.text(),
            Token::Char(ref t) => t.text(),
            Token::Comment(ref t) => t.text(),
            Token::Float(ref t) => t.text(),
            Token::Integer(ref t) => t.text(),
            Token::Keyword(ref t) => t.text(),
            Token::String(ref t) => t.text(),
            Token::Symbol(ref t) => t.text(),
            Token::Variable(ref t) => t.text(),
            Token::Whitespace(ref t) => t.text(),
        }
    }
}
impl From<AtomToken> for Token {
    fn from(f: AtomToken) -> Self {
        Token::Atom(f)
    }
}
impl From<CharToken> for Token {
    fn from(f: CharToken) -> Self {
        Token::Char(f)
    }
}
impl From<CommentToken> for Token {
    fn from(f: CommentToken) -> Self {
        Token::Comment(f)
    }
}
impl From<FloatToken> for Token {
    fn from(f: FloatToken) -> Self {
        Token::Float(f)
    }
}
impl From<IntegerToken> for Token {
    fn from(f: IntegerToken) -> Self {
        Token::Integer(f)
    }
}
impl From<KeywordToken> for Token {
    fn from(f: KeywordToken) -> Self {
        Token::Keyword(f)
    }
}
impl From<StringToken> for Token {
    fn from(f: StringToken) -> Self {
        Token::String(f)
    }
}
impl From<SymbolToken> for Token {
    fn from(f: SymbolToken) -> Self {
        Token::Symbol(f)
    }
}
impl From<VariableToken> for Token {
    fn from(f: VariableToken) -> Self {
        Token::Variable(f)
    }
}
impl From<WhitespaceToken> for Token {
    fn from(f: WhitespaceToken) -> Self {
        Token::Whitespace(f)
    }
}
impl From<HiddenToken> for Token {
    fn from(f: HiddenToken) -> Self {
        match f {
            HiddenToken::Comment(t) => t.into(),
            HiddenToken::Whitespace(t) => t.into(),
        }
    }
}
impl From<LexicalToken> for Token {
    fn from(f: LexicalToken) -> Self {
        match f {
            LexicalToken::Atom(t) => t.into(),
            LexicalToken::Char(t) => t.into(),
            LexicalToken::Float(t) => t.into(),
            LexicalToken::Integer(t) => t.into(),
            LexicalToken::Keyword(t) => t.into(),
            LexicalToken::String(t) => t.into(),
            LexicalToken::Symbol(t) => t.into(),
            LexicalToken::Variable(t) => t.into(),
        }
    }
}
impl PositionRange for Token {
    fn start_position(&self) -> Position {
        match *self {
            Token::Atom(ref t) => t.start_position(),
            Token::Char(ref t) => t.start_position(),
            Token::Comment(ref t) => t.start_position(),
            Token::Float(ref t) => t.start_position(),
            Token::Integer(ref t) => t.start_position(),
            Token::Keyword(ref t) => t.start_position(),
            Token::String(ref t) => t.start_position(),
            Token::Symbol(ref t) => t.start_position(),
            Token::Variable(ref t) => t.start_position(),
            Token::Whitespace(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Token::Atom(ref t) => t.end_position(),
            Token::Char(ref t) => t.end_position(),
            Token::Comment(ref t) => t.end_position(),
            Token::Float(ref t) => t.end_position(),
            Token::Integer(ref t) => t.end_position(),
            Token::Keyword(ref t) => t.end_position(),
            Token::String(ref t) => t.end_position(),
            Token::Symbol(ref t) => t.end_position(),
            Token::Variable(ref t) => t.end_position(),
            Token::Whitespace(ref t) => t.end_position(),
        }
    }
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.text().fmt(f)
    }
}
