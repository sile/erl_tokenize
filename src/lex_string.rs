use std::borrow::Cow;

use crate::TokenKind;
use crate::escape;
use crate::lex::Scanned;
use crate::lex_sigil;
use crate::{Error, ErrorKind, Position, Result};

/// Validate a string literal at the start of `source` and return its
/// length.
///
/// Accepts both ordinary `"..."` and triple-quoted `"""..."""` forms.
pub(crate) fn scan_string(source: &str, pos: Position) -> Result<Scanned> {
    let (end, is_triple) = scan_string_body(source, pos, None)?;
    // Adjacent string literals without intervening whitespace are rejected
    // only for the ordinary (non-triple) form to match `erl_scan`.
    if !is_triple && source.get(end..end + 1) == Some("\"") {
        let pos = pos.step_by_text(&source[0..end]);
        return Err(Error::new(ErrorKind::AdjacentStringLiterals, pos));
    }
    Ok(Scanned::new(TokenKind::String, end))
}

/// Scan a `"..."` or `"""..."""` string body and return `(length,
/// is_triple)` without applying the adjacent-string rejection rule.
///
/// `prefix` is the sigil prefix (`None` for a plain string). It selects the
/// verbatim/escape-processing behaviour, which differs between the two
/// forms in `erl_scan`:
///
/// * Ordinary `"..."`: empty, `s`, and `b` prefixes are non-verbatim
///   (escapes decoded); every other prefix (and a plain string) is
///   verbatim. See [`lex_sigil::is_verbatim_sigil_prefix`].
/// * Triple-quoted `"""..."""`: only `s` and `b` are non-verbatim; the
///   empty prefix, every other prefix, and a plain string are all verbatim
///   (`erl_scan`'s `scan_tqstring` classifies `SigilType` `b`/`s` as
///   non-verbatim and everything else as verbatim).
pub(crate) fn scan_string_body(
    source: &str,
    pos: Position,
    prefix: Option<&str>,
) -> Result<(usize, bool)> {
    if source.starts_with(r#"""""#) {
        let verbatim = !matches!(prefix, Some("b") | Some("s"));
        Ok((scan_triple_quoted(source, pos, verbatim)?, true))
    } else {
        if !source.starts_with('"') {
            return Err(Error::new(ErrorKind::InvalidStringToken, pos));
        }
        let verbatim = prefix.is_some_and(lex_sigil::is_verbatim_sigil_prefix);
        let inner_end = if verbatim {
            escape::find_verbatim_quotation_end(pos, &source[1..], '"')?
        } else {
            escape::find_quotation_end(pos, &source[1..], '"')?
        };
        Ok((1 + inner_end + 1, false))
    }
}

/// Locate the closing delimiter of a triple-quoted string body.
///
/// `body_start` is the byte offset of the character immediately after the
/// LF that follows the opening delimiter, and `quote_count` is the number
/// of `"` characters in that opening delimiter. Returns `(indent,
/// end_line_start, end_line_end)` on success, where:
///
/// * `indent` is the whitespace column count on the closing line (`0`
///   means the closer sits flush against column 1);
/// * `end_line_start` is the byte offset of the first character of the
///   closing line (i.e. the character just past the preceding LF);
/// * `end_line_end` is the byte offset just past the last `"` of the
///   closing delimiter.
///
/// The closer is only accepted when it consists of `quote_count`
/// contiguous `"` characters on a line otherwise containing only
/// whitespace, matching `erl_scan`'s `scan_tqstring_lines` rules — a run
/// like `""" ""` does not close a 3-quote string because the space
/// interrupts the quote run.
fn find_triple_quoted_closer(
    source: &str,
    body_start: usize,
    quote_count: usize,
) -> Option<(usize, usize, usize)> {
    let mut indent = 0usize;
    let mut end_line_start = body_start;
    let mut end_line_end = body_start;
    // `remaining` is the number of quote characters still needed to close.
    // `on_end_line` tracks whether the current line is still eligible to be
    // the closing line — a non-quote non-whitespace character disqualifies
    // it, and so does whitespace *after* a partial quote run.
    let mut remaining = quote_count;
    let mut on_end_line = true;
    for c in source[body_start..].chars() {
        end_line_end += c.len_utf8();
        if c == '\n' {
            indent = 0;
            remaining = quote_count;
            on_end_line = true;
            end_line_start = end_line_end;
        } else if on_end_line && c == '"' {
            remaining -= 1;
            if remaining == 0 {
                return Some((indent, end_line_start, end_line_end));
            }
        } else if c.is_ascii_whitespace() {
            if remaining == quote_count {
                // Still in the leading indent of the line.
                indent += 1;
            } else if on_end_line {
                // A partial quote run followed by whitespace is not a
                // valid closer.
                on_end_line = false;
            }
        } else {
            on_end_line = false;
        }
    }
    None
}

