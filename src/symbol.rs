/// Punctuation or operator symbol.
///
/// Each variant corresponds to exactly one textual form. Convert to and
/// from the textual form with [`Symbol::as_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    /// `[`
    OpenSquare,

    /// `]`
    CloseSquare,

    /// `(`
    OpenParen,

    /// `)`
    CloseParen,

    /// `{`
    OpenBrace,

    /// `}`
    CloseBrace,

    /// `#`
    Sharp,

    /// `#_`
    WildcardRecord,

    /// `/`
    Slash,

    /// `.`
    Dot,

    /// `..`
    DoubleDot,

    /// `...`
    TripleDot,

    /// `,`
    Comma,

    /// `:`
    Colon,

    /// `::`
    DoubleColon,

    /// `;`
    Semicolon,

    /// `=`
    Match,

    /// `:=`
    MapMatch,

    /// `|`
    VerticalBar,

    /// `||`
    DoubleVerticalBar,

    /// `?`
    Question,

    /// `?=`
    MaybeMatch,

    /// `!`
    Bang,

    /// `-`
    Hyphen,

    /// `--`
    MinusMinus,

    /// `+`
    Plus,

    /// `++`
    PlusPlus,

    /// `*`
    Multiply,

    /// `->`
    RightArrow,

    /// `<-`
    LeftArrow,

    /// `=>`
    DoubleRightArrow,

    /// `<=`
    DoubleLeftArrow,

    /// `>>`
    DoubleRightAngle,

    /// `<<`
    DoubleLeftAngle,

    /// `==`
    Eq,

    /// `=:=`
    ExactEq,

    /// `/=`
    NotEq,

    /// `=/=`
    ExactNotEq,

    /// `>`
    Greater,

    /// `>=`
    GreaterEq,

    /// `<`
    Less,

    /// `=<`
    LessEq,

    /// `&&`
    DoubleAmpersand,

    /// `<:-`
    StrictLeftArrow,

    /// `<:=`
    StrictDoubleLeftArrow,
}
impl Symbol {
    /// Returns the textual representation of this symbol.
    pub fn as_str(self) -> &'static str {
        match self {
            Symbol::OpenSquare => "[",
            Symbol::CloseSquare => "]",
            Symbol::OpenParen => "(",
            Symbol::CloseParen => ")",
            Symbol::OpenBrace => "{",
            Symbol::CloseBrace => "}",
            Symbol::Sharp => "#",
            Symbol::WildcardRecord => "#_",
            Symbol::Slash => "/",
            Symbol::Dot => ".",
            Symbol::DoubleDot => "..",
            Symbol::TripleDot => "...",
            Symbol::Comma => ",",
            Symbol::Colon => ":",
            Symbol::DoubleColon => "::",
            Symbol::Semicolon => ";",
            Symbol::Match => "=",
            Symbol::MapMatch => ":=",
            Symbol::VerticalBar => "|",
            Symbol::DoubleVerticalBar => "||",
            Symbol::Question => "?",
            Symbol::Bang => "!",
            Symbol::Hyphen => "-",
            Symbol::MinusMinus => "--",
            Symbol::Plus => "+",
            Symbol::PlusPlus => "++",
            Symbol::Multiply => "*",
            Symbol::RightArrow => "->",
            Symbol::LeftArrow => "<-",
            Symbol::DoubleRightArrow => "=>",
            Symbol::DoubleLeftArrow => "<=",
            Symbol::DoubleRightAngle => ">>",
            Symbol::DoubleLeftAngle => "<<",
            Symbol::Eq => "==",
            Symbol::ExactEq => "=:=",
            Symbol::NotEq => "/=",
            Symbol::ExactNotEq => "=/=",
            Symbol::Greater => ">",
            Symbol::GreaterEq => ">=",
            Symbol::Less => "<",
            Symbol::LessEq => "=<",
            Symbol::MaybeMatch => "?=",
            Symbol::DoubleAmpersand => "&&",
            Symbol::StrictLeftArrow => "<:-",
            Symbol::StrictDoubleLeftArrow => "<:=",
        }
    }
}
