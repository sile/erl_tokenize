//! Tokens.
use num::{BigUint, Num};
use std::borrow::Cow;
use std::fmt;
use std::str;

use crate::util;
use crate::values::{Keyword, Symbol, Whitespace};
use crate::{Error, Position, PositionRange, Result};

/// Atom token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::AtomToken;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(AtomToken::from_text("foo", pos.clone()).unwrap().value(), "foo");
/// assert_eq!(AtomToken::from_text("foo  ", pos.clone()).unwrap().value(), "foo");
/// assert_eq!(AtomToken::from_text("'foo'", pos.clone()).unwrap().value(), "foo");
/// assert_eq!(AtomToken::from_text(r"'f\x6Fo'", pos.clone()).unwrap().value(), "foo");
///
/// // Err
/// assert!(AtomToken::from_text("  foo", pos.clone()).is_err());
/// assert!(AtomToken::from_text("123", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AtomToken {
    value: Option<String>,
    text: String,
    pos: Position,
}
impl AtomToken {
    /// Makes a new `AtomToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::AtomToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(AtomToken::from_value("foo", pos.clone()).text(), "'foo'");
    /// assert_eq!(AtomToken::from_value("foo's", pos.clone()).text(), r"'foo\'s'");
    /// ```
    pub fn from_value(value: &str, pos: Position) -> Self {
        let mut text = "'".to_string();
        for c in value.chars() {
            match c {
                '\'' => text.push_str("\\'"),
                '\\' => text.push_str("\\\\"),
                _ => text.push(c),
            }
        }
        text.push('\'');
        AtomToken {
            value: Some(value.to_string()),
            text,
            pos,
        }
    }

    /// Tries to convert from any prefixes of the input text to an `AtomToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let head_len = text
            .chars()
            .next()
            .ok_or_else(|| Error::invalid_atom_token(pos.clone()))?
            .len_utf8();
        let (head, tail) = text.split_at(head_len);
        let (value, text) = if head == "'" {
            let (value, end) = util::parse_quotation(pos.clone(), tail, '\'')?;
            let value = Some(value.to_string());
            (value, unsafe { text.get_unchecked(0..=1 + end) })
        } else {
            let head = head.chars().next().expect("unreachable");
            if !util::is_atom_head_char(head) {
                return Err(Error::invalid_atom_token(pos));
            }
            let end = head.len_utf8()
                + tail
                    .find(|c| !util::is_atom_non_head_char(c))
                    .unwrap_or(tail.len());
            let text_slice = unsafe { text.get_unchecked(0..end) };
            (None, text_slice)
        };
        let text = text.to_owned();
        Ok(AtomToken { value, text, pos })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::AtomToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(AtomToken::from_text("foo", pos.clone()).unwrap().value(), "foo");
    /// assert_eq!(AtomToken::from_text("'foo'", pos.clone()).unwrap().value(), "foo");
    /// assert_eq!(AtomToken::from_text(r"'f\x6Fo'", pos.clone()).unwrap().value(), "foo");
    /// ```
    pub fn value(&self) -> &str {
        self.value.as_ref().unwrap_or(&self.text)
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::AtomToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(AtomToken::from_text("foo", pos.clone()).unwrap().text(), "foo");
    /// assert_eq!(AtomToken::from_text("'foo'", pos.clone()).unwrap().text(), "'foo'");
    /// assert_eq!(AtomToken::from_text(r"'f\x6Fo'", pos.clone()).unwrap().text(), r"'f\x6Fo'");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl PositionRange for AtomToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        if self.value.is_none() {
            self.pos.clone().step_by_width(self.text.len())
        } else {
            self.pos.clone().step_by_text(&self.text)
        }
    }
}
impl fmt::Display for AtomToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// Character token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::CharToken;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(CharToken::from_text("$a", pos.clone()).unwrap().value(), 'a');
/// assert_eq!(CharToken::from_text("$a  ", pos.clone()).unwrap().value(), 'a');
/// assert_eq!(CharToken::from_text(r"$\t", pos.clone()).unwrap().value(), '\t');
/// assert_eq!(CharToken::from_text(r"$\123", pos.clone()).unwrap().value(), 'I');
/// assert_eq!(CharToken::from_text(r"$\x6F", pos.clone()).unwrap().value(), 'o');
/// assert_eq!(CharToken::from_text(r"$\x{06F}", pos.clone()).unwrap().value(), 'o');
/// assert_eq!(CharToken::from_text(r"$\^a", pos.clone()).unwrap().value(), '\u{1}');
///
/// // Err
/// assert!(CharToken::from_text("  $a", pos.clone()).is_err());
/// assert!(CharToken::from_text(r"$\", pos.clone()).is_err());
/// assert!(CharToken::from_text("a", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharToken {
    value: char,
    text: String,
    pos: Position,
}
impl CharToken {
    /// Makes a new `CharToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::CharToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(CharToken::from_value('a', pos.clone()).text(), "$a");
    /// ```
    pub fn from_value(value: char, pos: Position) -> Self {
        let text = if value == '\\' {
            r"$\\".to_string()
        } else {
            format!("${}", value)
        };
        CharToken { value, text, pos }
    }

