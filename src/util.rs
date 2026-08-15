use std::borrow::Cow;
use std::iter::Peekable;

use crate::{Error, ErrorKind, Position, Result};

pub(crate) fn is_atom_head_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'ß'..='ö' | 'ø'..='ÿ')
}

pub(crate) fn is_atom_non_head_char(c: char) -> bool {
    matches!(
        c,
        'a'..='z' | 'A'..='Z' | '@' | '_' | '0'..='9'
            | 'À'..='Ö'
            | 'Ø'..='Þ'
            | 'ß'..='ö'
            | 'ø'..='ÿ'
    )
}

/// Match erl_scan's effective `?NAMECHAR` set: ASCII alphanumerics,
/// `_`, and `@`. Latin-1 letters are intentionally excluded: erl_scan's
/// macro attempts to include them but chains its Latin-1 clauses with
/// `andalso`, so `ß..ÿ ∩ À..Þ` collapses to the empty set and no
/// Latin-1 letter satisfies the guard in practice.
pub(crate) fn is_namechar(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '@')
}

pub(crate) fn is_variable_head_char(c: char) -> bool {
    // Matches erl_scan: ASCII `A-Z`, `_`, and Latin-1 uppercase letters
    // (`À..Þ` minus the multiplication sign `×`).
    matches!(c, 'A'..='Z' | '_' | 'À'..='Ö' | 'Ø'..='Þ')
}

pub(crate) fn is_variable_non_head_char(c: char) -> bool {
    // Matches erl_scan's `scan_name`: ASCII alphanumerics, `_`, `@`, and
    // Latin-1 letters (`À..Þ` minus `×`, `ß..ÿ` minus `÷`).
    matches!(
        c,
        'a'..='z'
            | 'A'..='Z'
            | '@'
            | '_'
            | '0'..='9'
            | 'À'..='Ö'
            | 'Ø'..='Þ'
            | 'ß'..='ö'
            | 'ø'..='ÿ'
    )
}

/// Walk a quoted region and return the byte index of the closing
/// terminator, validating any escape sequences along the way.
///
/// [`crate::lex`] uses this to determine token boundaries without
/// allocating; [`parse_quotation`] uses it to locate the terminator
/// before decoding the content.
pub(crate) fn find_quotation_end(pos: Position, input: &str, terminator: char) -> Result<usize> {
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            parse_escaped_char(pos.step_by_text(&input[..i]), &mut chars)?;
        } else if c == terminator {
            return Ok(i);
        }
    }
    Err(Error::new(ErrorKind::NoClosingQuotation, pos))
}

/// Verbatim variant of [`find_quotation_end`]: does not treat `\` as an
/// escape introducer, so the terminator matches the first literal
/// occurrence. Used for verbatim sigil strings (`~B"..."`, `~S"..."`
/// etc.) whose content preserves `\` as-is.
pub(crate) fn find_verbatim_quotation_end(
    pos: Position,
    input: &str,
    terminator: char,
) -> Result<usize> {
    for (i, c) in input.char_indices() {
        if c == terminator {
            return Ok(i);
        }
    }
    Err(Error::new(ErrorKind::NoClosingQuotation, pos))
}

/// Validate that every `\` in `input` introduces a well-formed escape
/// sequence, without requiring a terminator.
///
/// Unlike [`find_quotation_end`] (which also validates escapes), this
/// never looks for a closing terminator, so it is usable on content that
/// has no terminating character of its own (e.g. a single line of a
/// triple-quoted string body).
pub(crate) fn validate_escapes(pos: Position, input: &str) -> Result<()> {
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            parse_escaped_char(pos.step_by_text(&input[..i]), &mut chars)?;
        }
    }
    Ok(())
}

/// Strip one trailing `\` from `line` when the count of consecutive
/// trailing `\` characters is odd, meaning the last backslash is unpaired.
///
/// This models `erl_scan`'s line-continuation handling in non-verbatim
/// triple-quoted content: a `\` immediately before the raw LF that ends a
/// content line is consumed together with that LF (the LF still acts as
/// the content-line separator). Since the LF has already been consumed by
/// splitting the body on `\n`, only the dangling `\` remains at the end
/// of the line.
///
/// `\\` (an escaped backslash) has an even trailing count and is left
/// alone so [`parse_escaped_char`] can decode it as a single `\`.
pub(crate) fn strip_line_continuation(line: &str) -> &str {
    let trailing = line.bytes().rev().take_while(|b| *b == b'\\').count();
    if trailing % 2 == 1 {
        &line[..line.len() - 1]
    } else {
        line
    }
}

