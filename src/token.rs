use std::borrow::Cow;
use std::fmt;

use crate::keyword::Keyword;
use crate::lex;
use crate::symbol::Symbol;
use crate::{Position, Result};

/// Kind of a scanned token.
///
/// [`Keyword`] and [`Symbol`] are fully determined by their textual form,
/// so this enum resolves them eagerly and carries the resolved value as
/// the payload of [`TokenKind::Keyword`] and [`TokenKind::Symbol`]. Other
/// kinds have textual forms whose semantic value is not uniquely
/// determined until the caller decodes them with [`Token::value`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// Erlang atom (bare `foo` or single-quoted `'Foo'`).
    Atom,
    /// Erlang character literal (`$x`, `$\n`, and so on).
    Char,
    /// Line comment starting with `%`. The trailing LF is not included.
    Comment,
    /// Floating-point number literal.
    Float,
    /// Integer literal in decimal or `base#digits` radix form.
    Integer,
    /// Reserved word (`case`, `end`, ...).
    Keyword(Keyword),
    /// Sigil string literal (`~"..."`, `~b(...)`, ...).
    SigilString,
    /// Double-quoted string literal, including the triple-quoted form.
    String,
    /// Punctuation or operator symbol.
    Symbol(Symbol),
    /// Variable identifier (starts with an uppercase letter or `_`).
    Variable,
    /// Whitespace token. See [`Token`] for the aggregation rules.
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

/// A scanned token: kind and a half-open position range in the source.
///
/// `Token` does not borrow the source and does not own any decoded
/// value. Call [`text`](Self::text) with the same source that was passed
/// to [`scan_token`] to retrieve the original substring, and
/// [`value`](Self::value) to decode the token's semantic value on
/// demand.
///
/// # Whitespace aggregation
///
/// A [`TokenKind::Whitespace`] token follows the same aggregation rules
/// as `erl_scan`'s `return_white_spaces` option.
///
/// - Each whitespace token holds at most one LF (`\n`), and that LF is
///   always at the start of the token.
/// - Reaching a LF starts a new whitespace token.
/// - Non-LF whitespace (space, tab, CR, NBSP, ...) is aggregated into
///   a single token as long as it runs consecutively.
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
    ///
    /// Pass this position to the next [`scan_token`] call to continue
    /// scanning.
    pub const fn end(self) -> Position {
        self.end
    }

    /// Returns the substring of `source` that this token represents.
    ///
    /// `source` must be the same source string that was passed to the
    /// [`scan_token`] call that produced this token. Passing a different
    /// source of the same length is undetected and yields nonsensical
    /// text.
    ///
    /// # Panics
    ///
    /// Panics if `source.len() < self.end().offset()`, or if
    /// `self.start().offset()..self.end().offset()` does not lie on
    /// UTF-8 character boundaries of `source`. Both indicate a caller
    /// contract violation.
    pub fn text(self, source: &str) -> &str {
        source
            .get(self.start.offset()..self.end.offset())
            .expect("token range must lie on UTF-8 boundaries of the original source")
    }

    /// Decodes this token's value from `source`.
    ///
    /// The returned [`TokenValue`] borrows from `source` wherever
    /// possible: bare atoms, single-quoted atoms with no escape
    /// sequences, plain strings with no escape sequences, the prefix and
    /// suffix of a sigil string, comment bodies, variable names,
    /// whitespace text, and triple-quoted strings whose closing line has
    /// no indentation are all returned as `Cow::Borrowed`. Content that
    /// requires escape decoding or indentation stripping is returned as
    /// `Cow::Owned`.
    ///
    /// This method does not cache the decoded value; each call re-decodes.
    ///
    /// # Panics
    ///
    /// Panics for the same reasons as [`text`](Self::text): `source`
    /// must be the same string that was passed to [`scan_token`].
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
/// prefix and suffix) hold a `&str`. Variants whose value may require
/// escape decoding or indentation stripping hold a `Cow<'a, str>` and
/// return owned data only when transformation is required.
///
/// `TokenValue` does not implement `Eq` because [`f64`] does not; compare
/// float values with application-specific rules.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue<'a> {
    /// Atom value. Bare atoms and single-quoted atoms with no escape
    /// sequences are returned as `Cow::Borrowed`; atoms with escape
    /// sequences are returned as `Cow::Owned`.
    Atom(Cow<'a, str>),
    /// Decoded character value.
    Char(char),
    /// Comment body without the leading `%`.
    Comment(&'a str),
    /// Decoded floating-point value.
    ///
    /// Always finite: the scanner rejects a literal whose decoded value
    /// would be `±f64::INFINITY` (matching `erl_scan`'s `illegal float`
    /// error), and Erlang syntax admits no NaN literal. Underflow to
    /// `0.0` is accepted.
    Float(f64),
    /// Decoded integer value, or `None` when the value exceeds `u64::MAX`.
    ///
    /// Erlang integer literals are always non-negative (`-10` is the
    /// unary `-` operator applied to `10`), so `u64` matches the
    /// language's token domain.
    Integer(Option<u64>),
    /// Reserved word. Same value as the payload of
    /// [`TokenKind::Keyword`].
    Keyword(Keyword),
    /// Sigil string parts.
    SigilString {
        /// Prefix identifier between `~` and the opening delimiter.
        prefix: &'a str,
        /// Content between the opening and closing delimiters. Borrowed
        /// when no escape decoding or indentation stripping is required,
        /// otherwise owned.
        content: Cow<'a, str>,
        /// Suffix identifier after the closing delimiter.
        suffix: &'a str,
    },
    /// Decoded string value. Borrowed when no escape decoding or
    /// indentation stripping is required, otherwise owned.
    String(Cow<'a, str>),
    /// Punctuation or operator symbol. Same value as the payload of
    /// [`TokenKind::Symbol`].
    Symbol(Symbol),
    /// Variable identifier text.
    Variable(&'a str),
    /// Whitespace token text.
    Whitespace(&'a str),
}

/// Scans a single token from `source` starting at `position`.
///
/// - `source` must always be the whole source string used for the
///   session starting at [`Position::new`].
/// - Pass [`Position::new`] on the first call. On subsequent calls pass
///   [`Token::end`] after a successful scan or
///   [`Error::resume_position`](crate::Error::resume_position) after an
///   error.
/// - Returns `Ok(None)` when `position.offset() == source.len()`.
/// - Returns `Ok(Some(token))` when a token is recognised.
/// - Returns `Err(error)` when the input at `position` is not a valid
///   token. The error carries a diagnostic position and a resume
///   position.
///
/// # Panics
///
/// Panics if `position.offset()` is outside `0..=source.len()` or if it
/// does not lie on a UTF-8 character boundary of `source`. Line and
/// column consistency across different source strings is not verified.
/// These are caller contract violations.
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
        kind: scanned.kind,
        start: position,
        end,
    }))
}

/// Scans all tokens from `source` until EOF.
///
/// Convenience wrapper around repeated [`scan_token`] calls starting at
/// [`Position::new`]. Returns the full token stream in source order,
/// including comments and whitespace.
///
/// Stops at the first lexical error and returns it. Callers that resume
/// past errors should drive [`scan_token`] manually; see the crate-level
/// documentation for an example.
pub fn scan_tokens(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut position = Position::new();
    while let Some(token) = scan_token(source, position)? {
        position = token.end();
        tokens.push(token);
    }
    Ok(tokens)
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{}..{}", self.kind, self.start, self.end)
    }
}