    /// Tries to convert from any prefixes of the text to a `CharToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let mut chars = text.char_indices();
        if chars.next().map(|(_, c)| c) != Some('$') {
            return Err(Error::invalid_char_token(pos));
        }

        let (_, c) = chars
            .next()
            .ok_or_else(|| Error::invalid_char_token(pos.clone()))?;
        let (value, end) = if c == '\\' {
            let mut chars = chars.peekable();
            let value = util::parse_escaped_char(pos.clone(), &mut chars)?;
            let end = chars.next().map(|(i, _)| i).unwrap_or_else(|| text.len());
            (value, end)
        } else {
            let value = c;
            let end = chars.next().map(|(i, _)| i).unwrap_or_else(|| text.len());
            (value, end)
        };
        let text = unsafe { text.get_unchecked(0..end) }.to_owned();
        Ok(CharToken { value, text, pos })
    }

    /// Returns the value of this token.
    ///
    /// # Example
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::CharToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(CharToken::from_text("$a", pos.clone()).unwrap().value(), 'a');
    /// assert_eq!(CharToken::from_text(r"$\123", pos.clone()).unwrap().value(), 'I');
    /// ```
    pub fn value(&self) -> char {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Example
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::CharToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(CharToken::from_text("$a", pos.clone()).unwrap().text(), "$a");
    /// assert_eq!(CharToken::from_text(r"$\123", pos.clone()).unwrap().text(), r#"$\123"#);
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl PositionRange for CharToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_text(&self.text)
    }
}
impl fmt::Display for CharToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// Comment token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::CommentToken;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(CommentToken::from_text("%", pos.clone()).unwrap().value(), "");
/// assert_eq!(CommentToken::from_text("%% foo ", pos.clone()).unwrap().value(), "% foo ");
///
/// // Err
/// assert!(CommentToken::from_text("  % foo", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommentToken {
    text: String,
    pos: Position,
}
impl CommentToken {
    /// Makes a new `CommentToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::CommentToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(CommentToken::from_value("foo", pos.clone()).unwrap().text(), "%foo");
    /// ```
    pub fn from_value(value: &str, pos: Position) -> Result<Self> {
        if value.find('\n').is_some() {
            return Err(Error::invalid_comment_token(pos));
        }

        let text = format!("%{}", value);
        Ok(CommentToken { text, pos })
    }

    /// Tries to convert from any prefixes of the text to a `CommentToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        if !text.starts_with('%') {
            return Err(Error::invalid_comment_token(pos));
        }

        let end = text.find('\n').unwrap_or(text.len());
        let text = unsafe { text.get_unchecked(0..end) }.to_owned();
        Ok(CommentToken { text, pos })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::CommentToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(CommentToken::from_text("%", pos.clone()).unwrap().value(), "");
    /// assert_eq!(CommentToken::from_text("%% foo ", pos.clone()).unwrap().value(), "% foo ");
    /// ```
    pub fn value(&self) -> &str {
        unsafe { self.text().get_unchecked(1..self.text.len()) }
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::CommentToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(CommentToken::from_text("%", pos.clone()).unwrap().text(), "%");
    /// assert_eq!(CommentToken::from_text("%% foo ", pos.clone()).unwrap().text(), "%% foo ");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl PositionRange for CommentToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_width(self.text.len())
    }
}
impl fmt::Display for CommentToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// Floating point number token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::FloatToken;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(FloatToken::from_text("0.1", pos.clone()).unwrap().value(), 0.1);
/// assert_eq!(FloatToken::from_text("12.3e-1  ", pos.clone()).unwrap().value(), 1.23);
/// assert_eq!(FloatToken::from_text("1_2.3_4e-1_0", pos.clone()).unwrap().value(), 0.000000001234);
///
/// // Err
/// assert!(FloatToken::from_text("123", pos.clone()).is_err());
/// assert!(FloatToken::from_text(".123", pos.clone()).is_err());
/// assert!(FloatToken::from_text("1.", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12_.3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12._3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12.3_", pos.clone()).is_err());
/// assert!(FloatToken::from_text("1__2.3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12.3__4", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12.34e-1__0", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FloatToken {
    value: f64,
    text: String,
    pos: Position,
}
impl FloatToken {
    /// Makes a new `FloatToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::FloatToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(FloatToken::from_value(1.23, pos.clone()).text(), "1.23");
    /// ```
    pub fn from_value(value: f64, pos: Position) -> Self {
        let text = format!("{}", value);
        FloatToken { value, text, pos }
    }

