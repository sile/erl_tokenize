use std::borrow::Cow;
use std::fmt;

use crate::lex::{self, ScanKind};
use crate::values::{Keyword, Symbol};
use crate::{Position, Result};

/// Kind of a scanned token.
///
/// Keywords and symbols are fully determined by their textual form and
/// therefore carry the resolved enum value directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// Erlang atom (bare or single-quoted).
    Atom,
    /// Erlang character literal (`$x` or `$\...`).
    Char,
    /// Line comment starting with `%`.
    Comment,
    /// Floating-point number literal.
    Float,
    /// Integer literal (decimal or radix-prefixed).
    Integer,
    /// Reserved word (`case`, `end`, …).
    Keyword(Keyword),
    /// Sigil string literal (`~"..."`, `~b(...)`, …).
    SigilString,
    /// Double-quoted string literal (including triple-quoted form).
    String,
    /// Punctuation or operator symbol.
    Symbol(Symbol),
    /// Variable identifier (starting with an uppercase letter or `_`).
    Variable,
    /// A single whitespace character (space, tab, CR, LF, or NBSP).
    Whitespace,
}

impl TokenKind {
    /// Returns `true` for kinds that are ignored by grammatical analysis
    /// (currently comments and whitespace).
    pub const fn is_hidden(self) -> bool {
        matches!(self, TokenKind::Comment | TokenKind::Whitespace)
    }

    /// Returns `true` for kinds that carry lexical meaning (the negation
    /// of [`is_hidden`](Self::is_hidden)).
    pub const fn is_lexical(self) -> bool {
        !self.is_hidden()
    }
}

impl From<ScanKind> for TokenKind {
    fn from(kind: ScanKind) -> Self {
        match kind {
            ScanKind::Atom => TokenKind::Atom,
            ScanKind::Char => TokenKind::Char,
            ScanKind::Comment => TokenKind::Comment,
            ScanKind::Float => TokenKind::Float,
            ScanKind::Integer => TokenKind::Integer,
            ScanKind::Keyword(k) => TokenKind::Keyword(k),
            ScanKind::SigilString => TokenKind::SigilString,
            ScanKind::String => TokenKind::String,
            ScanKind::Symbol(s) => TokenKind::Symbol(s),
            ScanKind::Variable => TokenKind::Variable,
            ScanKind::Whitespace => TokenKind::Whitespace,
        }
    }
}

/// A scanned token: kind and half-open position range in the source.
///
/// `Token` does not borrow the source and does not own any decoded value.
/// Call [`text`](Self::text) with the same source that was passed to
/// [`scan_token`] to retrieve the original substring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    kind: TokenKind,
    start: Position,
    end: Position,
}

impl Token {
    /// Returns the kind of this token.
    pub const fn kind(self) -> TokenKind {
        self.kind
    }

    /// Returns the (inclusive) start position of this token.
    pub const fn start(self) -> Position {
        self.start
    }

    /// Returns the (exclusive) end position of this token.
    pub const fn end(self) -> Position {
        self.end
    }

    /// Returns `true` when the token is a hidden token (comment or
    /// whitespace).
    pub const fn is_hidden(self) -> bool {
        self.kind.is_hidden()
    }

    /// Returns `true` when the token carries lexical meaning.
    pub const fn is_lexical(self) -> bool {
        self.kind.is_lexical()
    }

    /// Returns the original substring of `source` that this token
    /// represents.
    ///
    /// # Panics
    ///
    /// Panics if `source` is shorter than `self.end().offset()` or if
    /// `self.start().offset()..self.end().offset()` does not correspond to
    /// valid UTF-8 character boundaries.
    ///
    /// Callers are expected to pass the same source that was given to
    /// [`scan_token`]; passing a different source of the same length is
    /// undetected and yields nonsensical text.
    pub fn text(self, source: &str) -> &str {
        source
            .get(self.start.offset()..self.end.offset())
            .expect("token range must lie on UTF-8 boundaries of the original source")
    }

