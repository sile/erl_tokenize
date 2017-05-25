//! Tokens.
use std::str;
use std::borrow::Cow;
use num::{Num, BigUint};

use {Result, ErrorKind};
use misc;
use types::Keyword;

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

/// Integer token.
///
/// # Examples
///
/// ```
/// # extern crate num;
/// # extern crate erl_tokenize;
/// use erl_tokenize::tokens::IntegerToken;
/// use num::traits::ToPrimitive;
///
/// # fn main() {
/// // Ok
/// assert_eq!(IntegerToken::from_text("10").unwrap().value().to_u32(), Some(10u32));
/// assert_eq!(IntegerToken::from_text("16#ab0e").unwrap().value().to_u32(), Some(0xab0e));
///
/// // Err
/// assert!(IntegerToken::from_text("-10").is_err());
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct IntegerToken<'a> {
    value: BigUint,
    text: &'a str,
}
impl<'a> IntegerToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        let mut start = 0;
        let mut radix = 10;
        let mut chars = text.char_indices().peekable();
        while let Some((i, c)) = chars.peek().cloned() {
            if c == '#' && start == 0 {
                start = i + 1;
                radix = track_try!(unsafe { text.slice_unchecked(0, i) }.parse());
                track_assert!(1 < radix && radix < 37,
                              ErrorKind::InvalidInput,
                              "radix={}",
                              radix);
            } else if !c.is_digit(radix) {
                break;
            }
            chars.next();
        }
        let end = chars.peek().map(|&(i, _)| i).unwrap_or(text.len());
        let input = unsafe { text.slice_unchecked(start, end) };
        let value = track_try!(Num::from_str_radix(input, radix),
                               "input={:?}, radix={}",
                               input,
                               radix);
        let text = unsafe { text.slice_unchecked(0, end) };
        Ok(IntegerToken { value, text })
    }
    pub fn value(&self) -> &BigUint {
        &self.value
    }
    pub fn text(&self) -> &str {
        self.text
    }
}

/// Keyword token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::KeywordToken;
/// use erl_tokenize::types::Keyword;
///
/// // Ok
/// assert_eq!(KeywordToken::from_text("receive").unwrap().value(), Keyword::Receive);
/// assert_eq!(KeywordToken::from_text("and  ").unwrap().value(), Keyword::And);
///
/// // Err
/// assert!(KeywordToken::from_text("foo").is_err());
/// assert!(KeywordToken::from_text("  and").is_err());
/// assert!(KeywordToken::from_text("andfoo").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct KeywordToken<'a> {
    value: Keyword,
    text: &'a str,
}
impl<'a> KeywordToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        let end = text.find(|c| if let 'a'...'z' = c { false } else { true })
            .unwrap_or(text.len());
        let text = unsafe { text.slice_unchecked(0, end) };
        let value = track_try!(text.parse());
        Ok(KeywordToken { value, text })
    }
    pub fn value(&self) -> Keyword {
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
