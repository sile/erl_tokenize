//! Tokens.
use std::str;
use std::borrow::Cow;

use {Result, ErrorKind};
use misc;

/// Atom token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::AtomToken;
///
/// // Ok
/// assert_eq!(AtomToken::from_text("foo").unwrap().value(), "foo");
/// assert_eq!(AtomToken::from_text("foo  ").unwrap().value(), "foo");
/// assert_eq!(AtomToken::from_text("'foo'").unwrap().value(), "foo");
/// assert_eq!(AtomToken::from_text(r"'f\x6Fo'").unwrap().value(), "foo");
///
/// // Err
/// assert!(AtomToken::from_text("  foo").is_err());
/// assert!(AtomToken::from_text("123").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct AtomToken<'a> {
    value: Cow<'a, str>,
    text: &'a str,
}
impl<'a> AtomToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        track_assert!(!text.is_empty(), ErrorKind::InvalidInput);
        let (head, tail) = text.split_at(1);
        let (value, text) = if head == "'" {
            let (value, end) = track_try!(misc::parse_string(tail, '\''));
            (value, unsafe { text.slice_unchecked(0, 1 + end + 1) })
        } else {
            let head = head.chars().nth(0).expect("Never fails");
            track_assert!(misc::is_atom_head_char(head), ErrorKind::InvalidInput);
            let end = head.len_utf8() +
                      tail.find(|c| !misc::is_atom_non_head_char(c))
                          .unwrap_or(tail.len());
            let text_slice = unsafe { text.slice_unchecked(0, end) };
            (Cow::Borrowed(text_slice), text_slice)
        };
        Ok(AtomToken { value, text })
    }
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
    pub fn text(&self) -> &str {
        self.text
    }
}

/// Character token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::CharToken;
///
/// // Ok
/// assert_eq!(CharToken::from_text("$a").unwrap().value(), 'a');
/// assert_eq!(CharToken::from_text("$a  ").unwrap().value(), 'a');
/// assert_eq!(CharToken::from_text(r"$\t").unwrap().value(), '\t');
/// assert_eq!(CharToken::from_text(r"$\123").unwrap().value(), 'I');
/// assert_eq!(CharToken::from_text(r"$\x6F").unwrap().value(), 'o');
/// assert_eq!(CharToken::from_text(r"$\x{06F}").unwrap().value(), 'o');
/// assert_eq!(CharToken::from_text(r"$\^a").unwrap().value(), '\u{1}');
///
/// // Err
/// assert!(CharToken::from_text("  $a").is_err());
/// assert!(CharToken::from_text(r"$\").is_err());
/// assert!(CharToken::from_text("a").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct CharToken<'a> {
    value: char,
    text: &'a str,
}
impl<'a> CharToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        let mut chars = text.char_indices();
        track_assert_eq!(chars.next().map(|(_, c)| c),
                         Some('$'),
                         ErrorKind::InvalidInput);

        let (_, c) = track_try!(chars.next().ok_or(ErrorKind::UnexpectedEos));
        let (value, end) = if c == '\\' {
            let mut chars = chars.peekable();
            let value = track_try!(misc::parse_escaped_char(&mut chars));
            let end = chars.next().map(|(i, _)| i).unwrap_or(text.len());
            (value, end)
        } else {
            let value = c;
            let end = chars.next().map(|(i, _)| i).unwrap_or(text.len());
            (value, end)
        };
        let text = unsafe { text.slice_unchecked(0, end) };
        Ok(CharToken { value, text })
    }
    pub fn value(&self) -> char {
        self.value
    }
    pub fn text(&self) -> &str {
        self.text
    }
}

/// Comment token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::CommentToken;
///
/// // Ok
/// assert_eq!(CommentToken::from_text("%").unwrap().value(), "");
/// assert_eq!(CommentToken::from_text("%% foo ").unwrap().value(), "% foo ");
///
/// // Err
/// assert!(CommentToken::from_text("  % foo").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct CommentToken<'a> {
    text: &'a str,
}
impl<'a> CommentToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        track_assert_eq!(text.chars().nth(0), Some('%'), ErrorKind::InvalidInput);
        let end = text.find('\n').unwrap_or(text.len());
        let text = unsafe { text.slice_unchecked(0, end) };
        Ok(CommentToken { text })
    }
    pub fn value(&self) -> &str {
        unsafe { self.text.slice_unchecked(1, self.text.len()) }
    }
    pub fn text(&self) -> &str {
        self.text
    }
}

