//! Tokens.
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
#[derive(Debug, Clone)]
pub struct AtomToken {
    value: Option<String>,
    text: String,
    pos: Position,
}
impl AtomToken {
    /// Makes a new `AtomToken` instance from the value.
    ///
    /// The generated text is a valid Erlang quoted atom which can be
    /// parsed back by [`from_text`](Self::from_text).
    ///
    /// Exception: U+FFFE and U+FFFF are not part of the Erlang character
    /// set (erl_scan rejects them), so the generated text for such values
    /// is not a valid Erlang literal, although [`from_text`](Self::from_text)
    /// can still parse it back.
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
    /// assert_eq!(AtomToken::from_value("a\0b", pos.clone()).text(), r"'a\x{0}b'");
    /// ```
    pub fn from_value(value: &str, pos: Position) -> Self {
        let mut text = String::from("'");
        for c in value.chars() {
            util::push_escaped_char(&mut text, c);
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
        let scanned = crate::lex::scan_atom(text, pos.clone())?;
        let slice = &text[..scanned.len];
        let value = if let Some(inner) = slice.strip_prefix('\'') {
            let (v, _) = util::parse_quotation(pos.clone(), inner, '\'')?;
            Some(v.into_owned())
        } else {
            None
        };
        Ok(AtomToken {
            value,
            text: slice.to_owned(),
            pos,
        })
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
/// assert_eq!(CharToken::from_text(r"$\123", pos.clone()).unwrap().value(), 'S'); // 0o123 = 83 = 'S'
/// assert_eq!(CharToken::from_text(r"$\x6F", pos.clone()).unwrap().value(), 'o');
/// assert_eq!(CharToken::from_text(r"$\x{06F}", pos.clone()).unwrap().value(), 'o');
/// assert_eq!(CharToken::from_text(r"$\^a", pos.clone()).unwrap().value(), '\u{1}');
///
/// // Err
/// assert!(CharToken::from_text("  $a", pos.clone()).is_err());
/// assert!(CharToken::from_text(r"$\", pos.clone()).is_err());
/// assert!(CharToken::from_text("a", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct CharToken {
    value: char,
    text: String,
    pos: Position,
}
impl CharToken {
    /// Makes a new `CharToken` instance from the value.
    ///
    /// The generated text is a valid Erlang character literal which can be
    /// parsed back by [`from_text`](Self::from_text).
    ///
    /// Exception: U+FFFE and U+FFFF are not part of the Erlang character
    /// set (erl_scan rejects them), so the generated text for such values
    /// is not a valid Erlang literal, although [`from_text`](Self::from_text)
    /// can still parse it back.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::CharToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(CharToken::from_value('a', pos.clone()).text(), "$a");
    /// assert_eq!(CharToken::from_value('\n', pos.clone()).text(), r"$\n");
    /// assert_eq!(CharToken::from_value('\0', pos.clone()).text(), r"$\x{0}");
    /// ```
    pub fn from_value(value: char, pos: Position) -> Self {
        let mut text = String::from("$");
        util::push_escaped_char(&mut text, value);
        CharToken { value, text, pos }
    }

    /// Tries to convert from any prefixes of the text to a `CharToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let scanned = crate::lex::scan_char(text, pos.clone())?;
        let slice = &text[..scanned.len];
        let mut chars = slice.char_indices();
        let _ = chars.next();
        let (_, c) = chars.next().expect("scanner validated payload");
        let value = if c == '\\' {
            let mut chars = chars.peekable();
            util::parse_escaped_char(pos.clone(), &mut chars)?
        } else {
            c
        };
        Ok(CharToken {
            value,
            text: slice.to_owned(),
            pos,
        })
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
    /// assert_eq!(CharToken::from_text(r"$\123", pos.clone()).unwrap().value(), 'S'); // 0o123 = 83 = 'S'
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
#[derive(Debug, Clone)]
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

        let text = format!("%{value}");
        Ok(CommentToken { text, pos })
    }

    /// Tries to convert from any prefixes of the text to a `CommentToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let scanned = crate::lex::scan_comment(text, pos.clone())?;
        Ok(CommentToken {
            text: text[..scanned.len].to_owned(),
            pos,
        })
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
/// assert_eq!(FloatToken::from_text("2#0.111", pos.clone()).unwrap().value(), 0.875);
/// assert_eq!(FloatToken::from_text("2#0.10101#e8", pos.clone()).unwrap().value(), 168.0);
/// assert_eq!(FloatToken::from_text("16#f_f.F_F", pos.clone()).unwrap().value(), 255.99609375);
/// assert_eq!(FloatToken::from_text("1_6#fefe.fefe#e1_6", pos.clone()).unwrap().value(), 1.2041849337671418e24);
/// assert_eq!(FloatToken::from_text("32#vrv.vrv#e15", pos.clone()).unwrap().value(), 1.2331041872800477e27);
///
/// // Err
/// assert!(FloatToken::from_text("123", pos.clone()).is_err());
/// assert!(FloatToken::from_text(".123", pos.clone()).is_err());
/// assert!(FloatToken::from_text("10#.123", pos.clone()).is_err());
/// assert!(FloatToken::from_text("1.", pos.clone()).is_err());
/// assert!(FloatToken::from_text("10#1.", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12_.3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("10#12_.3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12._3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("10#12._3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12.3_", pos.clone()).is_err());
/// assert!(FloatToken::from_text("10#12.3_", pos.clone()).is_err());
/// assert!(FloatToken::from_text("1__2.3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("10#1__2.3", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12.3__4", pos.clone()).is_err());
/// assert!(FloatToken::from_text("10#12.3__4", pos.clone()).is_err());
/// assert!(FloatToken::from_text("10_#12.34", pos.clone()).is_err());
/// assert!(FloatToken::from_text("12.34e-1__0", pos.clone()).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct FloatToken {
    value: f64,
    text: String,
    pos: Position,
}
impl FloatToken {
    /// Makes a new `FloatToken` instance from the value.
    ///
    /// For finite non-negative values (including `-0.0`), the generated text
    /// is a valid Erlang float literal which can be parsed back by
    /// [`from_text`](Self::from_text).
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::FloatToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(FloatToken::from_value(1.23, pos.clone()).text(), "1.23");
    /// assert_eq!(FloatToken::from_value(1.0, pos.clone()).text(), "1.0");
    /// assert_eq!(FloatToken::from_value(-0.0, pos.clone()).text(), "0.0");
    /// ```
    pub fn from_value(value: f64, pos: Position) -> Self {
        // `-0.0` is not negative in comparison but its `Display` form `"-0"`
        // would be unparseable; normalize the text to `"0"` so the guarantee
        // above covers it. The stored value is left untouched.
        let mut text = format!("{}", if value == 0.0 { 0.0 } else { value });
        // `f64::Display` never uses exponent notation, so a text without
        // `'.'` is always plain integer notation.
        if !text.contains('.') {
            text.push_str(".0");
        }
        FloatToken { value, text, pos }
    }

