use crate::Position;

/// Possible errors.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum Error {
    /// No closing quotation.
    NoClosingQuotation { position: Position },

    /// Invalid escaped character.
    InvalidEscapedChar { position: Position },

    /// Adjacent string literals without intervening white space.
    AdjacentStringLiterals { position: Position },

    /// A token was expected, but not found.
    MissingToken { position: Position },

    /// Unknown keyword.
    UnknownKeyword { position: Position, keyword: String },

    /// Invalid atom token.
    InvalidAtomToken { position: Position },

    /// Invalid character token.
    InvalidCharToken { position: Position },

    /// Invalid comment token.
    InvalidCommentToken { position: Position },

    /// Invalid float token.
    InvalidFloatToken { position: Position },

    /// Invalid integer token.
    InvalidIntegerToken { position: Position },

    /// Invalid string token.
    InvalidStringToken { position: Position },

    /// Invalid sigil string token.
    InvalidSigilStringToken { position: Position },

    /// Invalid symbol token.
    InvalidSymbolToken { position: Position },

    /// Invalid variable token.
    InvalidVariableToken { position: Position },

    /// Invalid whitespace token.
    InvalidWhitespaceToken { position: Position },
}

impl Error {
    /// Return a `Position` at where this error occurred.
    pub fn position(&self) -> &Position {
        match self {
            Self::NoClosingQuotation { position } => position,
            Self::InvalidEscapedChar { position } => position,
            Self::AdjacentStringLiterals { position } => position,
            Self::MissingToken { position } => position,
            Self::UnknownKeyword { position, .. } => position,
            Self::InvalidAtomToken { position } => position,
            Self::InvalidCharToken { position } => position,
            Self::InvalidCommentToken { position } => position,
            Self::InvalidFloatToken { position } => position,
            Self::InvalidIntegerToken { position } => position,
            Self::InvalidSigilStringToken { position } => position,
            Self::InvalidStringToken { position } => position,
            Self::InvalidSymbolToken { position } => position,
            Self::InvalidVariableToken { position } => position,
            Self::InvalidWhitespaceToken { position } => position,
        }
    }

    pub(crate) fn no_closing_quotation(position: Position) -> Self {
        Self::NoClosingQuotation { position }
    }

    pub(crate) fn invalid_escaped_char(position: Position) -> Self {
        Self::InvalidEscapedChar { position }
    }

    pub(crate) fn adjacent_string_literals(position: Position) -> Self {
        Self::AdjacentStringLiterals { position }
    }

    pub(crate) fn missing_token(position: Position) -> Self {
        Self::MissingToken { position }
    }

    pub(crate) fn unknown_keyword(position: Position, keyword: String) -> Self {
        Self::UnknownKeyword { position, keyword }
    }

    pub(crate) fn invalid_atom_token(position: Position) -> Self {
        Self::InvalidAtomToken { position }
    }

    pub(crate) fn invalid_char_token(position: Position) -> Self {
        Self::InvalidCharToken { position }
    }

    pub(crate) fn invalid_comment_token(position: Position) -> Self {
        Self::InvalidCommentToken { position }
    }

    pub(crate) fn invalid_float_token(position: Position) -> Self {
        Self::InvalidFloatToken { position }
    }

    pub(crate) fn invalid_integer_token(position: Position) -> Self {
        Self::InvalidIntegerToken { position }
    }

    pub(crate) fn invalid_sigil_string_token(position: Position) -> Self {
        Self::InvalidSigilStringToken { position }
    }

    pub(crate) fn invalid_string_token(position: Position) -> Self {
        Self::InvalidStringToken { position }
    }

    pub(crate) fn invalid_symbol_token(position: Position) -> Self {
        Self::InvalidSymbolToken { position }
    }

    pub(crate) fn invalid_variable_token(position: Position) -> Self {
        Self::InvalidVariableToken { position }
    }

    pub(crate) fn invalid_whitespace_token(position: Position) -> Self {
        Self::InvalidWhitespaceToken { position }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoClosingQuotation { position } => {
                write!(f, "no closing quotation ({position})")
            }
            Error::InvalidEscapedChar { position } => {
                write!(f, "cannot parse a escaped character ({position})")
            }
            Error::AdjacentStringLiterals { position } => {
                write!(
                    f,
                    "adjacent string literals without intervening white space ({position})"
                )
            }
            Error::MissingToken { position } => {
                write!(f, "a token was expected, but not found ({position})")
            }
            Error::UnknownKeyword { position, keyword } => {
                write!(f, "unknown keyword {keyword:?} ({position})")
            }
            Error::InvalidAtomToken { position } => {
                write!(f, "cannot parse an atom token ({position})")
            }
            Error::InvalidCharToken { position } => {
                write!(f, "cannot parse a character token ({position})")
            }
            Error::InvalidCommentToken { position } => {
                write!(f, "cannot parse a comment token ({position})")
            }
            Error::InvalidFloatToken { position } => {
                write!(f, "cannot parse a float token ({position})")
            }
            Error::InvalidIntegerToken { position } => {
                write!(f, "cannot parse a integer token ({position})")
            }
            Error::InvalidStringToken { position } => {
                write!(f, "cannot parse a string token ({position})")
            }
            Error::InvalidSigilStringToken { position } => {
                write!(f, "cannot parse a sigil string token ({position})")
            }
            Error::InvalidSymbolToken { position } => {
                write!(f, "cannot parse a symbol token ({position})")
            }
            Error::InvalidVariableToken { position } => {
                write!(f, "cannot parse a variable token ({position})")
            }
            Error::InvalidWhitespaceToken { position } => {
                write!(f, "cannot parse a whitespace token ({position})")
            }
        }
    }
}

impl std::error::Error for Error {}