    /// Tries to convert from any prefixes of the text to a `FloatToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        fn read_digits(
            buf: &mut String,
            chars: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)>>,
            pos: &Position,
        ) -> Result<()> {
            let mut needs_digit = true;
            while let Some((_, c @ ('0'..='9' | '_'))) = chars.peek().cloned() {
                if c == '_' {
                    if needs_digit {
                        break;
                    }
                    needs_digit = true;
                } else {
                    buf.push(c);
                    needs_digit = false;
                }
                let _ = chars.next();
            }
            if needs_digit {
                Err(Error::invalid_float_token(pos.clone()))
            } else {
                Ok(())
            }
        }

        let mut chars = text.char_indices().peekable();
        let mut buf = String::new();
        read_digits(&mut buf, &mut chars, &pos)?;
        if chars.next().map(|(_, c)| c) != Some('.') {
            return Err(Error::invalid_float_token(pos));
        }
        buf.push('.');

        read_digits(&mut buf, &mut chars, &pos)?;

        if let Some((_, c @ ('e' | 'E'))) = chars.peek().cloned() {
            let _ = chars.next();
            buf.push(c);
            if let Some((_, c @ ('+' | '-'))) = chars.peek().cloned() {
                let _ = chars.next();
                buf.push(c);
            }
            read_digits(&mut buf, &mut chars, &pos)?;
        }

        let end = chars.next().map(|(i, _)| i).unwrap_or_else(|| text.len());
        let text = unsafe { text.get_unchecked(0..end) }.to_owned();
        let value = buf
            .parse()
            .map_err(|_| Error::invalid_float_token(pos.clone()))?;
        Ok(FloatToken { value, text, pos })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::FloatToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(FloatToken::from_text("0.1", pos.clone()).unwrap().value(), 0.1);
    /// assert_eq!(FloatToken::from_text("12.3e-1", pos.clone()).unwrap().value(), 1.23);
    /// ```
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::FloatToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(FloatToken::from_text("0.1", pos.clone()).unwrap().text(), "0.1");
    /// assert_eq!(FloatToken::from_text("12.3e-1", pos.clone()).unwrap().text(), "12.3e-1");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl PositionRange for FloatToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_width(self.text.len())
    }
}
impl fmt::Display for FloatToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// Integer token.
///
/// # Examples
///
/// ```
/// # extern crate num;
/// # extern crate erl_tokenize;
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::IntegerToken;
/// use num::traits::ToPrimitive;
///
/// # fn main() {
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(IntegerToken::from_text("10", pos.clone()).unwrap().value().to_u32(),
///            Some(10u32));
/// assert_eq!(IntegerToken::from_text("123_456", pos.clone()).unwrap().value().to_u32(),
///            Some(123456));
/// assert_eq!(IntegerToken::from_text("16#ab0e", pos.clone()).unwrap().value().to_u32(),
///            Some(0xab0e));
/// assert_eq!(IntegerToken::from_text("1_6#a_b_0e", pos.clone()).unwrap().value().to_u32(),
///            Some(0xab0e));
///
/// // Err
/// assert!(IntegerToken::from_text("-10", pos.clone()).is_err());
/// assert!(IntegerToken::from_text("123_456_", pos.clone()).is_err());
/// assert!(IntegerToken::from_text("123__456", pos.clone()).is_err());
/// # }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntegerToken {
    value: BigUint,
    text: String,
    pos: Position,
}
impl IntegerToken {
    /// Makes a new `IntegerToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::IntegerToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(IntegerToken::from_value(123u32.into(), pos.clone()).text(), "123");
    /// ```
    pub fn from_value(value: BigUint, pos: Position) -> Self {
        let text = format!("{}", value);
        IntegerToken { value, text, pos }
    }