/// Floating point number token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::FloatToken;
///
/// // Ok
/// assert_eq!(FloatToken::from_text("0.1").unwrap().value(), 0.1);
/// assert_eq!(FloatToken::from_text("12.3e-1  ").unwrap().value(), 1.23);
///
/// // Err
/// assert!(FloatToken::from_text(".123").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct FloatToken<'a> {
    value: f64,
    text: &'a str,
}
impl<'a> FloatToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        let mut chars = text.char_indices().peekable();

        while let Some((_, '0'...'9')) = chars.peek().cloned() {
            let _ = chars.next();
        }
        track_assert_ne!(chars.peek().map(|&(i, _)| i),
                         Some(0),
                         ErrorKind::InvalidInput);

        if let Some((_, '.')) = chars.peek().cloned() {
            let _ = chars.next();
        }
        while let Some((_, '0'...'9')) = chars.peek().cloned() {
            let _ = chars.next();
        }
        if let Some((_, 'e')) = chars.peek().cloned() {
            let _ = chars.next();
        }
        if let Some((_, 'E')) = chars.peek().cloned() {
            let _ = chars.next();
        }
        if let Some((_, '+')) = chars.peek().cloned() {
            let _ = chars.next();
        }
        if let Some((_, '-')) = chars.peek().cloned() {
            let _ = chars.next();
        }
        while let Some((_, '0'...'9')) = chars.peek().cloned() {
            let _ = chars.next();
        }

        let end = chars.next().map(|(i, _)| i).unwrap_or(text.len());
        let text = unsafe { text.slice_unchecked(0, end) };
        let value = track_try!(text.parse());
        Ok(FloatToken { value, text })
    }
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn text(&self) -> &str {
        self.text
    }
}


// /// Variable token.
// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub struct Var(pub String);
// impl Deref for Var {
//     type Target = str;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// /// String token.
// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub struct Str(pub String);
// impl Deref for Str {
//     type Target = str;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// /// White space token.
// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub enum Whitespace {
//     /// `' '`
//     Space,

//     /// `'\t'`
//     Tab,

//     /// `'\r'`
//     Return,

//     /// `'\n'`
//     Newline,

//     /// `'\u{A0}'`
//     NoBreakSpace,
// }
// impl Whitespace {
//     /// Coverts to the corresponding character.
//     pub fn as_char(&self) -> char {
//         match *self {
//             Whitespace::Space => ' ',
//             Whitespace::Tab => '\t',
//             Whitespace::Return => '\r',
//             Whitespace::Newline => '\n',
//             Whitespace::NoBreakSpace => '\u{A0}',
//         }
//     }
// }

// /// Integer token.
// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub struct Int(pub BigUint);
// impl Deref for Int {
//     type Target = BigUint;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// /// Symbol token.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum Symbol {
//     /// `[`
//     OpenSquare,

//     /// `]`
//     CloseSquare,

//     /// `(`
//     OpenParen,

//     /// `)`
//     CloseParen,

//     /// `{`
//     OpenBrace,

//     /// `}`
//     CloseBrace,

//     /// `#`
//     Sharp,

//     /// `/`
//     Slash,

//     /// ``
//     Dot,

//     /// `,`
//     Comma,

//     /// `:`
//     Colon,

//     /// `;`
//     Semicolon,

//     /// `=`
//     Match,

//     /// `:=`
//     MapMatch,

//     /// `|`
//     VerticalBar,

//     /// `||`
//     DoubleVerticalBar,

//     /// `?`
//     Question,

//     /// `!`
//     Not,

//     /// `-`
//     Hyphen,

//     /// `--`
//     MinusMinus,

//     /// `+`
//     Plus,

//     /// `++`
//     PlusPlus,

//     /// `*`
//     Multiply,

//     /// `->`
//     RightAllow,

//     /// `<-`
//     LeftAllow,

//     /// `=>`
//     DoubleRightAllow,

//     /// `<=`
//     DoubleLeftAllow,

//     /// `>>`
//     DoubleRightAngle,

//     /// `<<`
//     DoubleLeftAngle,

//     /// `==`
//     Eq,

//     /// `=:=`
//     ExactEq,

//     /// `/=`
//     NotEq,

//     /// `=/=`
//     ExactNotEq,

//     /// `>`
//     Greater,

