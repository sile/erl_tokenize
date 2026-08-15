use crate::TokenKind;
use crate::escape;
use crate::lex::Scanned;
use crate::{Error, ErrorKind, Position, Result};

/// Validate a character literal at the start of `source` and return its
/// length.
pub(crate) fn scan_char(source: &str, pos: Position) -> Result<Scanned> {
    let mut chars = source.char_indices();
    chars
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidCharToken, pos))?;
    let (i, c) = chars
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidCharToken, pos))?;
    let end = if c == '\\' {
        let mut chars = chars.peekable();
        escape::parse_escaped_char(pos.step_by_width(i + 1), &mut chars)?;
        chars.peek().map(|&(i, _)| i).unwrap_or(source.len())
    } else {
        i + c.len_utf8()
    };
    Ok(Scanned::new(TokenKind::Char, end))
}

/// Decode a character token's value (`$X` or `$\...`) from validated text.
pub(crate) fn decode_char(text: &str) -> char {
    let after_dollar = &text[1..];
    let mut chars = after_dollar.char_indices().peekable();
    let (_, first) = chars.next().expect("scanner validated char payload");
    if first == '\\' {
        escape::parse_escaped_char(Position::new(), &mut chars).expect("scanner validated escape")
    } else {
        first
    }
}