    /// Tries to convert from any prefixes of the text to an `IntegerToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let mut has_radix = false;
        let mut radix = 10;
        let mut chars = text.char_indices().peekable();
        let mut digits = String::new();
        let mut needs_digit = true;
        while let Some((_, c)) = chars.peek().cloned() {
            if c == '#' && !has_radix && !needs_digit {
                radix = digits
                    .parse()
                    .map_err(|_| Error::invalid_integer_token(pos.clone()))?;
                if !(1 < radix && radix < 37) {
                    return Err(Error::invalid_integer_token(pos));
                }
                digits.clear();
                needs_digit = true;
                has_radix = true;
            } else if c.is_digit(radix) {
                digits.push(c);
                needs_digit = false;
            } else if c == '_' && !needs_digit {
                needs_digit = true;
            } else {
                break;
            }
            chars.next();
        }
        if needs_digit {
            return Err(Error::invalid_integer_token(pos));
        }

        let end = chars.peek().map(|&(i, _)| i).unwrap_or_else(|| text.len());
        let value = Num::from_str_radix(&digits, radix)
            .map_err(|_| Error::invalid_integer_token(pos.clone()))?;
        let text = unsafe { text.get_unchecked(0..end) }.to_owned();
        Ok(IntegerToken { value, text, pos })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate num;
    /// # extern crate erl_tokenize;
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::IntegerToken;
    /// use num::traits::ToPrimitive;
    ///
    /// # fn main() {
    /// let pos = Position::new();
    ///
    /// assert_eq!(IntegerToken::from_text("10", pos.clone()).unwrap().value().to_u32(),
    ///            Some(10u32));
    /// assert_eq!(IntegerToken::from_text("16#ab0e", pos.clone()).unwrap().value().to_u32(),
    ///            Some(0xab0e));
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
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::IntegerToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(IntegerToken::from_text("10", pos.clone()).unwrap().text(), "10");
    /// assert_eq!(IntegerToken::from_text("16#ab0e", pos.clone()).unwrap().text(), "16#ab0e");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl PositionRange for IntegerToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_width(self.text.len())
    }
}
impl fmt::Display for IntegerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// Keyword token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::KeywordToken;
/// use erl_tokenize::values::Keyword;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(KeywordToken::from_text("receive", pos.clone()).unwrap().value(), Keyword::Receive);
/// assert_eq!(KeywordToken::from_text("and  ", pos.clone()).unwrap().value(), Keyword::And);
///
/// // Err
/// assert!(KeywordToken::from_text("foo", pos.clone()).is_err());
/// assert!(KeywordToken::from_text("  and", pos.clone()).is_err());
/// assert!(KeywordToken::from_text("andfoo", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeywordToken {
    value: Keyword,
    pos: Position,
}
impl KeywordToken {
    /// Makes a new `KeywordToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::KeywordToken;
    /// use erl_tokenize::values::Keyword;
    ///
    /// let pos = Position::new();
    /// assert_eq!(KeywordToken::from_value(Keyword::Case, pos.clone()).text(), "case");
    /// ```
    pub fn from_value(value: Keyword, pos: Position) -> Self {
        KeywordToken { value, pos }
    }

