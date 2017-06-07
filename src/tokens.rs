//! Tokens.
use std::borrow::Cow;
use std::str;
use num::{Num, BigUint};

use {Result, ErrorKind};
use util;
use values::{Keyword, Symbol, Whitespace};

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
pub struct AtomToken {
    value: Option<String>,
    text: String,
}
impl AtomToken {
    /// Tries to convert from any prefixes of the input text to an `AtomToken`.
    pub fn from_text(text: &str) -> Result<Self> {
        track_assert!(!text.is_empty(), ErrorKind::InvalidInput);
        let (head, tail) = text.split_at(1);
        let (value, text) = if head == "'" {
            let (value, end) = track_try!(util::parse_string(tail, '\''));
            let value = Some(value.to_string());
            (value, unsafe { text.slice_unchecked(0, 1 + end + 1) })
        } else {
            let head = head.chars().nth(0).expect("Never fails");
            track_assert!(util::is_atom_head_char(head), ErrorKind::InvalidInput);
            let end = head.len_utf8() +
                      tail.find(|c| !util::is_atom_non_head_char(c))
                          .unwrap_or(tail.len());
            let text_slice = unsafe { text.slice_unchecked(0, end) };
            (None, text_slice)
        };
        let text = text.to_owned();
        Ok(AtomToken { value, text })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::AtomToken;
    ///
    /// assert_eq!(AtomToken::from_text("foo").unwrap().value(), "foo");
    /// assert_eq!(AtomToken::from_text("'foo'").unwrap().value(), "foo");
    /// assert_eq!(AtomToken::from_text(r"'f\x6Fo'").unwrap().value(), "foo");
    /// ```
    pub fn value(&self) -> &str {
        self.value.as_ref().unwrap_or(&self.text)
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::AtomToken;
    ///
    /// assert_eq!(AtomToken::from_text("foo").unwrap().text(), "foo");
    /// assert_eq!(AtomToken::from_text("'foo'").unwrap().text(), "'foo'");
    /// assert_eq!(AtomToken::from_text(r"'f\x6Fo'").unwrap().text(), r"'f\x6Fo'");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
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
pub struct CharToken {
    value: char,
    text: String,
}
impl CharToken {
    /// Tries to convert from any prefixes of the text to a `CharToken`.
    pub fn from_text(text: &str) -> Result<Self> {
        let mut chars = text.char_indices();
        track_assert_eq!(chars.next().map(|(_, c)| c),
                         Some('$'),
                         ErrorKind::InvalidInput);

        let (_, c) = track_try!(chars.next().ok_or(ErrorKind::UnexpectedEos));
        let (value, end) = if c == '\\' {
            let mut chars = chars.peekable();
            let value = track_try!(util::parse_escaped_char(&mut chars));
            let end = chars.next().map(|(i, _)| i).unwrap_or(text.len());
            (value, end)
        } else {
            let value = c;
            let end = chars.next().map(|(i, _)| i).unwrap_or(text.len());
            (value, end)
        };
        let text = unsafe { text.slice_unchecked(0, end) }.to_owned();
        Ok(CharToken { value, text })
    }

    /// Returns the value of this token.
    ///
    /// # Example
    ///
    /// ```
    /// use erl_tokenize::tokens::CharToken;
    ///
    /// assert_eq!(CharToken::from_text("$a").unwrap().value(), 'a');
    /// assert_eq!(CharToken::from_text(r"$\123").unwrap().value(), 'I');
    /// ```
    pub fn value(&self) -> char {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Example
    ///
    /// ```
    /// use erl_tokenize::tokens::CharToken;
    ///
    /// assert_eq!(CharToken::from_text("$a").unwrap().text(), "$a");
    /// assert_eq!(CharToken::from_text(r"$\123").unwrap().text(), r#"$\123"#);
    /// ```
    pub fn text(&self) -> &str {
        &self.text
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
pub struct CommentToken {
    text: String,
}
impl CommentToken {
    /// Tries to convert from any prefixes of the text to a `CommentToken`.
    pub fn from_text(text: &str) -> Result<Self> {
        track_assert_eq!(text.chars().nth(0), Some('%'), ErrorKind::InvalidInput);
        let end = text.find('\n').unwrap_or(text.len());
        let text = unsafe { text.slice_unchecked(0, end) }.to_owned();
        Ok(CommentToken { text })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::CommentToken;
    ///
    /// assert_eq!(CommentToken::from_text("%").unwrap().value(), "");
    /// assert_eq!(CommentToken::from_text("%% foo ").unwrap().value(), "% foo ");
    /// ```
    pub fn value(&self) -> &str {
        unsafe { self.text().slice_unchecked(1, self.text.len()) }
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::CommentToken;
    ///
    /// assert_eq!(CommentToken::from_text("%").unwrap().text(), "%");
    /// assert_eq!(CommentToken::from_text("%% foo ").unwrap().text(), "%% foo ");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
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
/// assert!(FloatToken::from_text("123").is_err());
/// assert!(FloatToken::from_text(".123").is_err());
/// assert!(FloatToken::from_text("1.").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct FloatToken {
    value: f64,
    text: String,
}
impl FloatToken {
    /// Tries to convert from any prefixes of the text to a `FloatToken`.
    pub fn from_text(text: &str) -> Result<Self> {
        let mut chars = text.char_indices().peekable();

        while let Some((_, '0'...'9')) = chars.peek().cloned() {
            let _ = chars.next();
        }
        track_assert_ne!(chars.peek().map(|&(i, _)| i),
                         Some(0),
                         ErrorKind::InvalidInput);
        track_assert_eq!(chars.next().map(|(_, c)| c),
                         Some('.'),
                         ErrorKind::InvalidInput);
        track_assert_eq!(chars.next().map(|(_, c)| c.is_digit(10)),
                         Some(true),
                         ErrorKind::InvalidInput);

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
        let text = unsafe { text.slice_unchecked(0, end) }.to_owned();
        let value = track_try!(text.parse());
        Ok(FloatToken { value, text })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::FloatToken;
    ///
    /// assert_eq!(FloatToken::from_text("0.1").unwrap().value(), 0.1);
    /// assert_eq!(FloatToken::from_text("12.3e-1").unwrap().value(), 1.23);
    /// ```
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::FloatToken;
    ///
    /// assert_eq!(FloatToken::from_text("0.1").unwrap().text(), "0.1");
    /// assert_eq!(FloatToken::from_text("12.3e-1").unwrap().text(), "12.3e-1");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
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
pub struct IntegerToken {
    value: BigUint,
    text: String,
}
impl IntegerToken {
    /// Tries to convert from any prefixes of the text to an `IntegerToken`.
    pub fn from_text(text: &str) -> Result<Self> {
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
        let text = unsafe { text.slice_unchecked(0, end) }.to_owned();
        Ok(IntegerToken { value, text })
    }

    /// Returns the value of this token.
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
    /// assert_eq!(IntegerToken::from_text("10").unwrap().value().to_u32(), Some(10u32));
    /// assert_eq!(IntegerToken::from_text("16#ab0e").unwrap().value().to_u32(), Some(0xab0e));
    /// # }
    /// ```
    pub fn value(&self) -> &BigUint {
        &self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::IntegerToken;
    ///
    /// assert_eq!(IntegerToken::from_text("10").unwrap().text(), "10");
    /// assert_eq!(IntegerToken::from_text("16#ab0e").unwrap().text(), "16#ab0e");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Keyword token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::KeywordToken;
/// use erl_tokenize::values::Keyword;
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
pub struct KeywordToken {
    value: Keyword,
}
impl KeywordToken {
    /// Tries to convert from any prefixes of the text to a `KeywordToken`.
    pub fn from_text(text: &str) -> Result<Self> {
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
        Ok(KeywordToken { value })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::KeywordToken;
    /// use erl_tokenize::values::Keyword;
    ///
    /// assert_eq!(KeywordToken::from_text("receive").unwrap().value(), Keyword::Receive);
    /// assert_eq!(KeywordToken::from_text("and  ").unwrap().value(), Keyword::And);
    /// ```
    pub fn value(&self) -> Keyword {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::KeywordToken;
    ///
    /// assert_eq!(KeywordToken::from_text("receive").unwrap().text(), "receive");
    /// assert_eq!(KeywordToken::from_text("and  ").unwrap().text(), "and");
    /// ```
    pub fn text(&self) -> &'static str {
        self.value.as_str()
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
pub struct StringToken {
    value: Option<String>,
    text: String,
}
impl StringToken {
    /// Tries to convert from any prefixes of the text to a `StringToken`.
    pub fn from_text(text: &str) -> Result<Self> {
        track_assert!(!text.is_empty(), ErrorKind::InvalidInput);
        let (head, tail) = text.split_at(1);
        track_assert_eq!(head, "\"", ErrorKind::InvalidInput);
        let (value, end) = track_try!(util::parse_string(tail, '"'));
        let value = match value {
            Cow::Borrowed(_) => None,
            Cow::Owned(v) => Some(v),
        };
        let text = unsafe { text.slice_unchecked(0, 1 + end + 1) }.to_owned();
        Ok(StringToken { value, text })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::StringToken;
    ///
    /// assert_eq!(StringToken::from_text(r#""foo""#).unwrap().value(), "foo");
    /// assert_eq!(StringToken::from_text(r#""foo"  "#).unwrap().value(), "foo");
    /// assert_eq!(StringToken::from_text(r#""f\x6Fo""#).unwrap().value(), "foo");
    /// ```
    pub fn value(&self) -> &str {
        if let Some(v) = self.value.as_ref() {
            v
        } else {
            let len = self.text.len();
            unsafe { self.text.slice_unchecked(1, len - 1) }
        }
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::StringToken;
    ///
    /// assert_eq!(StringToken::from_text(r#""foo""#).unwrap().text(), r#""foo""#);
    /// assert_eq!(StringToken::from_text(r#""foo"  "#).unwrap().text(), r#""foo""#);
    /// assert_eq!(StringToken::from_text(r#""f\x6Fo""#).unwrap().text(), r#""f\x6Fo""#);
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Symbol token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::SymbolToken;
/// use erl_tokenize::values::Symbol;
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
pub struct SymbolToken {
    value: Symbol,
}
impl SymbolToken {
    /// Tries to convert from any prefixes of the text to a `SymbolToken`.
    pub fn from_text(text: &str) -> Result<Self> {
        let bytes = text.as_bytes();
        let mut symbol = None;
        if bytes.len() >= 3 {
            symbol = match &bytes[0..3] {
                b"=:=" => Some(Symbol::ExactEq),
                b"=/=" => Some(Symbol::ExactNotEq),
                b"..." => Some(Symbol::TripleDot),
                _ => None,
            };
        }
        if symbol.is_none() && bytes.len() >= 2 {
            symbol = match &bytes[0..2] {
                b"::" => Some(Symbol::DoubleColon),
                b":=" => Some(Symbol::MapMatch),
                b"||" => Some(Symbol::DoubleVerticalBar),
                b"--" => Some(Symbol::MinusMinus),
                b"++" => Some(Symbol::PlusPlus),
                b"->" => Some(Symbol::RightArrow),
                b"<-" => Some(Symbol::LeftArrow),
                b"=>" => Some(Symbol::DoubleRightArrow),
                b"<=" => Some(Symbol::DoubleLeftArrow),
                b">>" => Some(Symbol::DoubleRightAngle),
                b"<<" => Some(Symbol::DoubleLeftAngle),
                b"==" => Some(Symbol::Eq),
                b"/=" => Some(Symbol::NotEq),
                b">=" => Some(Symbol::GreaterEq),
                b"=<" => Some(Symbol::LessEq),
                b"??" => Some(Symbol::DoubleQuestion),
                b".." => Some(Symbol::DoubleDot),
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
            Ok(SymbolToken { value })
        } else {
            track_panic!(ErrorKind::InvalidInput);
        }
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::SymbolToken;
    /// use erl_tokenize::values::Symbol;
    ///
    /// assert_eq!(SymbolToken::from_text(".").unwrap().value(), Symbol::Dot);
    /// assert_eq!(SymbolToken::from_text(":=  ").unwrap().value(), Symbol::MapMatch);
    /// ```
    pub fn value(&self) -> Symbol {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::SymbolToken;
    ///
    /// assert_eq!(SymbolToken::from_text(".").unwrap().text(), ".");
    /// assert_eq!(SymbolToken::from_text(":=  ").unwrap().text(), ":=");
    /// ```
    pub fn text(&self) -> &'static str {
        self.value.as_str()
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
pub struct VariableToken {
    text: String,
}
impl VariableToken {
    /// Tries to convert from any prefixes of the text to a `VariableToken`.
    pub fn from_text(text: &str) -> Result<Self> {
        let mut chars = text.char_indices();
        let (_, head) = track_try!(chars.next().ok_or(ErrorKind::InvalidInput));
        track_assert!(util::is_variable_head_char(head), ErrorKind::InvalidInput);
        let end = chars
            .find(|&(_, c)| !util::is_variable_non_head_char(c))
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        let text = unsafe { text.slice_unchecked(0, end) }.to_owned();
        Ok(VariableToken { text })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::VariableToken;
    ///
    /// assert_eq!(VariableToken::from_text("Foo").unwrap().value(), "Foo");
    /// assert_eq!(VariableToken::from_text("_foo  ").unwrap().value(), "_foo");
    /// ```
    pub fn value(&self) -> &str {
        &self.text
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::VariableToken;
    ///
    /// assert_eq!(VariableToken::from_text("Foo").unwrap().text(), "Foo");
    /// assert_eq!(VariableToken::from_text("_foo  ").unwrap().text(), "_foo");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Whitespace token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::tokens::WhitespaceToken;
/// use erl_tokenize::values::Whitespace;
///
/// // Ok
/// assert_eq!(WhitespaceToken::from_text(" ").unwrap().value(), Whitespace::Space);
/// assert_eq!(WhitespaceToken::from_text("\t ").unwrap().value(), Whitespace::Tab);
///
/// // Err
/// assert!(WhitespaceToken::from_text("foo").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct WhitespaceToken {
    value: Whitespace,
}
impl WhitespaceToken {
    /// Tries to convert from any prefixes of the text to a `WhitespaceToken`.
    pub fn from_text(text: &str) -> Result<Self> {
        track_assert!(!text.is_empty(), ErrorKind::InvalidInput);
        let (text, _) = text.split_at(1);
        let value = match text.as_bytes()[0] {
            b' ' => Whitespace::Space,
            b'\t' => Whitespace::Tab,
            b'\r' => Whitespace::Return,
            b'\n' => Whitespace::Newline,
            0xA0 => Whitespace::NoBreakSpace,
            _ => track_panic!(ErrorKind::InvalidInput, "Not a whitespace: {:?}", text),
        };
        Ok(WhitespaceToken { value })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::WhitespaceToken;
    /// use erl_tokenize::values::Whitespace;
    ///
    /// assert_eq!(WhitespaceToken::from_text(" ").unwrap().value(), Whitespace::Space);
    /// assert_eq!(WhitespaceToken::from_text("\t ").unwrap().value(), Whitespace::Tab);
    /// ```
    pub fn value(&self) -> Whitespace {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::tokens::WhitespaceToken;
    ///
    /// assert_eq!(WhitespaceToken::from_text(" ").unwrap().text(), " ");
    /// assert_eq!(WhitespaceToken::from_text("\t ").unwrap().text(), "\t");
    /// ```
    pub fn text(&self) -> &'static str {
        self.value.as_str()
    }
}