    /// Decode this token's value from `source`.
    ///
    /// The returned [`TokenValue`] borrows from `source` whenever
    /// possible: bare atoms, single-line quoted atoms and strings whose
    /// content has no escape sequences, sigil-string prefixes and
    /// suffixes, comments, variables, whitespace, and triple-quoted
    /// strings whose closing line has no indentation. Content that
    /// requires escape decoding or indentation stripping is returned as
    /// `Cow::Owned`. `Token::value` never caches; each call re-decodes.
    ///
    /// # Panics
    ///
    /// Panics for the same reasons as [`text`](Self::text): `source` must
    /// be the same string that was passed to [`scan_token`].
    pub fn value<'a>(self, source: &'a str) -> TokenValue<'a> {
        let text = self.text(source);
        match self.kind {
            TokenKind::Atom => TokenValue::Atom(lex::decode_atom(text)),
            TokenKind::Char => TokenValue::Char(lex::decode_char(text)),
            TokenKind::Comment => TokenValue::Comment(lex::decode_comment(text)),
            TokenKind::Float => TokenValue::Float(lex::decode_float(text)),
            TokenKind::Integer => TokenValue::Integer(lex::decode_integer(text)),
            TokenKind::Keyword(k) => TokenValue::Keyword(k),
            TokenKind::SigilString => {
                let (prefix, content, suffix) = lex::decode_sigil(text);
                TokenValue::SigilString {
                    prefix,
                    content,
                    suffix,
                }
            }
            TokenKind::String => TokenValue::String(lex::decode_string(text)),
            TokenKind::Symbol(s) => TokenValue::Symbol(s),
            TokenKind::Variable => TokenValue::Variable(text),
            TokenKind::Whitespace => TokenValue::Whitespace(text),
        }
    }
}

/// Decoded value of a [`Token`], borrowing from the source where
/// possible.
///
/// Produced by [`Token::value`]. Variants that only need to reference
/// the source (comment body, variable name, whitespace text, sigil-string
/// prefix and suffix) hold a `&str`; variants that may require escape or
/// indentation decoding hold a `Cow<'a, str>`.
///
/// `TokenValue` does not implement `Eq` because [`f64`] does not; use
/// value-specific comparisons for `Float`.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue<'a> {
    /// Atom value (borrowed for bare atoms and escape-free quoted atoms).
    Atom(Cow<'a, str>),
    /// Decoded character value.
    Char(char),
    /// Comment body without the leading `%`.
    Comment(&'a str),
    /// Decoded floating-point value.
    Float(f64),
    /// Decoded integer value, or `None` when it exceeds `i64::MAX`.
    Integer(Option<i64>),
    /// Reserved word.
    Keyword(Keyword),
    /// Sigil string parts.
    SigilString {
        /// Prefix identifier between `~` and the opening delimiter.
        prefix: &'a str,
        /// Content between the opening and closing delimiters. Borrowed
        /// when no escape decoding or indentation stripping is required.
        content: Cow<'a, str>,
        /// Suffix identifier after the closing delimiter.
        suffix: &'a str,
    },
    /// Decoded string value (borrowed when no escape decoding or
    /// indentation stripping is required).
    String(Cow<'a, str>),
    /// Punctuation or operator symbol.
    Symbol(Symbol),
    /// Variable identifier text.
    Variable(&'a str),
    /// Whitespace token text.
    Whitespace(&'a str),
}

/// Scan a single token from `source` starting at `position`.
///
/// - `source` must always be the whole source string used with
///   [`Position::new`] onward.
/// - The first call takes `Position::new()`. Successive calls pass
///   `token.end()` after a successful scan, or `error.resume_position()`
///   after an error.
/// - Returns `Ok(None)` when `position.offset() == source.len()`.
/// - Returns `Ok(Some(token))` when a token is recognised.
/// - Returns `Err(error)` when the source at `position` is not a valid
///   token; the error carries a diagnostic position and a resume position.
///
/// # Panics
///
/// Panics if `position.offset()` is outside `0..=source.len()` or if it
/// does not lie on a UTF-8 character boundary of `source`. Line and column
/// consistency across different source strings is not verified.
pub fn scan_token(source: &str, position: Position) -> Result<Option<Token>> {
    let offset = position.offset();
    assert!(
        offset <= source.len(),
        "position.offset() ({offset}) exceeds source length ({})",
        source.len()
    );
    if offset == source.len() {
        return Ok(None);
    }
    let tail = source
        .get(offset..)
        .expect("position.offset() must lie on a UTF-8 boundary of source");
    let scanned = lex::scan_one(tail, position)?;
    let end = position.step_by_text(&tail[..scanned.len]);
    Ok(Some(Token {
        kind: TokenKind::from(scanned.kind),
        start: position,
        end,
    }))
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{}..{}", self.kind, self.start, self.end)
    }
}