//     /// `>=`
//     GreaterEq,

//     /// `<`
//     Less,

//     /// `=<`
//     LessEq,
// }

// /// Keyword tokens.
// ///
// /// Reference: [Erlang's Reserved Words][Reserved Words]
// ///
// /// [Reserved Words]: http://erlang.org/doc/reference_manual/introduction.html#id61721
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum Keyword {
//     /// `after`
//     After,

//     /// `and`.
//     And,

//     /// `andalso`.
//     Andalso,

//     /// `band`.
//     Band,

//     /// `begin`.
//     Begin,

//     /// `bnot`.
//     Bnot,

//     /// `bor`.
//     Bor,

//     /// `bsl`.
//     Bsl,

//     /// `bsr`.
//     Bsr,

//     /// `bxor`.
//     Bxor,

//     /// `case`.
//     Case,

//     /// `catch`.
//     Catch,

//     /// `cond`.
//     Cond,

//     /// `div`.
//     Div,

//     /// `end`.
//     End,

//     /// `fun`.
//     Fun,

//     /// `if`.
//     If,

//     /// `let`.
//     Let,

//     /// `not`.
//     Not,

//     /// `of`.
//     Of,

//     /// `or`.
//     Or,

//     /// `orelse`.
//     Orelse,

//     /// `receive`.
//     Receive,

//     /// `rem`.
//     Rem,

//     /// `try`.
//     Try,

//     /// `when`.
//     When,

//     /// `xor`.
//     Xor,
// }
// impl Keyword {
//     /// Tries to convert from a string to a `Keyword` instance.
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use erl_tokenize::tokens::Keyword;
//     ///
//     /// assert_eq!(Keyword::from_str("bor"), Some(Keyword::Bor));
//     /// assert_eq!(Keyword::from_str("foo"), None);
//     /// ```
//     pub fn from_str(s: &str) -> Option<Self> {
//         match s {
//             "after" => Some(Keyword::After),
//             "and" => Some(Keyword::And),
//             "andalso" => Some(Keyword::Andalso),
//             "band" => Some(Keyword::Band),
//             "begin" => Some(Keyword::Begin),
//             "bnot" => Some(Keyword::Bnot),
//             "bor" => Some(Keyword::Bor),
//             "bsl" => Some(Keyword::Bsl),
//             "bsr" => Some(Keyword::Bsr),
//             "bxor" => Some(Keyword::Bxor),
//             "case" => Some(Keyword::Case),
//             "catch" => Some(Keyword::Catch),
//             "cond" => Some(Keyword::Cond),
//             "div" => Some(Keyword::Div),
//             "end" => Some(Keyword::End),
//             "fun" => Some(Keyword::Fun),
//             "if" => Some(Keyword::If),
//             "let" => Some(Keyword::Let),
//             "not" => Some(Keyword::Not),
//             "of" => Some(Keyword::Of),
//             "or" => Some(Keyword::Or),
//             "orelse" => Some(Keyword::Orelse),
//             "receive" => Some(Keyword::Receive),
//             "rem" => Some(Keyword::Rem),
//             "try" => Some(Keyword::Try),
//             "when" => Some(Keyword::When),
//             "xor" => Some(Keyword::Xor),
//             _ => None,
//         }
//     }

//     /// Returns the string representation of this keyword.
//     pub fn as_str(&self) -> &'static str {
//         match *self {
//             Keyword::After => "after",
//             Keyword::And => "and",
//             Keyword::Andalso => "andalso",
//             Keyword::Band => "band",
//             Keyword::Begin => "begin",
//             Keyword::Bnot => "bnot",
//             Keyword::Bor => "bor",
//             Keyword::Bsl => "bsl",
//             Keyword::Bsr => "bsr",
//             Keyword::Bxor => "bxor",
//             Keyword::Case => "case",
//             Keyword::Catch => "catch",
//             Keyword::Cond => "cond",
//             Keyword::Div => "div",
//             Keyword::End => "end",
//             Keyword::Fun => "fun",
//             Keyword::If => "if",
//             Keyword::Let => "let",
//             Keyword::Not => "not",
//             Keyword::Of => "of",
//             Keyword::Or => "or",
//             Keyword::Orelse => "orelse",
//             Keyword::Receive => "receive",
//             Keyword::Rem => "rem",
//             Keyword::Try => "try",
//             Keyword::When => "when",
//             Keyword::Xor => "xor",
//         }
//     }
// }