    /// Tries to convert from any prefixes of the text to a `FloatToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let scanned = crate::lex::scan_float(text, pos.clone())?;
        let slice = &text[..scanned.len];
        let value = if let Some(hash) = slice.find('#') {
            Self::decode_radix(slice, hash)
        } else {
            util::strip_underscores(slice)
                .parse::<f64>()
                .expect("scanner validated decimal float")
        };
        Ok(FloatToken {
            value,
            text: slice.to_owned(),
            pos,
        })
    }

    fn decode_radix(slice: &str, hash: usize) -> f64 {
        let radix: u32 = util::strip_underscores(&slice[..hash])
            .parse()
            .expect("scanner validated radix");
        let rest = &slice[hash + 1..];
        let dot = rest.find('.').expect("scanner validated dot");
        let int_part = &rest[..dot];
        let after_dot = &rest[dot + 1..];
        let (frac_part, exp_opt) = if let Some(second_hash) = after_dot.find('#') {
            (
                &after_dot[..second_hash],
                Some(&after_dot[second_hash + 2..]),
            )
        } else {
            (after_dot, None)
        };

        let mut value = 0.0_f64;
        for c in int_part.chars() {
            if c == '_' {
                continue;
            }
            let d = c.to_digit(radix).expect("scanner validated integer digit");
            value = value * radix as f64 + d as f64;
        }
        let mut j = 1_i32;
        for c in frac_part.chars() {
            if c == '_' {
                continue;
            }
            let d = c
                .to_digit(radix)
                .expect("scanner validated fractional digit");
            value += d as f64 / (radix as f64).powi(j);
            j += 1;
        }
        if let Some(exp_str) = exp_opt {
            let exp: i32 = util::strip_underscores(exp_str)
                .parse()
                .expect("scanner validated exponent");
            value *= (radix as f64).powi(exp);
        }
        value
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
/// # extern crate erl_tokenize;
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::IntegerToken;
///
/// # fn main() {
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(IntegerToken::from_text("10", pos.clone()).unwrap().value(),
///            Some(10i64));
/// assert_eq!(IntegerToken::from_text("123_456", pos.clone()).unwrap().value(),
///            Some(123456i64));
/// assert_eq!(IntegerToken::from_text("16#ab0e", pos.clone()).unwrap().value(),
///            Some(0xab0e));
/// assert_eq!(IntegerToken::from_text("1_6#a_b_0e", pos.clone()).unwrap().value(),
///            Some(0xab0e));
///
/// // Out of range returns None
/// assert_eq!(IntegerToken::from_text("9223372036854775808", pos.clone()).unwrap().value(),
///            None);
///
/// // Err
/// assert!(IntegerToken::from_text("-10", pos.clone()).is_err());
/// assert!(IntegerToken::from_text("123_456_", pos.clone()).is_err());
/// assert!(IntegerToken::from_text("123__456", pos.clone()).is_err());
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct IntegerToken {
    value: Option<i64>, // None if the value is out of range
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
    /// assert_eq!(IntegerToken::from_value(123, pos.clone()).text(), "123");
    /// ```
    pub fn from_value(value: i64, pos: Position) -> Self {
        let text = format!("{value}");
        IntegerToken {
            value: Some(value),
            text,
            pos,
        }
    }

    /// Tries to convert from any prefixes of the text to an `IntegerToken`.
    ///
    /// Returns `Ok` even if the parsed value is out of range for `i64`;
    /// in such cases, `value()` will return `None`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let scanned = crate::lex::scan_integer(text, pos.clone())?;
        let slice = &text[..scanned.len];
        let (radix, digits_slice) = if let Some(hash) = slice.find('#') {
            let radix: u32 = util::strip_underscores(&slice[..hash])
                .parse()
                .expect("scanner validated radix");
            (radix, &slice[hash + 1..])
        } else {
            (10u32, slice)
        };
        let cleaned = util::strip_underscores(digits_slice);
        let value = if radix == 10 {
            cleaned.parse::<i64>().ok()
        } else {
            i64::from_str_radix(&cleaned, radix).ok()
        };
        Ok(IntegerToken {
            value,
            text: slice.to_owned(),
            pos,
        })
    }

    /// Returns the value of this token.
    ///
    /// Returns `Some(value)` if the integer fits in an `i64`, or `None` if it's out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate erl_tokenize;
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::IntegerToken;
    ///
    /// # fn main() {
    /// let pos = Position::new();
    ///
    /// assert_eq!(IntegerToken::from_text("10", pos.clone()).unwrap().value(),
    ///            Some(10i64));
    /// assert_eq!(IntegerToken::from_text("16#ab0e", pos.clone()).unwrap().value(),
    ///            Some(0xab0e));
    /// assert_eq!(IntegerToken::from_text("9223372036854775808", pos.clone()).unwrap().value(),
    ///            None);
    /// # }
    /// ```
    pub fn value(&self) -> Option<i64> {
        self.value
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
#[derive(Debug, Clone)]
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
        let scanned = crate::lex::scan_keyword(text, pos.clone())?;
        match scanned.kind {
            crate::lex::ScanKind::Keyword(value) => Ok(KeywordToken { value, pos }),
            _ => unreachable!("scan_keyword returns Keyword or errors"),
        }
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

/// Sigil string token.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Position;
/// use erl_tokenize::tokens::SigilStringToken;
///
/// # fn main() -> erl_tokenize::Result<()> {
/// let pos = Position::new();
///
/// // Ok
/// assert_eq!(SigilStringToken::from_text(r#"~"foo""#, pos.clone())?.value(), ("", "foo", ""));
/// assert_eq!(SigilStringToken::from_text(r#"~(foo)"#, pos.clone())?.value(), ("", "foo", ""));
/// assert_eq!(SigilStringToken::from_text(r#"~b"foo"  "#, pos.clone())?.value(), ("b", "foo", ""));
///
/// // Err
/// assert!(SigilStringToken::from_text(r#""foo""#, pos.clone()).is_err());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SigilStringToken {
    prefix: String,
    content: String,
    suffix: String,
    text: String,
    pos: Position,
}

impl SigilStringToken {
    /// Returns the value (i.e., prefix, content, suffix) of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::SigilStringToken;
    ///
    /// # fn main() -> erl_tokenize::Result<()> {
    /// let pos = Position::new();
    ///
    /// assert_eq!(SigilStringToken::from_text(r#"~"foo""#, pos.clone())?.value(), ("", "foo", ""));
    /// assert_eq!(SigilStringToken::from_text(r#"~(foo)"#, pos.clone())?.value(), ("", "foo", ""));
    /// assert_eq!(SigilStringToken::from_text(r#"~b"foo"  "#, pos.clone())?.value(), ("b", "foo", ""));
    /// # Ok(())
    /// # }
    /// ```
    pub fn value(&self) -> (&str, &str, &str) {
        (&self.prefix, &self.content, &self.suffix)
    }

    /// Returns the original textual representation of this token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::SigilStringToken;
    ///
    /// # fn main() -> erl_tokenize::Result<()> {
    /// let pos = Position::new();
    ///
    /// assert_eq!(SigilStringToken::from_text(r#"~"foo""#, pos.clone())?.text(), r#"~"foo""#);
    /// assert_eq!(SigilStringToken::from_text(r#"~(foo)"#, pos.clone())?.text(), r#"~(foo)"#);
    /// assert_eq!(SigilStringToken::from_text(r#"~b"foo"  "#, pos.clone())?.text(), r#"~b"foo""#);
    /// # Ok(())
    /// # }
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Tries to convert from any prefixes of the text to a [`SigilStringToken`].
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let scanned = crate::lex::scan_sigil_string(text, pos.clone())?;
        let slice = &text[..scanned.len];
        let mut offset = 1;
        for c in slice[offset..].chars() {
            if !util::is_atom_non_head_char(c) {
                break;
            }
            offset += c.len_utf8();
        }
        let prefix = slice[1..offset].to_owned();
        let open_delimiter = slice[offset..].chars().next().expect("scanner validated");
        let (content, content_end) = if open_delimiter == '"' {
            let t = StringToken::from_text(&slice[offset..], pos.clone().step_by_width(offset))?;
            let end = offset + t.text().len();
            (t.value().to_owned(), end)
        } else {
            let close_delimiter = match open_delimiter {
                '(' => ')',
                '[' => ']',
                '{' => '}',
                '<' => '>',
                other => other,
            };
            let (v, end) = util::parse_quotation(
                pos.clone().step_by_width(offset + 1),
                &slice[offset + 1..],
                close_delimiter,
            )?;
            (v.into_owned(), offset + 1 + end + 1)
        };
        let suffix = slice[content_end..].to_owned();
        Ok(Self {
            prefix,
            content,
            suffix,
            text: slice.to_owned(),
            pos,
        })
    }
}

impl PositionRange for SigilStringToken {
    fn start_position(&self) -> Position {
        self.pos.clone()
    }

    fn end_position(&self) -> Position {
        self.pos.clone().step_by_text(&self.text)
    }
}

impl fmt::Display for SigilStringToken {
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
#[derive(Debug, Clone)]
pub struct StringToken {
    value: Option<String>,
    text: String,
    pos: Position,
}
impl StringToken {
    /// Makes a new `StringToken` instance from the value.
    ///
    /// The generated text is a valid Erlang string literal which can be
    /// parsed back by [`from_text`](Self::from_text).
    ///
    /// Exception: U+FFFE and U+FFFF are not part of the Erlang character
    /// set (erl_scan rejects them), so the generated text for such values
    /// is not a valid Erlang literal, although [`from_text`](Self::from_text)
    /// can still parse it back.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Position;
    /// use erl_tokenize::tokens::StringToken;
    ///
    /// let pos = Position::new();
    /// assert_eq!(StringToken::from_value("foo", pos.clone()).text(), r#""foo""#);
    /// assert_eq!(StringToken::from_value("a\u{1}b", pos.clone()).text(), r#""a\x{1}b""#);
    /// ```
    pub fn from_value(value: &str, pos: Position) -> Self {
        let mut text = String::from("\"");
        for c in value.chars() {
            util::push_escaped_char(&mut text, c);
        }
        text.push('"');
        StringToken {
            value: Some(value.to_string()),
            text,
            pos,
        }
    }

    /// Tries to convert from any prefixes of the text to a `StringToken`.
    pub fn from_text(text: &str, pos: Position) -> Result<Self> {
        let scanned = crate::lex::scan_string(text, pos.clone())?;
        let slice = &text[..scanned.len];
        let value = if slice.starts_with(r#"""""#) {
            let (v, _) = Self::parse_triple_quoted(slice, pos.clone())?;
            Some(v.into_owned())
        } else {
            let (v, _) = util::parse_quotation(pos.clone(), &slice[1..], '"')?;
            match v {
                Cow::Borrowed(_) => None,
                Cow::Owned(s) => Some(s),
            }
        };
        Ok(StringToken {
            value,
            text: slice.to_owned(),
            pos,
        })
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
        for c in chars {
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

        let mut indent = 0;
        let mut maybe_end_line = true;
        let mut remaining_quote_count = quote_count;
        let mut end_line_start = start_line_end;
        let mut end_line_end = start_line_end;
        for c in text[start_line_end..].chars() {
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
                Cow::Owned(
                    text[start_line_end..(end_line_start - 1).max(start_line_end)].to_owned(),
                ),
                end_line_end,
            ));
        }

        let mut value = String::new();
        for line in text[start_line_end..end_line_start - 1].lines() {
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
#[derive(Debug, Clone)]
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
        let scanned = crate::lex::scan_symbol(text, pos.clone())?;
        match scanned.kind {
            crate::lex::ScanKind::Symbol(value) => Ok(SymbolToken { value, pos }),
            _ => unreachable!("scan_symbol returns Symbol or errors"),
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
#[derive(Debug, Clone)]
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
        let scanned = crate::lex::scan_variable(text, pos.clone())?;
        Ok(VariableToken {
            text: text[..scanned.len].to_owned(),
            pos,
        })
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
#[derive(Debug, Clone)]
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
        let scanned = crate::lex::scan_whitespace(text, pos.clone())?;
        match scanned.kind {
            crate::lex::ScanKind::Whitespace(value) => Ok(WhitespaceToken { value, pos }),
            _ => unreachable!("scan_whitespace returns Whitespace or errors"),
        }
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
