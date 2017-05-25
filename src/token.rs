use tokens;

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
