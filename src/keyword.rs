/// Erlang reserved word.
///
/// Reference: [Erlang's Reserved Words][Reserved Words]
///
/// [Reserved Words]: https://www.erlang.org/doc/system/reference_manual.html#reserved-words
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
    /// Every `Keyword` variant.
    ///
    /// Useful for iteration in tests (Display / round-trip checks),
    /// documentation generation, or building lookup tables.
    // Keep in sync when adding a variant — the compiler will not catch a
    // missing entry (`as_str`'s exhaustive match will, so a fresh variant
    // surfaces there first).
    pub const ALL: &'static [Self] = &[
        Keyword::After,
        Keyword::And,
        Keyword::Andalso,
        Keyword::Band,
        Keyword::Begin,
        Keyword::Bnot,
        Keyword::Bor,
        Keyword::Bsl,
        Keyword::Bsr,
        Keyword::Bxor,
        Keyword::Case,
        Keyword::Catch,
        Keyword::Cond,
        Keyword::Div,
        Keyword::End,
        Keyword::Fun,
        Keyword::If,
        Keyword::Let,
        Keyword::Not,
        Keyword::Of,
        Keyword::Or,
        Keyword::Orelse,
        Keyword::Receive,
        Keyword::Rem,
        Keyword::Try,
        Keyword::When,
        Keyword::Xor,
        Keyword::Maybe,
        Keyword::Else,
    ];

    /// Returns the string representation of this keyword.
    pub const fn as_str(self) -> &'static str {
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

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
