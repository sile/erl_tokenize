use std::borrow::Cow;

use crate::TokenKind;
use crate::charset;
use crate::escape;
use crate::lex::Scanned;
use crate::lex_string;
use crate::{Error, ErrorKind, Position, Result};

/// Validate a sigil string literal at the start of `source` and return its
/// length.
pub(crate) fn scan_sigil_string(source: &str, pos: Position) -> Result<Scanned> {
    if !source.starts_with('~') {
        return Err(Error::new(ErrorKind::InvalidSigilStringToken, pos));
    }
    let mut offset = 1;
    for c in source[offset..].chars() {
        if !charset::is_atom_non_head_char(c) {
            break;
        }
        offset += c.len_utf8();
    }
    let prefix = &source[1..offset];
    let verbatim = is_verbatim_sigil_prefix(prefix);
    let open = source[offset..]
        .chars()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidSigilStringToken, pos))?;
    let content_end = if open == '"' {
        // Reuse the string-body scanner so that both single- and
        // triple-quoted forms behave identically for sigils, minus the
        // adjacent-string rejection (which is checked later against the
        // sigil suffix).
        let (len, _is_triple) = lex_string::scan_string_body(
            &source[offset..],
            pos.step_by_width(offset),
            Some(prefix),
        )?;
        offset + len
    } else {
        let close = match open {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '<' => '>',
            '/' | '|' | '\'' | '`' | '#' => open,
            _ => return Err(Error::new(ErrorKind::InvalidSigilStringToken, pos)),
        };
        let inner_pos = pos.step_by_width(offset + 1);
        let content = &source[offset + 1..];
        let inner_end = if verbatim {
            escape::find_verbatim_quotation_end(inner_pos, content, close)?
        } else {
            escape::find_quotation_end(inner_pos, content, close)?
        };
        offset + 1 + inner_end + 1
    };
    let mut end = content_end;
    for c in source[end..].chars() {
        if !charset::is_atom_non_head_char(c) {
            break;
        }
        end += c.len_utf8();
    }
    // A sigil with an empty suffix followed by `"` is an adjacent-string
    // error, matching `erl_scan`'s `scan_string_concat` rule; a non-empty
    // suffix separates the tokens and no error is raised.
    if end == content_end && source.get(end..end + 1) == Some("\"") {
        let pos = pos.step_by_text(&source[0..end]);
        return Err(Error::new(ErrorKind::AdjacentStringLiterals, pos));
    }
    Ok(Scanned::new(TokenKind::SigilString, end))
}

/// Return `true` when the sigil prefix indicates a verbatim string, i.e.
/// escape sequences inside the content are preserved as-is.
///
/// `erl_scan` classifies the empty prefix (`~"..."`), `b` (`~b"..."`),
/// and `s` (`~s"..."`) as non-verbatim; every other prefix — `~B`, `~S`,
/// `~foo`, `~X`, and so on — is verbatim.
pub(crate) fn is_verbatim_sigil_prefix(prefix: &str) -> bool {
    !matches!(prefix, "" | "b" | "s")
}

/// Split a sigil string's validated text into its `~<prefix><open>content
/// <close><suffix>` pieces.
///
/// `prefix` and `suffix` always borrow from `text`; `content` borrows when
/// no escape sequences or triple-quoted indentation appear inside it.
pub(crate) fn decode_sigil(text: &str) -> (&str, Cow<'_, str>, &str) {
    let mut prefix_end = 1; // skip leading `~`
    for c in text[prefix_end..].chars() {
        if !charset::is_atom_non_head_char(c) {
            break;
        }
        prefix_end += c.len_utf8();
    }
    let prefix = &text[1..prefix_end];
    let verbatim = is_verbatim_sigil_prefix(prefix);
    let open = text[prefix_end..]
        .chars()
        .next()
        .expect("scanner validated sigil delimiter");
    let (content, content_end) = if open == '"' {
        // The content is itself a full (regular or triple-quoted) string.
        let sub = &text[prefix_end..];
        let (len, is_triple) = lex_string::scan_string_body(sub, Position::new(), Some(prefix))
            .expect("scanner validated sigil string");
        let body = &sub[..len];
        let value = if is_triple {
            // Triple-quoted sigils follow a different verbatim rule than
            // single-quoted ones: only `b`/`s` are non-verbatim, and the
            // empty prefix is verbatim (matching `erl_scan`'s
            // `scan_tqstring`).
            lex_string::decode_triple_quoted(body, !matches!(prefix, "b" | "s"))
        } else {
            lex_string::decode_regular_string(&body[1..len - 1], verbatim)
        };
        (value, prefix_end + len)
    } else {
        let close = match open {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '<' => '>',
            other => other,
        };
        let content_start = prefix_end + 1;
        let content_len = if verbatim {
            escape::find_verbatim_quotation_end(Position::new(), &text[content_start..], close)
                .expect("scanner validated sigil close")
        } else {
            escape::find_quotation_end(Position::new(), &text[content_start..], close)
                .expect("scanner validated sigil close")
        };
        let inner = &text[content_start..content_start + content_len];
        let value = lex_string::decode_regular_string(inner, verbatim);
        (value, content_start + content_len + 1)
    };
    let suffix = &text[content_end..];
    (prefix, content, suffix)
}
