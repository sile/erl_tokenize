use crate::Position;

/// Possible errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum Error {
    /// No closing quotation.
    #[error("no closing quotation ({position})")]
    NoClosingQuotation { position: Position },

    /// Invalid escaped character.
    #[error("cannot parse a escaped character ({position})")]
    InvalidEscapedChar { position: Position },

    /// A token was expected, but not found.
    #[error("a token was expected, but not found ({position})")]
    MissingToken { position: Position },

    /// Unknown keyword.
    #[error("unknown keyword {keyword:?} ({position})")]
    UnknownKeyword { position: Position, keyword: String },

    /// Invalid atom token.
    #[error("Canot parse an atom token ({position})")]
    InvalidAtomToken { position: Position },

    /// Invalid character token.
    #[error("cannot parse a character token ({position})")]
    InvalidCharToken { position: Position },

    /// Invalid comment token.
    #[error("cannot parse a comment token ({position})")]
    InvalidCommentToken { position: Position },

    /// Invalid float token.
    #[error("cannot parse a float token ({position})")]
    InvalidFloatToken { position: Position },

    /// Invalid integer token.
    #[error("cannot parse a integer token ({position})")]
    InvalidIntegerToken { position: Position },

    /// Invalid string token.
    #[error("cannot parse a string token ({position})")]
    InvalidStringToken { position: Position },

    /// Invalid symbol token.
    #[error("cannot parse a symbol token ({position})")]
    InvalidSymbolToken { position: Position },

    /// Invalid variable token.
    #[error("cannot parse a variable token ({position})")]
    InvalidVariableToken { position: Position },

    /// Invalid whitespace token.
    #[error("cannot parse a whitespace token ({position})")]
    InvalidWhitespaceToken { position: Position },
}

impl Error {
    /// Return a `Position` at where this error occurred.
    pub fn position(&self) -> &Position {
        match self {
            Self::NoClosingQuotation { position } => position,
            Self::InvalidEscapedChar { position } => position,
            Self::MissingToken { position } => position,
            Self::UnknownKeyword { position, .. } => position,
            Self::InvalidAtomToken { position } => position,
            Self::InvalidCharToken { position } => position,
            Self::InvalidCommentToken { position } => position,
            Self::InvalidFloatToken { position } => position,
            Self::InvalidIntegerToken { position } => position,
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
