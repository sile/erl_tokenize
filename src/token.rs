use tokens;

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Atom(tokens::AtomToken<'a>),
    Char(tokens::CharToken<'a>),
    // Comment(tokens::CommentToken),
    // Float(tokens::FloatToken),
    // Integer(tokens::IntegerToken),
    // Keyword(tokens::KeywordToken),
    // String(tokens::StringToken),
    // Symbol(tokens::SymbolToken),
    // Variable(tokens::VariableToken),
    // Whitespace(tokens::WhitespaceToken)
}
impl<'a> Token<'a> {
    pub fn text(&self) -> &str {
        match *self {
            Token::Atom(ref t) => t.text(),
            Token::Char(ref t) => t.text(),
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
// impl From<tokens::Symbol> for Token {
//     fn from(f: tokens::Symbol) -> Self {
//         Token::Symbol(f)
//     }
// }
// impl From<tokens::Keyword> for Token {
//     fn from(f: tokens::Keyword) -> Self {
//         Token::Keyword(f)
//     }
// }
// impl From<tokens::Int> for Token {
//     fn from(f: tokens::Int) -> Self {
//         Token::Int(f)
//     }
// }
// impl From<tokens::Float> for Token {
//     fn from(f: tokens::Float) -> Self {
//         Token::Float(f)
//     }
// }
// impl From<tokens::Var> for Token {
//     fn from(f: tokens::Var) -> Self {
//         Token::Var(f)
//     }
// }
// impl From<tokens::Str> for Token {
//     fn from(f: tokens::Str) -> Self {
//         Token::Str(f)
//     }
// }
// impl From<tokens::Comment> for Token {
//     fn from(f: tokens::Comment) -> Self {
//         Token::Comment(f)
//     }
// }
// impl From<tokens::Whitespace> for Token {
//     fn from(f: tokens::Whitespace) -> Self {
//         Token::Whitespace(f)
//     }
// }

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
