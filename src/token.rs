use std::fmt;
use num::BigUint;

use {Result, ErrorKind, Position, PositionRange, HiddenToken, LexicalToken};
use tokens;
use values::{Keyword, Symbol, Whitespace};

/// Token.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum Token {
    Atom(tokens::AtomToken),
    Char(tokens::CharToken),
    Comment(tokens::CommentToken),
    Float(tokens::FloatToken),
    Integer(tokens::IntegerToken),
    Keyword(tokens::KeywordToken),
    String(tokens::StringToken),
    Symbol(tokens::SymbolToken),
    Variable(tokens::VariableToken),
    Whitespace(tokens::WhitespaceToken),
}
impl Token {
    /// Makes a new `Token` from the value.
    pub fn from_value(value: TokenValue, pos: Position) -> Result<Self> {
        match value {
            TokenValue::Atom(v) => Ok(tokens::AtomToken::from_value(v, pos).into()),
            TokenValue::Char(v) => Ok(tokens::CharToken::from_value(v, pos).into()),
            TokenValue::Comment(v) => {
                track!(tokens::CommentToken::from_value(v, pos)).map(Token::from)
            }
            TokenValue::Float(v) => Ok(tokens::FloatToken::from_value(v, pos).into()),
            TokenValue::Integer(v) => Ok(tokens::IntegerToken::from_value(v.clone(), pos).into()),
            TokenValue::Keyword(v) => Ok(tokens::KeywordToken::from_value(v, pos).into()),
            TokenValue::String(v) => Ok(tokens::StringToken::from_value(v, pos).into()),
            TokenValue::Symbol(v) => Ok(tokens::SymbolToken::from_value(v, pos).into()),
            TokenValue::Variable(v) => {
                track!(tokens::VariableToken::from_value(v, pos)).map(Token::from)
            }
            TokenValue::Whitespace(v) => Ok(tokens::WhitespaceToken::from_value(v, pos).into()),
        }
    }

