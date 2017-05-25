//! Tokens.
use std::str;
use std::borrow::Cow;
use num::{Num, BigUint};

use {Result, ErrorKind};
use misc;
use types::{Keyword, Symbol};

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
    pub fn text(&self) -> &'a str {
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
    pub fn text(&self) -> &'a str {
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
    pub fn text(&self) -> &'a str {
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
    pub fn text(&self) -> &'a str {
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
    pub fn text(&self) -> &'a str {
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
        let atom = track_try!(AtomToken::from_text(text));
        let value = match atom.text() {
            "after" => Keyword::After,
            "and" => Keyword::And,
            "andalso" => Keyword::Andalso,
            "band" => Keyword::Band,
            "begin" => Keyword::Begin,
            "bnot" => Keyword::Bnot,
            "bor" => Keyword::Bor,
            "bsl" => Keyword::Bsl,
            "bsr" => Keyword::Bsr,
            "bxor" => Keyword::Bxor,
            "case" => Keyword::Case,
            "catch" => Keyword::Catch,
            "cond" => Keyword::Cond,
            "div" => Keyword::Div,
            "end" => Keyword::End,
            "fun" => Keyword::Fun,
            "if" => Keyword::If,
            "let" => Keyword::Let,
            "not" => Keyword::Not,
            "of" => Keyword::Of,
            "or" => Keyword::Or,
            "orelse" => Keyword::Orelse,
            "receive" => Keyword::Receive,
            "rem" => Keyword::Rem,
            "try" => Keyword::Try,
            "when" => Keyword::When,
            "xor" => Keyword::Xor,
            s => track_panic!(ErrorKind::InvalidInput, "Undefined keyword: {:?}", s),
        };
        let text = atom.text();
        Ok(KeywordToken { value, text })
    }
    pub fn value(&self) -> Keyword {
        self.value
    }
    pub fn text(&self) -> &'a str {
        self.text
    }
}

/// String token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::StringToken;
///
/// // Ok
/// assert_eq!(StringToken::from_text(r#""foo""#).unwrap().value(), "foo");
/// assert_eq!(StringToken::from_text(r#""foo"  "#).unwrap().value(), "foo");
/// assert_eq!(StringToken::from_text(r#""f\x6Fo""#).unwrap().value(), "foo");
///
/// // Err
/// assert!(StringToken::from_text(r#"  "foo""#).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct StringToken<'a> {
    value: Cow<'a, str>,
    text: &'a str,
}
impl<'a> StringToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        track_assert!(!text.is_empty(), ErrorKind::InvalidInput);
        let (head, tail) = text.split_at(1);
        track_assert_eq!(head, "\"", ErrorKind::InvalidInput);
        let (value, end) = track_try!(misc::parse_string(tail, '"'));
        let text = unsafe { text.slice_unchecked(0, 1 + end + 1) };
        Ok(StringToken { value, text })
    }
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
    pub fn text(&self) -> &'a str {
        self.text
    }
}

/// Symbol token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::SymbolToken;
/// use erl_tokenize::types::Symbol;
///
/// // Ok
/// assert_eq!(SymbolToken::from_text(".").unwrap().value(), Symbol::Dot);
/// assert_eq!(SymbolToken::from_text(":=  ").unwrap().value(), Symbol::MapMatch);
///
/// // Err
/// assert!(SymbolToken::from_text("  .").is_err());
/// assert!(SymbolToken::from_text("foo").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct SymbolToken<'a> {
    value: Symbol,
    text: &'a str,
}
impl<'a> SymbolToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        let bytes = text.as_bytes();
        let mut symbol = None;
        if bytes.len() >= 3 {
            symbol = match &bytes[0..3] {
                b"=:=" => Some(Symbol::ExactEq),
                b"=/=" => Some(Symbol::ExactNotEq),
                _ => None,
            };
        }
        if symbol.is_none() && bytes.len() >= 2 {
            symbol = match &bytes[0..2] {
                b":=" => Some(Symbol::MapMatch),
                b"||" => Some(Symbol::DoubleVerticalBar),
                b"--" => Some(Symbol::MinusMinus),
                b"++" => Some(Symbol::PlusPlus),
                b"->" => Some(Symbol::RightAllow),
                b"<-" => Some(Symbol::LeftAllow),
                b"=>" => Some(Symbol::DoubleRightAllow),
                b"<=" => Some(Symbol::DoubleLeftAllow),
                b">>" => Some(Symbol::DoubleRightAngle),
                b"<<" => Some(Symbol::DoubleLeftAngle),
                b"==" => Some(Symbol::Eq),
                b"/=" => Some(Symbol::NotEq),
                b">=" => Some(Symbol::GreaterEq),
                b"=<" => Some(Symbol::LessEq),
                _ => None,
            };
        }
        if symbol.is_none() && bytes.len() >= 1 {
            symbol = match bytes[0] {
                b'[' => Some(Symbol::OpenSquare),
                b']' => Some(Symbol::CloseSquare),
                b'(' => Some(Symbol::OpenParen),
                b')' => Some(Symbol::CloseParen),
                b'{' => Some(Symbol::OpenBrace),
                b'}' => Some(Symbol::CloseBrace),
                b'#' => Some(Symbol::Sharp),
                b'/' => Some(Symbol::Slash),
                b'.' => Some(Symbol::Dot),
                b',' => Some(Symbol::Comma),
                b':' => Some(Symbol::Colon),
                b';' => Some(Symbol::Semicolon),
                b'=' => Some(Symbol::Match),
                b'|' => Some(Symbol::VerticalBar),
                b'?' => Some(Symbol::Question),
                b'!' => Some(Symbol::Not),
                b'-' => Some(Symbol::Hyphen),
                b'+' => Some(Symbol::Plus),
                b'*' => Some(Symbol::Multiply),
                b'>' => Some(Symbol::Greater),
                b'<' => Some(Symbol::Less),
                _ => None,
            };
        }
        if let Some(value) = symbol {
            let end = value.as_str().len();
            let text = unsafe { text.slice_unchecked(0, end) };
            Ok(SymbolToken { value, text })
        } else {
            track_panic!(ErrorKind::InvalidInput);
        }
    }
    pub fn value(&self) -> Symbol {
        self.value
    }
    pub fn text(&self) -> &'a str {
        self.text
    }
}

/// Variable token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::VariableToken;
///
/// // Ok
/// assert_eq!(VariableToken::from_text("Foo").unwrap().value(), "Foo");
/// assert_eq!(VariableToken::from_text("_  ").unwrap().value(), "_");
/// assert_eq!(VariableToken::from_text("_foo@bar").unwrap().value(), "_foo@bar");
///
/// // Err
/// assert!(VariableToken::from_text("foo").is_err());
/// assert!(VariableToken::from_text("  Foo").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct VariableToken<'a> {
    text: &'a str,
}
impl<'a> VariableToken<'a> {
    pub fn from_text(text: &'a str) -> Result<Self> {
        let mut chars = text.char_indices();
        let (_, head) = track_try!(chars.next().ok_or(ErrorKind::InvalidInput));
        track_assert!(misc::is_variable_head_char(head), ErrorKind::InvalidInput);
        let end = chars
            .find(|&(_, c)| !misc::is_variable_non_head_char(c))
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        let text = unsafe { text.slice_unchecked(0, end) };
        Ok(VariableToken { text })
    }
    pub fn value(&self) -> &str {
        self.text
    }
    pub fn text(&self) -> &'a str {
        self.text
    }
}

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
