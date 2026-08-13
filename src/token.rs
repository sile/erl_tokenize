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
            ScanKind::Whitespace(_) => TokenKind::Whitespace,
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