    /// Tries to convert from any prefixes of the text to a token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::{Token, TokenValue, Position};
    /// use erl_tokenize::values::Symbol;
    ///
    /// let pos = Position::new();
    ///
    /// // Atom
    /// assert_eq!(Token::from_text("foo", pos.clone()).unwrap().value(), TokenValue::Atom("foo"));
    ///
    /// // Symbol
    /// assert_eq!(Token::from_text("[foo]", pos.clone()).unwrap().value(),
    ///            TokenValue::Symbol(Symbol::OpenSquare));
    /// ```
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let head = track_try!(text.chars().nth(0).ok_or(ErrorKind::UnexpectedEos));
        match head {
            ' ' | '\t' | '\r' | '\n' | '\u{A0}' => {
                track!(tokens::WhitespaceToken::from_text(text, pos)).map(Token::from)
            }
            'A'...'Z' | '_' => track!(tokens::VariableToken::from_text(text, pos)).map(Token::from),
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
                    track!(tokens::FloatToken::from_text(text, pos)).map(Token::from)
                } else {
                    track!(tokens::IntegerToken::from_text(text, pos)).map(Token::from)
                }
            }
            '$' => track!(tokens::CharToken::from_text(text, pos)).map(Token::from),
            '"' => track!(tokens::StringToken::from_text(text, pos)).map(Token::from),
            '\'' => track!(tokens::AtomToken::from_text(text, pos)).map(Token::from),
            '%' => track!(tokens::CommentToken::from_text(text, pos)).map(Token::from),
            _ => {
                if head.is_alphabetic() {
                    let atom = track_try!(tokens::AtomToken::from_text(text, pos.clone()));
                    if let Ok(keyword) = tokens::KeywordToken::from_text(atom.text(), pos) {
                        Ok(Token::from(keyword))
                    } else {
                        Ok(Token::from(atom))
                    }
                } else {
                    track!(tokens::SymbolToken::from_text(text, pos)).map(Token::from)
                }
            }
        }
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::{Token, TokenValue, Position};
    ///
    /// let pos = Position::new();
    ///
    /// // Comment
    /// assert_eq!(Token::from_text("% foo", pos.clone()).unwrap().value(),
    ///            TokenValue::Comment(" foo"));
    ///
    /// // Float
    /// assert_eq!(Token::from_text("1.23", pos.clone()).unwrap().value(),
    ///            TokenValue::Float(1.23));
    /// ```
    pub fn value(&self) -> TokenValue {
        match *self {
            Token::Atom(ref t) => TokenValue::Atom(t.value()),
            Token::Char(ref t) => TokenValue::Char(t.value()),
            Token::Comment(ref t) => TokenValue::Comment(t.value()),
            Token::Float(ref t) => TokenValue::Float(t.value()),
            Token::Integer(ref t) => TokenValue::Integer(t.value()),
            Token::Keyword(ref t) => TokenValue::Keyword(t.value()),
            Token::String(ref t) => TokenValue::String(t.value()),
            Token::Symbol(ref t) => TokenValue::Symbol(t.value()),
            Token::Variable(ref t) => TokenValue::Variable(t.value()),
            Token::Whitespace(ref t) => TokenValue::Whitespace(t.value()),
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

    /// Returns the kind of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::{Token, TokenKind, Position};
    ///
    /// let pos = Position::new();
    /// assert_eq!(Token::from_text("foo", pos.clone()).unwrap().kind(), TokenKind::Atom);
    /// assert_eq!(Token::from_text("123", pos.clone()).unwrap().kind(), TokenKind::Integer);
    /// assert_eq!(Token::from_text(" ", pos.clone()).unwrap().kind(), TokenKind::Whitespace);
    /// ```
    pub fn kind(&self) -> TokenKind {
        match *self {
            Token::Atom(_) => TokenKind::Atom,
            Token::Char(_) => TokenKind::Char,
            Token::Comment(_) => TokenKind::Comment,
            Token::Float(_) => TokenKind::Float,
            Token::Integer(_) => TokenKind::Integer,
            Token::Keyword(_) => TokenKind::Keyword,
            Token::String(_) => TokenKind::String,
            Token::Symbol(_) => TokenKind::Symbol,
            Token::Variable(_) => TokenKind::Variable,
            Token::Whitespace(_) => TokenKind::Whitespace,
        }
    }
}
impl From<tokens::AtomToken> for Token {
    fn from(f: tokens::AtomToken) -> Self {
        Token::Atom(f)
    }
}
impl From<tokens::CharToken> for Token {
    fn from(f: tokens::CharToken) -> Self {
        Token::Char(f)
    }
}
impl From<tokens::CommentToken> for Token {
    fn from(f: tokens::CommentToken) -> Self {
        Token::Comment(f)
    }
}
impl From<tokens::FloatToken> for Token {
    fn from(f: tokens::FloatToken) -> Self {
        Token::Float(f)
    }
}
impl From<tokens::IntegerToken> for Token {
    fn from(f: tokens::IntegerToken) -> Self {
        Token::Integer(f)
    }
}
impl From<tokens::KeywordToken> for Token {
    fn from(f: tokens::KeywordToken) -> Self {
        Token::Keyword(f)
    }
}
impl From<tokens::StringToken> for Token {
    fn from(f: tokens::StringToken) -> Self {
        Token::String(f)
    }
}
impl From<tokens::SymbolToken> for Token {
    fn from(f: tokens::SymbolToken) -> Self {
        Token::Symbol(f)
    }
}
impl From<tokens::VariableToken> for Token {
    fn from(f: tokens::VariableToken) -> Self {
        Token::Variable(f)
    }
}
impl From<tokens::WhitespaceToken> for Token {
    fn from(f: tokens::WhitespaceToken) -> Self {
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

/// Token kind.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Atom,
    Char,
    Comment,
    Float,
    Integer,
    Keyword,
    String,
    Symbol,
    Variable,
    Whitespace,
}

/// Token value.
#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub enum TokenValue<'a> {
    Atom(&'a str),
    Char(char),
    Comment(&'a str),
    Float(f64),
    Integer(&'a BigUint),
    Keyword(Keyword),
    String(&'a str),
    Symbol(Symbol),
    Variable(&'a str),
    Whitespace(Whitespace),
}
