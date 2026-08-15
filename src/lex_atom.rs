use std::borrow::Cow;

use crate::TokenKind;
use crate::charset;
use crate::escape;
use crate::lex::Scanned;
use crate::{Error, ErrorKind, Position, Result};

/// Validate an atom token at the start of `source` and return its length.
pub(crate) fn scan_atom(source: &str, pos: Position) -> Result<Scanned> {
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidAtomToken, pos))?;
    if head == '\'' {
        let inner_end = escape::find_quotation_end(pos, &source[1..], '\'')?;
        Ok(Scanned::new(TokenKind::Atom, 1 + inner_end + 1))
    } else {
        if !charset::is_atom_head_char(head) {
            return Err(Error::new(ErrorKind::InvalidAtomToken, pos));
        }
        let mut end = head.len_utf8();
        for c in source[end..].chars() {
            if !charset::is_atom_non_head_char(c) {
                break;
            }
            end += c.len_utf8();
        }
        Ok(Scanned::new(TokenKind::Atom, end))
    }
}

/// Decode an atom token's value from its validated text.
///
/// Bare atoms borrow the text directly. Quoted atoms drop the outer
/// quotes; the content is borrowed when no escape sequences appear and
/// owned when escape decoding is required.
pub(crate) fn decode_atom(text: &str) -> Cow<'_, str> {
    if let Some(after_open) = text.strip_prefix('\'') {
        // `after_open` still contains the closing quote; parse_quotation
        // walks up to it while decoding escapes.
        let (v, _) = escape::parse_quotation(Position::new(), after_open, '\'')
            .expect("scanner validated atom quotation");
        v
    } else {
        Cow::Borrowed(text)
    }
}