    /// Tries to convert from any prefixes of the text to a `KeywordToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let atom = AtomToken::from_text(text, pos.clone())?;
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
            "maybe" => Keyword::Maybe,
            "else" => Keyword::Else,
            s => return Err(Error::unknown_keyword(pos, s.to_owned())),
        };
        Ok(KeywordToken { value, pos })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::KeywordToken;
    /// use erl_tokenize::values::Keyword;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(KeywordToken::from_text("receive", pos.clone()).unwrap().value(),
    ///            Keyword::Receive);
    /// assert_eq!(KeywordToken::from_text("and  ", pos.clone()).unwrap().value(),
    ///            Keyword::And);
    /// ```
    pub fn value(&self) -> Keyword {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::KeywordToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(KeywordToken::from_text("receive", pos.clone()).unwrap().text(), "receive");
    /// assert_eq!(KeywordToken::from_text("and  ", pos.clone()).unwrap().text(), "and");
    /// ```
    pub fn text(&self) -> &'static str {
        self.value.as_str()
    }
}
impl PositionRange for KeywordToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_width(self.text().len())
    }
}
impl fmt::Display for KeywordToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// String token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::StringToken;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(StringToken::from_text(r#""foo""#, pos.clone()).unwrap().value(), "foo");
/// assert_eq!(StringToken::from_text(r#""foo"  "#, pos.clone()).unwrap().value(), "foo");
/// assert_eq!(StringToken::from_text(r#""f\x6Fo""#, pos.clone()).unwrap().value(), "foo");
///
/// // Err
/// assert!(StringToken::from_text(r#"  "foo""#, pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StringToken {
    value: Option<String>,
    text: String,
    pos: Position,
}
impl StringToken {
    /// Makes a new `StringToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::StringToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(StringToken::from_value("foo", pos.clone()).text(), r#""foo""#);
    /// ```
    pub fn from_value(value: &str, pos: Position) -> Self {
        let text = format!("{:?}", value);
        StringToken {
            value: Some(value.to_string()),
            text,
            pos,
        }
    }

