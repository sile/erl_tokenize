use crate::TokenKind;
use crate::charset;
use crate::lex::Scanned;
use crate::{Error, ErrorKind, Position, Result};

/// Validate a variable at the start of `source` and return its length.
pub(crate) fn scan_variable(source: &str, pos: Position) -> Result<Scanned> {
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidVariableToken, pos))?;
    if !charset::is_variable_head_char(head) {
        return Err(Error::new(ErrorKind::InvalidVariableToken, pos));
    }
    let mut end = head.len_utf8();
    for c in source[end..].chars() {
        if !charset::is_variable_non_head_char(c) {
            break;
        }
        end += c.len_utf8();
    }
    Ok(Scanned::new(TokenKind::Variable, end))
}
