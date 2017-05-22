use tokens;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Symbol(tokens::Symbol),
    Keyword(tokens::Keyword),
    Int(tokens::Int),
    Float(tokens::Float),
    Char(tokens::Char),
    Var(tokens::Var),
    Atom(tokens::Atom),
    Str(tokens::Str),
    Comment(tokens::Comment),
    Whitespace(tokens::Whitespace),
}
impl From<tokens::Symbol> for Token {
    fn from(f: tokens::Symbol) -> Self {
        Token::Symbol(f)
    }
}
impl From<tokens::Keyword> for Token {
    fn from(f: tokens::Keyword) -> Self {
        Token::Keyword(f)
    }
}
impl From<tokens::Int> for Token {
    fn from(f: tokens::Int) -> Self {
        Token::Int(f)
    }
}
impl From<tokens::Float> for Token {
    fn from(f: tokens::Float) -> Self {
        Token::Float(f)
    }
}
impl From<tokens::Char> for Token {
    fn from(f: tokens::Char) -> Self {
        Token::Char(f)
    }
}
impl From<tokens::Var> for Token {
    fn from(f: tokens::Var) -> Self {
        Token::Var(f)
    }
}
impl From<tokens::Atom> for Token {
    fn from(f: tokens::Atom) -> Self {
        Token::Atom(f)
    }
}
impl From<tokens::Str> for Token {
    fn from(f: tokens::Str) -> Self {
        Token::Str(f)
    }
}
impl From<tokens::Comment> for Token {
    fn from(f: tokens::Comment) -> Self {
        Token::Comment(f)
    }
}
impl From<tokens::Whitespace> for Token {
    fn from(f: tokens::Whitespace) -> Self {
        Token::Whitespace(f)
    }
}
