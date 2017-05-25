/// Keyword (a.k.a., reserved word).
///
/// Reference: [Erlang's Reserved Words][Reserved Words]
///
/// [Reserved Words]: http://erlang.org/doc/reference_manual/introduction.html#id61721
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    /// `after`
    After,

    /// `and`.
    And,

    /// `andalso`.
    Andalso,

    /// `band`.
    Band,

    /// `begin`.
    Begin,

    /// `bnot`.
    Bnot,

    /// `bor`.
    Bor,

    /// `bsl`.
    Bsl,

    /// `bsr`.
    Bsr,

    /// `bxor`.
    Bxor,

    /// `case`.
    Case,

    /// `catch`.
    Catch,

    /// `cond`.
    Cond,

    /// `div`.
    Div,

    /// `end`.
    End,

    /// `fun`.
    Fun,

    /// `if`.
    If,

    /// `let`.
    Let,

    /// `not`.
    Not,

    /// `of`.
    Of,

    /// `or`.
    Or,

    /// `orelse`.
    Orelse,

    /// `receive`.
    Receive,

    /// `rem`.
    Rem,

    /// `try`.
    Try,

    /// `when`.
    When,

    /// `xor`.
    Xor,
}
impl Keyword {
    /// Tries to convert from a string to a `Keyword` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::types::Keyword;
    ///
    /// assert_eq!(Keyword::from_str("bor"), Some(Keyword::Bor));
    /// assert_eq!(Keyword::from_str("foo"), None);
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "after" => Some(Keyword::After),
            "and" => Some(Keyword::And),
            "andalso" => Some(Keyword::Andalso),
            "band" => Some(Keyword::Band),
            "begin" => Some(Keyword::Begin),
            "bnot" => Some(Keyword::Bnot),
            "bor" => Some(Keyword::Bor),
            "bsl" => Some(Keyword::Bsl),
            "bsr" => Some(Keyword::Bsr),
            "bxor" => Some(Keyword::Bxor),
            "case" => Some(Keyword::Case),
            "catch" => Some(Keyword::Catch),
            "cond" => Some(Keyword::Cond),
            "div" => Some(Keyword::Div),
            "end" => Some(Keyword::End),
            "fun" => Some(Keyword::Fun),
            "if" => Some(Keyword::If),
            "let" => Some(Keyword::Let),
            "not" => Some(Keyword::Not),
            "of" => Some(Keyword::Of),
            "or" => Some(Keyword::Or),
            "orelse" => Some(Keyword::Orelse),
            "receive" => Some(Keyword::Receive),
            "rem" => Some(Keyword::Rem),
            "try" => Some(Keyword::Try),
            "when" => Some(Keyword::When),
            "xor" => Some(Keyword::Xor),
            _ => None,
        }
    }

    /// Returns the string representation of this keyword.
    pub fn as_str(&self) -> &'static str {
        match *self {
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
        }
    }
}
