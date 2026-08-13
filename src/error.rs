use crate::Position;

/// Lexical error produced by the scanner.
///
/// Each variant carries the diagnostic position of the offending input
/// and a resume position that the caller can pass back to
/// [`scan_token`](crate::scan_token) to continue past it. Use
/// [`position`](Self::position) to report where the error occurred and
/// [`resume_position`](Self::resume_position) to continue scanning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The scanner reached the end of the source without finding the
    /// closing quotation character of a string, atom, or sigil string.
    NoClosingQuotation {
        /// Position at which the opening quote was seen.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner encountered a backslash-introduced escape sequence
    /// that is not a valid Erlang escape.
    InvalidEscapedChar {
        /// Position of the offending escape sequence.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// Two string literals appear next to each other without an
    /// intervening whitespace token.
    AdjacentStringLiterals {
        /// Position of the second literal.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// A token was expected but not found (empty input at the current
    /// position).
    MissingToken {
        /// Position at which the token was expected.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse an atom token.
    InvalidAtomToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse a character token.
    InvalidCharToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse a comment token.
    InvalidCommentToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse a float token.
    InvalidFloatToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse an integer token.
    InvalidIntegerToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse a string token.
    InvalidStringToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse a sigil string token.
    InvalidSigilStringToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse a symbol token.
    InvalidSymbolToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse a variable token.
    InvalidVariableToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },

    /// The scanner failed to parse a whitespace token.
    InvalidWhitespaceToken {
        /// Position of the offending input.
        position: Position,
        /// Position at which the caller should resume scanning.
        resume: Position,
    },
}

impl Error {
    /// Returns the diagnostic position of this error.
    pub const fn position(self) -> Position {
        match self {
            Self::NoClosingQuotation { position, .. }
            | Self::InvalidEscapedChar { position, .. }
            | Self::AdjacentStringLiterals { position, .. }
            | Self::MissingToken { position, .. }
            | Self::InvalidAtomToken { position, .. }
            | Self::InvalidCharToken { position, .. }
            | Self::InvalidCommentToken { position, .. }
            | Self::InvalidFloatToken { position, .. }
            | Self::InvalidIntegerToken { position, .. }
            | Self::InvalidSigilStringToken { position, .. }
            | Self::InvalidStringToken { position, .. }
            | Self::InvalidSymbolToken { position, .. }
            | Self::InvalidVariableToken { position, .. }
            | Self::InvalidWhitespaceToken { position, .. } => position,
        }
    }

    /// Returns the position at which the caller should resume scanning.
    ///
    /// The resume position always lies on a UTF-8 boundary of the same
    /// source, strictly after the position at which the failing token
    /// scan started, and no further than the end of the source. On a
    /// non-empty input this advances by exactly one Unicode scalar value
    /// from the scan-start position, so a caller that loops on errors
    /// with `err.resume_position()` never spins in place.
    pub const fn resume_position(self) -> Position {
        match self {
            Self::NoClosingQuotation { resume, .. }
            | Self::InvalidEscapedChar { resume, .. }
            | Self::AdjacentStringLiterals { resume, .. }
            | Self::MissingToken { resume, .. }
            | Self::InvalidAtomToken { resume, .. }
            | Self::InvalidCharToken { resume, .. }
            | Self::InvalidCommentToken { resume, .. }
            | Self::InvalidFloatToken { resume, .. }
            | Self::InvalidIntegerToken { resume, .. }
            | Self::InvalidSigilStringToken { resume, .. }
            | Self::InvalidStringToken { resume, .. }
            | Self::InvalidSymbolToken { resume, .. }
            | Self::InvalidVariableToken { resume, .. }
            | Self::InvalidWhitespaceToken { resume, .. } => resume,
        }
    }

    /// Returns a copy of this error with the resume position replaced.
    ///
    /// Used by the scanner to attach the scan-start-derived resume
    /// position to errors reported from deeper helpers (which know only
    /// the diagnostic position of the offending byte).
    pub(crate) fn with_resume(mut self, new_resume: Position) -> Self {
        let slot = match &mut self {
            Self::NoClosingQuotation { resume, .. }
            | Self::InvalidEscapedChar { resume, .. }
            | Self::AdjacentStringLiterals { resume, .. }
            | Self::MissingToken { resume, .. }
            | Self::InvalidAtomToken { resume, .. }
            | Self::InvalidCharToken { resume, .. }
            | Self::InvalidCommentToken { resume, .. }
            | Self::InvalidFloatToken { resume, .. }
            | Self::InvalidIntegerToken { resume, .. }
            | Self::InvalidSigilStringToken { resume, .. }
            | Self::InvalidStringToken { resume, .. }
            | Self::InvalidSymbolToken { resume, .. }
            | Self::InvalidVariableToken { resume, .. }
            | Self::InvalidWhitespaceToken { resume, .. } => resume,
        };
        *slot = new_resume;
        self
    }

    pub(crate) fn no_closing_quotation(position: Position) -> Self {
        Self::NoClosingQuotation {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_escaped_char(position: Position) -> Self {
        Self::InvalidEscapedChar {
            position,
            resume: position,
        }
    }

    pub(crate) fn adjacent_string_literals(position: Position) -> Self {
        Self::AdjacentStringLiterals {
            position,
            resume: position,
        }
    }

    pub(crate) fn missing_token(position: Position) -> Self {
        Self::MissingToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_atom_token(position: Position) -> Self {
        Self::InvalidAtomToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_char_token(position: Position) -> Self {
        Self::InvalidCharToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_comment_token(position: Position) -> Self {
        Self::InvalidCommentToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_float_token(position: Position) -> Self {
        Self::InvalidFloatToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_integer_token(position: Position) -> Self {
        Self::InvalidIntegerToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_sigil_string_token(position: Position) -> Self {
        Self::InvalidSigilStringToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_string_token(position: Position) -> Self {
        Self::InvalidStringToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_symbol_token(position: Position) -> Self {
        Self::InvalidSymbolToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_variable_token(position: Position) -> Self {
        Self::InvalidVariableToken {
            position,
            resume: position,
        }
    }

    pub(crate) fn invalid_whitespace_token(position: Position) -> Self {
        Self::InvalidWhitespaceToken {
            position,
            resume: position,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let position = self.position();
        match self {
            Error::NoClosingQuotation { .. } => write!(f, "no closing quotation ({position})"),
            Error::InvalidEscapedChar { .. } => {
                write!(f, "cannot parse an escaped character ({position})")
            }
            Error::AdjacentStringLiterals { .. } => {
                write!(
                    f,
                    "adjacent string literals without intervening white space ({position})"
                )
            }
            Error::MissingToken { .. } => {
                write!(f, "a token was expected, but not found ({position})")
            }
            Error::InvalidAtomToken { .. } => write!(f, "cannot parse an atom token ({position})"),
            Error::InvalidCharToken { .. } => {
                write!(f, "cannot parse a character token ({position})")
            }
            Error::InvalidCommentToken { .. } => {
                write!(f, "cannot parse a comment token ({position})")
            }
            Error::InvalidFloatToken { .. } => {
                write!(f, "cannot parse a float token ({position})")
            }
            Error::InvalidIntegerToken { .. } => {
                write!(f, "cannot parse an integer token ({position})")
            }
            Error::InvalidStringToken { .. } => {
                write!(f, "cannot parse a string token ({position})")
            }
            Error::InvalidSigilStringToken { .. } => {
                write!(f, "cannot parse a sigil string token ({position})")
            }
            Error::InvalidSymbolToken { .. } => {
                write!(f, "cannot parse a symbol token ({position})")
            }
            Error::InvalidVariableToken { .. } => {
                write!(f, "cannot parse a variable token ({position})")
            }
            Error::InvalidWhitespaceToken { .. } => {
                write!(f, "cannot parse a whitespace token ({position})")
            }
        }
    }
}

impl std::error::Error for Error {}