/// Scan a triple-quoted string literal and return its byte length.
///
/// Mirrors [`decode_triple_quoted`] but without building the decoded
/// content. When `verbatim` is false, each content line's `\` escapes are
/// validated (matching erl_scan, which rejects malformed escapes in
/// non-verbatim sigil triple-quoted strings).
fn scan_triple_quoted(source: &str, pos: Position, verbatim: bool) -> Result<usize> {
    let mut quote_count = 0usize;
    let mut chars = source.chars().peekable();
    let mut start_line_end = 0usize;

    while let Some(&c) = chars.peek() {
        if c == '"' {
            quote_count += 1;
            start_line_end += chars.next().expect("peeked").len_utf8();
        } else {
            break;
        }
    }

    let mut start_line_end_found = false;
    for c in chars {
        start_line_end += c.len_utf8();
        if c == '\n' {
            start_line_end_found = true;
            break;
        } else if !c.is_ascii_whitespace() {
            return Err(Error::new(ErrorKind::InvalidStringToken, pos));
        }
    }
    if !start_line_end_found {
        return Err(Error::new(ErrorKind::NoClosingQuotation, pos));
    }

    let (indent, end_line_start, end_line_end) =
        find_triple_quoted_closer(source, start_line_end, quote_count)
            .ok_or_else(|| Error::new(ErrorKind::NoClosingQuotation, pos))?;

    // An indented closer with no body lines has `end_line_start ==
    // start_line_end`; `saturating_sub` keeps the range well-formed
    // (decode_triple_quoted uses the same formula).
    let body_end = end_line_start.saturating_sub(1).max(start_line_end);
    let body = &source[start_line_end..body_end];
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // Strip `indent` leading whitespace columns; a non-whitespace
        // character within the indent is an error. Indent chars are all
        // ASCII whitespace (1 byte each), so `line[indent..]` is a valid
        // byte slice covering the post-indent portion of the line.
        if indent > 0 {
            for (i, c) in line.chars().enumerate() {
                if i >= indent {
                    break;
                }
                if !c.is_ascii_whitespace() {
                    return Err(Error::new(ErrorKind::InvalidStringToken, pos));
                }
            }
        }
        if !verbatim {
            let stripped = &line[indent..];
            if stripped.contains('\\') {
                escape::validate_escapes(pos, escape::strip_line_continuation(stripped))?;
            }
        }
    }

    Ok(end_line_end)
}

/// Decode a string token's value from its validated text.
///
/// Handles both the regular `"..."` form (borrowed when the content has
/// no escape sequences) and the triple-quoted `"""..."""` form (borrowed
/// when no indentation stripping is required).
pub(crate) fn decode_string(text: &str) -> Cow<'_, str> {
    if text.starts_with(r#"""""#) {
        decode_triple_quoted(text, true)
    } else {
        let after_open = &text[1..];
        let (v, _) = escape::parse_quotation(Position::new(), after_open, '"')
            .expect("scanner validated string quotation");
        v
    }
}

/// Decode a triple-quoted string's body from validated text.
///
/// Borrowed when the closing line has no indentation and there are no
/// escapes to decode (the body is a contiguous slice of the source); owned
/// when indentation must be stripped from each content line or non-verbatim
/// escapes must be decoded.
pub(crate) fn decode_triple_quoted(text: &str, verbatim: bool) -> Cow<'_, str> {
    // Count the opening quote run.
    let mut quote_count = 0usize;
    let mut idx = 0usize;
    for c in text.chars() {
        if c == '"' {
            quote_count += 1;
            idx += c.len_utf8();
        } else {
            break;
        }
    }

    // Skip anything (whitespace) up to and including the first LF; the
    // scanner already verified only ASCII whitespace precedes it.
    let start_line_end = text[idx..]
        .find('\n')
        .map(|i| idx + i + 1)
        .expect("scanner validated opening newline");

    let (indent, end_line_start, _end_line_end) =
        find_triple_quoted_closer(text, start_line_end, quote_count)
            .expect("scanner validated triple-quoted closer");

    let body_end = end_line_start.saturating_sub(1).max(start_line_end);
    let body = &text[start_line_end..body_end];
    // A trailing `\r` is the CR of the last content line's CRLF line
    // ending. erl_scan strips just this one CR from the last content line
    // (intermediate lines keep theirs), so remove it from the body, which
    // does not include a trailing LF.
    let body = body.strip_suffix('\r').unwrap_or(body);

    // Decode escapes per line only for non-verbatim content. When the
    // content needs neither indentation stripping nor escape decoding, the
    // body is a contiguous slice of the source and can be borrowed.
    if indent == 0 && (verbatim || !body.contains('\\')) {
        return Cow::Borrowed(body);
    }
    let mut value = String::with_capacity(body.len());
    let mut first = true;
    for line in body.split('\n') {
        if !first {
            value.push('\n');
        }
        first = false;
        // Strip `indent` leading columns, then decode escapes (matching
        // erl_scan's "indent stripping, then per-line escape decoding,
        // then line joining" order). Indent chars are all ASCII
        // whitespace (validated by the scanner), so byte slicing on
        // `indent` is safe when the line has at least `indent` bytes;
        // shorter blank lines are treated as empty.
        let stripped: &str = if indent == 0 {
            line
        } else {
            line.get(indent..).unwrap_or("")
        };
        if verbatim {
            value.push_str(stripped);
        } else {
            let stripped = escape::strip_line_continuation(stripped);
            let decoded = escape::decode_quotation_content(Position::new(), stripped);
            value.push_str(&decoded);
        }
    }
    Cow::Owned(value)
}

/// Decode the content between the delimiters of an ordinary (non-triple)
/// quoted region. Non-verbatim content borrows when it has no `\` escapes
/// and is otherwise decoded into an owned `String`; verbatim content
/// always borrows the raw slice.
pub(crate) fn decode_regular_string(inner: &str, verbatim: bool) -> Cow<'_, str> {
    if verbatim || !inner.contains('\\') {
        Cow::Borrowed(inner)
    } else {
        Cow::Owned(escape::decode_quotation_content(Position::new(), inner))
    }
}
