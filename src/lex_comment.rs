use crate::TokenKind;
use crate::lex::Scanned;
use crate::{Error, ErrorKind, Position, Result};

/// Validate a comment at the start of `source` and return its length.
pub(crate) fn scan_comment(source: &str, pos: Position) -> Result<Scanned> {
    if !source.starts_with('%') {
        return Err(Error::new(ErrorKind::InvalidCommentToken, pos));
    }
    let end = source.find('\n').unwrap_or(source.len());
    Ok(Scanned::new(TokenKind::Comment, end))
}

/// Decode a comment token's value (the text after the leading `%`).
pub(crate) fn decode_comment(text: &str) -> &str {
    &text[1..]
}
