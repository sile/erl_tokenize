use tokens;

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Atom(tokens::AtomToken<'a>),
    Char(tokens::CharToken<'a>),
    Comment(tokens::CommentToken<'a>),
    Float(tokens::FloatToken<'a>),
    Integer(tokens::IntegerToken<'a>),
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
            Token::Comment(ref t) => t.text(),
            Token::Float(ref t) => t.text(),
            Token::Integer(ref t) => t.text(),
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
