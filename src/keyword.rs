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

    /// Returns the keyword whose textual form matches `text`, or `None`
    /// when `text` is not a reserved word.
    pub const fn from_text(text: &str) -> Option<Self> {
        Some(match text.as_bytes() {
            b"after" => Keyword::After,
            b"and" => Keyword::And,
            b"andalso" => Keyword::Andalso,
            b"band" => Keyword::Band,
            b"begin" => Keyword::Begin,
            b"bnot" => Keyword::Bnot,
            b"bor" => Keyword::Bor,
            b"bsl" => Keyword::Bsl,
            b"bsr" => Keyword::Bsr,
            b"bxor" => Keyword::Bxor,
            b"case" => Keyword::Case,
            b"catch" => Keyword::Catch,
            b"cond" => Keyword::Cond,
            b"div" => Keyword::Div,
            b"end" => Keyword::End,
            b"fun" => Keyword::Fun,
            b"if" => Keyword::If,
            b"let" => Keyword::Let,
            b"not" => Keyword::Not,
            b"of" => Keyword::Of,
            b"or" => Keyword::Or,
            b"orelse" => Keyword::Orelse,
            b"receive" => Keyword::Receive,
            b"rem" => Keyword::Rem,
            b"try" => Keyword::Try,
            b"when" => Keyword::When,
            b"xor" => Keyword::Xor,
            b"maybe" => Keyword::Maybe,
            b"else" => Keyword::Else,
            _ => return None,
        })
    }

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
