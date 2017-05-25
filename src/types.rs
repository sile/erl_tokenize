//! Miscellaneous types.
use std::str::FromStr;

use {Result, Error, ErrorKind};

/// Keyword (a.k.a., reserved word).
///
/// Reference: [Erlang's Reserved Words][Reserved Words]
///
/// [Reserved Words]: http://erlang.org/doc/reference_manual/introduction.html#id61721
///
/// # Examples
///
/// ```
/// use erl_tokenize::types::Keyword;
///
/// assert_eq!("bor".parse().ok(), Some(Keyword::Bor));
/// assert!("foo".parse::<Keyword>().is_err());
/// ```
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
}
impl Keyword {
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
impl FromStr for Keyword {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "after" => Ok(Keyword::After),
            "and" => Ok(Keyword::And),
            "andalso" => Ok(Keyword::Andalso),
            "band" => Ok(Keyword::Band),
            "begin" => Ok(Keyword::Begin),
            "bnot" => Ok(Keyword::Bnot),
            "bor" => Ok(Keyword::Bor),
            "bsl" => Ok(Keyword::Bsl),
            "bsr" => Ok(Keyword::Bsr),
            "bxor" => Ok(Keyword::Bxor),
            "case" => Ok(Keyword::Case),
            "catch" => Ok(Keyword::Catch),
            "cond" => Ok(Keyword::Cond),
            "div" => Ok(Keyword::Div),
            "end" => Ok(Keyword::End),
            "fun" => Ok(Keyword::Fun),
            "if" => Ok(Keyword::If),
            "let" => Ok(Keyword::Let),
            "not" => Ok(Keyword::Not),
            "of" => Ok(Keyword::Of),
            "or" => Ok(Keyword::Or),
            "orelse" => Ok(Keyword::Orelse),
            "receive" => Ok(Keyword::Receive),
            "rem" => Ok(Keyword::Rem),
            "try" => Ok(Keyword::Try),
            "when" => Ok(Keyword::When),
            "xor" => Ok(Keyword::Xor),
            _ => track_panic!(ErrorKind::InvalidInput, "Undefined keyword: {:?}", s),
        }
    }
}
