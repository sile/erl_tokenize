use types::Location;

use tokens;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub location: Location,
    pub value: TokenValue,
}
impl Token {
    pub fn new<T: Into<TokenValue>>(location: Location, value: T) -> Self {
        Token {
            location,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Symbol(tokens::Symbol),
    Keyword(tokens::Keyword),
    Int(tokens::Int),
    Float(tokens::Float),
    Char(tokens::Char),
    Var(tokens::Var),
    Atom(tokens::Atom),
    Str(tokens::Str),
    Comment(tokens::Comment),
    Space(tokens::Space),
}
impl From<tokens::Symbol> for TokenValue {
    fn from(f: tokens::Symbol) -> Self {
        TokenValue::Symbol(f)
    }
}
impl From<tokens::Keyword> for TokenValue {
    fn from(f: tokens::Keyword) -> Self {
        TokenValue::Keyword(f)
    }
}
impl From<tokens::Int> for TokenValue {
    fn from(f: tokens::Int) -> Self {
        TokenValue::Int(f)
    }
}
impl From<tokens::Float> for TokenValue {
    fn from(f: tokens::Float) -> Self {
        TokenValue::Float(f)
    }
}
impl From<tokens::Char> for TokenValue {
    fn from(f: tokens::Char) -> Self {
        TokenValue::Char(f)
    }
}
impl From<tokens::Var> for TokenValue {
    fn from(f: tokens::Var) -> Self {
        TokenValue::Var(f)
    }
}
impl From<tokens::Atom> for TokenValue {
    fn from(f: tokens::Atom) -> Self {
        TokenValue::Atom(f)
    }
}
impl From<tokens::Str> for TokenValue {
    fn from(f: tokens::Str) -> Self {
        TokenValue::Str(f)
    }
}
impl From<tokens::Comment> for TokenValue {
    fn from(f: tokens::Comment) -> Self {
        TokenValue::Comment(f)
    }
}
impl From<tokens::Space> for TokenValue {
    fn from(f: tokens::Space) -> Self {
        TokenValue::Space(f)
    }
}