    /// Tries to convert from any prefixes of the text to a `StringToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        if text.is_empty() {
            return Err(Error::invalid_string_token(pos));
        }

        let (value, end) = if text.starts_with(r#"""""#) {
            // Triple-quoted strings: https://www.erlang.org/eeps/eep-0064
            Self::parse_triple_quoted(text, pos.clone())?
        } else {
            let (head, tail) = text.split_at(1);
            if head != "\"" {
                return Err(Error::invalid_string_token(pos));
            }
            util::parse_quotation(pos.clone(), tail, '"').map(|(v, end)| (v, end + 2))?
        };
        if text.get(end..end + 1) == Some("\"") {
            return Err(Error::adjacent_string_literals(pos));
        }

        let value = match value {
            Cow::Borrowed(_) => None,
            Cow::Owned(v) => Some(v),
        };
        let text = unsafe { text.get_unchecked(0..end) }.to_owned();
        Ok(StringToken { value, text, pos })
    }

    fn parse_triple_quoted(text: &str, pos: Position) -> Result<(Cow<'_, str>, usize)> {
        let mut quote_count = 0;
        let mut chars = text.chars().peekable();
        let mut start_line_end = 0;

        while let Some(c) = chars.peek().copied() {
            if c == '"' {
                quote_count += 1;
                start_line_end += chars.next().expect("unreachable").len_utf8();
            } else {
                break;
            }
        }

        let mut start_line_end_found = false;
        while let Some(c) = chars.next() {
            start_line_end += c.len_utf8();
            if c == '\n' {
                start_line_end_found = true;
                break;
            } else if !c.is_ascii_whitespace() {
                return Err(Error::invalid_string_token(pos));
            }
        }
        if !start_line_end_found {
            return Err(Error::no_closing_quotation(pos));
        }

        let text = &text[start_line_end..];
        let mut indent = 0;
        let mut maybe_end_line = true;
        let mut remaining_quote_count = quote_count;
        let mut end_line_start = 0;
        let mut end_line_end = 0;
        for c in text.chars() {
            end_line_end += c.len_utf8();
            if c == '\n' {
                indent = 0;
                maybe_end_line = true;
                remaining_quote_count = quote_count;
                end_line_start = end_line_end;
            } else if c.is_ascii_whitespace() {
                indent += 1;
            } else if maybe_end_line && c == '"' {
                remaining_quote_count -= 1;
                if remaining_quote_count == 0 {
                    break;
                }
            } else {
                maybe_end_line = false;
            }
        }
        if remaining_quote_count != 0 {
            return Err(Error::no_closing_quotation(pos));
        }

        if indent == 0 {
            return Ok((
                Cow::Owned(format!("\"{}\"", &text[..end_line_start.saturating_sub(1)])),
                end_line_end,
            ));
        }

        let mut value = "\"".to_owned();
        for line in text[..end_line_start - 1].lines() {
            if line == "\n" {
                value.push('\n');
                continue;
            }

            let mut valid_line = false;
            for (i, c) in line.chars().enumerate() {
                if i < indent {
                    if c.is_ascii_whitespace() {
                        continue;
                    } else {
                        return Err(Error::invalid_string_token(pos));
                    }
                }
                value.push(c);
                valid_line = true;
            }
            if !valid_line {
                return Err(Error::invalid_string_token(pos));
            }
        }
        value.push('"');

        Ok((Cow::Owned(value), end_line_end))
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::StringToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(StringToken::from_text(r#""foo""#, pos.clone()).unwrap().value(), "foo");
    /// assert_eq!(StringToken::from_text(r#""foo"  "#, pos.clone()).unwrap().value(), "foo");
    /// assert_eq!(StringToken::from_text(r#""f\x6Fo""#, pos.clone()).unwrap().value(), "foo");
    /// ```
    pub fn value(&self) -> &str {
        if let Some(v) = self.value.as_ref() {
            v
        } else {
            let len = self.text.len();
            unsafe { self.text.get_unchecked(1..len - 1) }
        }
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::StringToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(StringToken::from_text(r#""foo""#, pos.clone()).unwrap().text(),
    ///            r#""foo""#);
    /// assert_eq!(StringToken::from_text(r#""foo"  "#, pos.clone()).unwrap().text(),
    ///            r#""foo""#);
    /// assert_eq!(StringToken::from_text(r#""f\x6Fo""#, pos.clone()).unwrap().text(),
    ///            r#""f\x6Fo""#);
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl PositionRange for StringToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_text(&self.text)
    }
}
impl fmt::Display for StringToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// Symbol token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::SymbolToken;
/// use erl_tokenize::values::Symbol;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(SymbolToken::from_text(".", pos.clone()).unwrap().value(), Symbol::Dot);
/// assert_eq!(SymbolToken::from_text(":=  ", pos.clone()).unwrap().value(), Symbol::MapMatch);
///
/// // Err
/// assert!(SymbolToken::from_text("  .", pos.clone()).is_err());
/// assert!(SymbolToken::from_text("foo", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolToken {
    value: Symbol,
    pos: Position,
}
impl SymbolToken {
    /// Makes a new `SymbolToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::SymbolToken;
    /// use erl_tokenize::values::Symbol;
    ///
    /// let pos = Position::new();
    /// assert_eq!(SymbolToken::from_value(Symbol::Dot, pos.clone()).text(), ".");
    /// ```
    pub fn from_value(value: Symbol, pos: Position) -> Self {
        SymbolToken { value, pos }
    }

    /// Tries to convert from any prefixes of the text to a `SymbolToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let bytes = text.as_bytes();
        let mut symbol = if bytes.len() >= 3 {
            match &bytes[0..3] {
                b"=:=" => Some(Symbol::ExactEq),
                b"=/=" => Some(Symbol::ExactNotEq),
                b"..." => Some(Symbol::TripleDot),
                _ => None,
            }
        } else {
            None
        };
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
                b"?=" => Some(Symbol::MaybeMatch),
                b".." => Some(Symbol::DoubleDot),
                _ => None,
            };
        }
        if symbol.is_none() && !bytes.is_empty() {
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
            Ok(SymbolToken { value, pos })
        } else {
            Err(Error::invalid_symbol_token(pos))
        }
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::SymbolToken;
    /// use erl_tokenize::values::Symbol;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(SymbolToken::from_text(".", pos.clone()).unwrap().value(), Symbol::Dot);
    /// assert_eq!(SymbolToken::from_text(":=  ", pos.clone()).unwrap().value(), Symbol::MapMatch);
    /// ```
    pub fn value(&self) -> Symbol {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::SymbolToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(SymbolToken::from_text(".", pos.clone()).unwrap().text(), ".");
    /// assert_eq!(SymbolToken::from_text(":=  ", pos.clone()).unwrap().text(), ":=");
    /// ```
    pub fn text(&self) -> &'static str {
        self.value.as_str()
    }
}
impl PositionRange for SymbolToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_width(self.text().len())
    }
}
impl fmt::Display for SymbolToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// Variable token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::VariableToken;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(VariableToken::from_text("Foo", pos.clone()).unwrap().value(), "Foo");
/// assert_eq!(VariableToken::from_text("_  ", pos.clone()).unwrap().value(), "_");
/// assert_eq!(VariableToken::from_text("_foo@bar", pos.clone()).unwrap().value(), "_foo@bar");
///
/// // Err
/// assert!(VariableToken::from_text("foo", pos.clone()).is_err());
/// assert!(VariableToken::from_text("  Foo", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableToken {
    text: String,
    pos: Position,
}
impl VariableToken {
    /// Makes a new `VariableToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::VariableToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(VariableToken::from_value("Foo", pos.clone()).unwrap().text(), "Foo");
    /// ```
    pub fn from_value(value: &str, pos: Position) -> Result<Self> {
        let var = Self::from_text(value, pos.clone())?;
        if var.text().len() != value.len() {
            Err(Error::invalid_variable_token(pos))
        } else {
            Ok(var)
        }
    }

