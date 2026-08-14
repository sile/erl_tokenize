use crate::Position;

/// Lexical error produced by the scanner.
///
/// Carries the diagnostic position of the offending input and a resume
/// position that the caller can pass back to
/// [`scan_token`](crate::scan_token) to continue past it. All fields
/// are public — the caller reads `kind` to classify the failure,
/// `position` to report where it occurred, and `resume_position` to
/// continue scanning.
///
/// The `resume_position` always lies on a UTF-8 boundary of the same
/// source, strictly after the position at which the failing token
/// scan started, and no further than the end of the source. On a
/// non-empty input this advances by exactly one Unicode scalar value
/// from the scan-start position, so a caller that loops on errors
/// with `err.resume_position` never spins in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error {
    /// Classification of the failure.
    pub kind: ErrorKind,
    /// Diagnostic position of the offending input.
    pub position: Position,
    /// Position at which the caller should resume scanning.
    pub resume_position: Position,
}

impl Error {
    pub(crate) const fn new(kind: ErrorKind, position: Position) -> Self {
        Self {
            kind,
            position,
            resume_position: position,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.kind.message(), self.position)
    }
}

impl std::error::Error for Error {}

/// Classification of a lexical [`Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// The scanner reached the end of the source without finding the
    /// closing quotation character of a string, atom, or sigil string.
    NoClosingQuotation,

    /// The scanner encountered a backslash-introduced escape sequence
    /// that is not a valid Erlang escape.
    InvalidEscapedChar,

    /// Two string literals appear next to each other without an
    /// intervening whitespace token.
    AdjacentStringLiterals,

    /// A token was expected but not found (empty input at the current
    /// position).
    MissingToken,

    /// The scanner failed to parse an atom token.
    InvalidAtomToken,

    /// The scanner failed to parse a character token.
    InvalidCharToken,

    /// The scanner failed to parse a comment token.
    InvalidCommentToken,

    /// The scanner failed to parse a float token.
    InvalidFloatToken,

    /// The scanner failed to parse an integer token.
    InvalidIntegerToken,

    /// The scanner failed to parse a string token.
    InvalidStringToken,

    /// The scanner failed to parse a sigil string token.
    InvalidSigilStringToken,

    /// The scanner failed to parse a symbol token.
    InvalidSymbolToken,

    /// The scanner failed to parse a variable token.
    InvalidVariableToken,

    /// The scanner failed to parse a whitespace token.
    InvalidWhitespaceToken,
}

impl ErrorKind {
    /// Returns the human-readable message associated with this kind.
    pub const fn message(self) -> &'static str {
        match self {
            Self::NoClosingQuotation => "no closing quotation",
            Self::InvalidEscapedChar => "cannot parse an escaped character",
            Self::AdjacentStringLiterals => {
                "adjacent string literals without intervening white space"
            }
            Self::MissingToken => "a token was expected, but not found",
            Self::InvalidAtomToken => "cannot parse an atom token",
            Self::InvalidCharToken => "cannot parse a character token",
            Self::InvalidCommentToken => "cannot parse a comment token",
            Self::InvalidFloatToken => "cannot parse a float token",
            Self::InvalidIntegerToken => "cannot parse an integer token",
            Self::InvalidStringToken => "cannot parse a string token",
            Self::InvalidSigilStringToken => "cannot parse a sigil string token",
            Self::InvalidSymbolToken => "cannot parse a symbol token",
            Self::InvalidVariableToken => "cannot parse a variable token",
            Self::InvalidWhitespaceToken => "cannot parse a whitespace token",
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}
