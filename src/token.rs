use num::BigUint;

use {Result, ErrorKind};
use tokens;
use values::{Keyword, Symbol, Whitespace};

/// Token.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum Token<'a> {
    Atom(tokens::AtomToken<'a>),
    Char(tokens::CharToken<'a>),
    Comment(tokens::CommentToken<'a>),
    Float(tokens::FloatToken<'a>),
    Integer(tokens::IntegerToken<'a>),
    Keyword(tokens::KeywordToken<'a>),
    String(tokens::StringToken<'a>),
    Symbol(tokens::SymbolToken<'a>),
    Variable(tokens::VariableToken<'a>),
    Whitespace(tokens::WhitespaceToken<'a>),
}
impl<'a> Token<'a> {
    /// Tries to convert from any prefixes of the text to a token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::{Token, TokenValue};
    /// use erl_tokenize::values::Symbol;
    ///
    /// // Atom
    /// assert_eq!(Token::from_text("foo").unwrap().value(), TokenValue::Atom("foo"));
    ///
    /// // Symbol
    /// assert_eq!(Token::from_text("[foo]").unwrap().value(),
    ///            TokenValue::Symbol(Symbol::OpenSquare));
    /// ```
    pub fn from_text(text: &'a str) -> Result<Self> {
        let head = track_try!(text.chars().nth(0).ok_or(ErrorKind::UnexpectedEos));
        match head {
            ' ' | '\t' | '\r' | '\n' | '\u{A0}' => {
                track!(tokens::WhitespaceToken::from_text(text)).map(Token::from)
            }
            'A'...'Z' | '_' => track!(tokens::VariableToken::from_text(text)).map(Token::from),
            '0'...'9' => {
                let maybe_float = text.find(|c: char| !c.is_digit(10))
                    .map(|i| text.as_bytes()[i]) == Some(b'.');
                if maybe_float {
                    track!(tokens::FloatToken::from_text(text)).map(Token::from)
                } else {
                    track!(tokens::IntegerToken::from_text(text)).map(Token::from)
                }
            }
            '$' => track!(tokens::CharToken::from_text(text)).map(Token::from),
            '"' => track!(tokens::StringToken::from_text(text)).map(Token::from),
            '\'' => track!(tokens::AtomToken::from_text(text)).map(Token::from),
            '%' => track!(tokens::CommentToken::from_text(text)).map(Token::from),
            _ => {
                if head.is_alphabetic() {
                    let atom = track_try!(tokens::AtomToken::from_text(text));
                    if let Ok(keyword) = tokens::KeywordToken::from_text(atom.text()) {
                        Ok(Token::from(keyword))
                    } else {
                        Ok(Token::from(atom))
                    }
                } else {
                    track!(tokens::SymbolToken::from_text(text)).map(Token::from)
                }
            }
        }
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::{Token, TokenValue};
    ///
    /// // Comment
    /// assert_eq!(Token::from_text("% foo").unwrap().value(), TokenValue::Comment(" foo"));
    ///
    /// // Float
    /// assert_eq!(Token::from_text("1.23").unwrap().value(), TokenValue::Float(1.23));
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
    /// use erl_tokenize::Token;
    ///
    /// // Comment
    /// assert_eq!(Token::from_text("% foo").unwrap().text(), "% foo");
    ///
    /// // Char
    /// assert_eq!(Token::from_text(r#"$\t"#).unwrap().text(), r#"$\t"#);
    /// ```
    pub fn text(&self) -> &'a str {
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
    /// use erl_tokenize::{Token, TokenKind};
    ///
    /// assert_eq!(Token::from_text("foo").unwrap().kind(), TokenKind::Atom);
    /// assert_eq!(Token::from_text("123").unwrap().kind(), TokenKind::Integer);
    /// assert_eq!(Token::from_text(" ").unwrap().kind(), TokenKind::Whitespace);
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
impl<'a> From<tokens::AtomToken<'a>> for Token<'a> {
    fn from(f: tokens::AtomToken<'a>) -> Self {
        Token::Atom(f)
    }
}
impl<'a> From<tokens::CharToken<'a>> for Token<'a> {
    fn from(f: tokens::CharToken<'a>) -> Self {
        Token::Char(f)
    }
}
impl<'a> From<tokens::CommentToken<'a>> for Token<'a> {
    fn from(f: tokens::CommentToken<'a>) -> Self {
        Token::Comment(f)
    }
}
impl<'a> From<tokens::FloatToken<'a>> for Token<'a> {
    fn from(f: tokens::FloatToken<'a>) -> Self {
        Token::Float(f)
    }
}
impl<'a> From<tokens::IntegerToken<'a>> for Token<'a> {
    fn from(f: tokens::IntegerToken<'a>) -> Self {
        Token::Integer(f)
    }
}
impl<'a> From<tokens::KeywordToken<'a>> for Token<'a> {
    fn from(f: tokens::KeywordToken<'a>) -> Self {
        Token::Keyword(f)
    }
}
impl<'a> From<tokens::StringToken<'a>> for Token<'a> {
    fn from(f: tokens::StringToken<'a>) -> Self {
        Token::String(f)
    }
}
impl<'a> From<tokens::SymbolToken<'a>> for Token<'a> {
    fn from(f: tokens::SymbolToken<'a>) -> Self {
        Token::Symbol(f)
    }
}
impl<'a> From<tokens::VariableToken<'a>> for Token<'a> {
    fn from(f: tokens::VariableToken<'a>) -> Self {
        Token::Variable(f)
    }
}
impl<'a> From<tokens::WhitespaceToken<'a>> for Token<'a> {
    fn from(f: tokens::WhitespaceToken<'a>) -> Self {
        Token::Whitespace(f)
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
