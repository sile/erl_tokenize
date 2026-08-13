/// Erlang reserved word.
///
/// Reference: [Erlang's Reserved Words][Reserved Words]
///
/// [Reserved Words]: http://erlang.org/doc/reference_manual/introduction.html#id61721
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    /// `after`
    After,

    /// `and`
    And,

    /// `andalso`
    Andalso,

    /// `band`
    Band,

    /// `begin`
    Begin,

    /// `bnot`
    Bnot,

    /// `bor`
    Bor,

    /// `bsl`
    Bsl,

    /// `bsr`
    Bsr,

    /// `bxor`
    Bxor,

    /// `case`
    Case,

    /// `catch`
    Catch,

    /// `cond`
    Cond,

    /// `div`
    Div,

    /// `end`
    End,

    /// `fun`
    Fun,

    /// `if`
    If,

    /// `let`
    Let,

    /// `not`
    Not,

    /// `of`
    Of,

    /// `or`
    Or,

    /// `orelse`
    Orelse,

    /// `receive`
    Receive,

    /// `rem`
    Rem,

    /// `try`
    Try,

    /// `when`
    When,

    /// `xor`
    Xor,

    /// `maybe`
    Maybe,

    /// `else`
    Else,
}
impl Keyword {
    /// Returns the string representation of this keyword.
    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::After => "after",
            Keyword::And => "and",
            Keyword::Andalso => "andalso",
            Keyword::Band => "band",
            Keyword::Begin => "begin",
            Keyword::Bnot => "bnot",
            Keyword::Bor => "bor",
            Keyword::Bsl => "bsl",
            Keyword::Bsr => "bsr",
            Keyword::Bxor => "bxor",
            Keyword::Case => "case",
            Keyword::Catch => "catch",
            Keyword::Cond => "cond",
            Keyword::Div => "div",
            Keyword::End => "end",
            Keyword::Fun => "fun",
            Keyword::If => "if",
            Keyword::Let => "let",
            Keyword::Not => "not",
            Keyword::Of => "of",
            Keyword::Or => "or",
            Keyword::Orelse => "orelse",
            Keyword::Receive => "receive",
            Keyword::Rem => "rem",
            Keyword::Try => "try",
            Keyword::When => "when",
            Keyword::Xor => "xor",
            Keyword::Maybe => "maybe",
            Keyword::Else => "else",
        }
    }
}