/// Locate the terminator and decode the quoted content into a `Cow`,
/// borrowing when no escape sequences are present.
pub(crate) fn parse_quotation(
    pos: Position,
    input: &str,
    terminator: char,
) -> Result<(Cow<'_, str>, usize)> {
    let end = find_quotation_end(pos, input, terminator)?;
    let inner = &input[..end];
    if inner.contains('\\') {
        let decoded = decode_quotation_content(pos, inner);
        Ok((Cow::Owned(decoded), end))
    } else {
        Ok((Cow::Borrowed(inner), end))
    }
}

/// Decode a quoted region's escaped content into an owned string.
/// Assumes the input has already been validated by
/// [`find_quotation_end`] (every `\` introduces a well-formed escape
/// sequence and the input does not contain the unescaped terminator).
pub(crate) fn decode_quotation_content(pos: Position, input: &str) -> String {
    let mut buf = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            let c = parse_escaped_char(pos.step_by_text(&input[..i]), &mut chars)
                .expect("scanner already validated escape");
            buf.push(c);
        } else {
            buf.push(c);
        }
    }
    buf
}

// https://www.erlang.org/doc/system/data_types.html#escape-sequences
pub(crate) fn parse_escaped_char<I>(pos: Position, chars: &mut Peekable<I>) -> Result<char>
where
    I: Iterator<Item = (usize, char)>,
{
    let error = || Error::new(ErrorKind::InvalidEscapedChar, pos);
    let (_, c) = chars.next().ok_or_else(error)?;
    match c {
        'b' => Ok(8 as char),   // Back Space
        'd' => Ok(127 as char), // Delete
        'e' => Ok(27 as char),  // Escape
        'f' => Ok(12 as char),  // Form Feed
        'n' => Ok('\n'),
        'r' => Ok('\r'),
        's' => Ok(' '),
        't' => Ok('\t'),
        'v' => Ok(11 as char), // Vertical Tabulation
        '^' => {
            let (_, c) = chars.next().ok_or_else(error)?;
            // Erlang's caret notation is defined only for these ASCII
            // characters (as of OTP 26, anything else is a syntax error;
            // `\^?` is Delete (127), not the `% 32` mapping of `_`).
            match c {
                '@' | '[' | '\\' | ']' | '^' | '_' | 'a'..='z' | 'A'..='Z' => {
                    Ok((c as u32 % 32) as u8 as char)
                }
                '?' => Ok(127 as char),
                _ => Err(error()),
            }
        }
        'x' => {
            let (_, c) = chars.next().ok_or_else(error)?;
            if c == '{' {
                let mut code: u32 = 0;
                let mut count = 0usize;
                let mut closed = false;
                for (_, c) in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    let d = c.to_digit(16).ok_or_else(error)?;
                    code = code.checked_mul(16).ok_or_else(error)?;
                    code = code.checked_add(d).ok_or_else(error)?;
                    count += 1;
                }
                if !closed || count == 0 {
                    return Err(error());
                }
                char::from_u32(code).ok_or_else(error)
            } else {
                let (_, c2) = chars.next().ok_or_else(error)?;
                let hi = c.to_digit(16).ok_or_else(error)?;
                let lo = c2.to_digit(16).ok_or_else(error)?;
                char::from_u32(hi * 16 + lo).ok_or_else(error)
            }
        }
        c @ '0'..='7' => {
            let mut limit = 2;
            let mut n = c.to_digit(8).expect("matched '0'..='7'");
            while let Some((_, d @ '0'..='7')) = chars.peek().cloned() {
                n = (n * 8) + d.to_digit(8).expect("matched '0'..='7'");
                let _ = chars.next();
                limit -= 1;
                if limit == 0 {
                    break;
                }
            }
            char::from_u32(n).ok_or_else(error)
        }
        _ => Ok(c),
    }
}

/// Strip underscore separators from a numeric literal chunk that the
/// scanner has already validated. The result is safe to feed to
/// [`str::parse`] / [`i64::from_str_radix`] / etc.
pub(crate) fn strip_underscores(s: &str) -> String {
    s.chars().filter(|c| *c != '_').collect()
}