    /// Tries to convert from any prefixes of the text to a `VariableToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let mut chars = text.char_indices();
        let (_, head) = chars
            .next()
            .ok_or_else(|| Error::invalid_variable_token(pos.clone()))?;
        if !util::is_variable_head_char(head) {
            return Err(Error::invalid_variable_token(pos));
        }
        let end = chars
            .find(|&(_, c)| !util::is_variable_non_head_char(c))
            .map(|(i, _)| i)
            .unwrap_or_else(|| text.len());
        let text = unsafe { text.get_unchecked(0..end) }.to_owned();
        Ok(VariableToken { text, pos })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::VariableToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(VariableToken::from_text("Foo", pos.clone()).unwrap().value(), "Foo");
    /// assert_eq!(VariableToken::from_text("_foo  ", pos.clone()).unwrap().value(), "_foo");
    /// ```
    pub fn value(&self) -> &str {
        &self.text
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::VariableToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(VariableToken::from_text("Foo", pos.clone()).unwrap().text(), "Foo");
    /// assert_eq!(VariableToken::from_text("_foo  ", pos.clone()).unwrap().text(), "_foo");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }
}
impl PositionRange for VariableToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_width(self.text.len())
    }
}
impl fmt::Display for VariableToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}

/// Whitespace token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::WhitespaceToken;
/// use erl_tokenize::values::Whitespace;
///
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(WhitespaceToken::from_text(" ", pos.clone()).unwrap().value(), Whitespace::Space);
/// assert_eq!(WhitespaceToken::from_text("\t ", pos.clone()).unwrap().value(), Whitespace::Tab);
///
/// // Err
/// assert!(WhitespaceToken::from_text("foo", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhitespaceToken {
    value: Whitespace,
    pos: Position,
}
impl WhitespaceToken {
    /// Makes a new `WhitespaceToken` instance from the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::WhitespaceToken;
    /// use erl_tokenize::values::Whitespace;
    ///
    /// let pos = Position::new();
    /// assert_eq!(WhitespaceToken::from_value(Whitespace::Space, pos.clone()).text(), " ");
    /// ```
    pub fn from_value(value: Whitespace, pos: Position) -> Self {
        WhitespaceToken { value, pos }
    }

    /// Tries to convert from any prefixes of the text to a `WhitespaceToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let value = if let Some(c) = text.chars().next() {
            match c {
                ' ' => Whitespace::Space,
                '\t' => Whitespace::Tab,
                '\r' => Whitespace::Return,
                '\n' => Whitespace::Newline,
                '\u{a0}' => Whitespace::NoBreakSpace,
                _ => return Err(Error::invalid_whitespace_token(pos)),
            }
        } else {
            return Err(Error::invalid_whitespace_token(pos));
        };
        Ok(WhitespaceToken { value, pos })
    }

    /// Returns the value of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::WhitespaceToken;
    /// use erl_tokenize::values::Whitespace;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(WhitespaceToken::from_text(" ", pos.clone()).unwrap().value(),
    ///            Whitespace::Space);
    /// assert_eq!(WhitespaceToken::from_text("\t ", pos.clone()).unwrap().value(),
    ///            Whitespace::Tab);
    /// ```
    pub fn value(&self) -> Whitespace {
        self.value
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::WhitespaceToken;
    ///
    /// let pos = Position::new();
    ///
    /// assert_eq!(WhitespaceToken::from_text(" ", pos.clone()).unwrap().text(), " ");
    /// assert_eq!(WhitespaceToken::from_text("\t ", pos.clone()).unwrap().text(), "\t");
    /// ```
    pub fn text(&self) -> &'static str {
        self.value.as_str()
    }
}
impl PositionRange for WhitespaceToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }
    fn end_position(&self) -> Position {
        self.pos.clone().step_by_text(self.text())
    }
}
impl fmt::Display for WhitespaceToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.text().fmt(f)
    }
}
