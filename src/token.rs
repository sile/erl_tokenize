use types::Location;

use tokens;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub location: Location,
    pub value: TokenValue,
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
}
