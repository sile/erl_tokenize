use crate::{Error, Position, Result};
use std::borrow::Cow;
use std::char;
use std::iter::Peekable;

pub fn is_atom_head_char(c: char) -> bool {
    if let 'a'..='z' = c {
        true
    } else {
        c.is_lowercase() && c.is_alphabetic()
    }
}

pub fn is_atom_non_head_char(c: char) -> bool {
    match c {
        '@' | '_' | '0'..='9' => true,
        _ => c.is_alphabetic(),
    }
}

pub fn is_variable_head_char(c: char) -> bool {
    matches!(c, 'A'..='Z' | '_')
}

pub fn is_variable_non_head_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '@' | '_' | '0'..='9')
}

/// Walk a quoted region and return the byte index of the closing
/// terminator, validating any escape sequences along the way.
///
/// The scanner in [`crate::lex`] uses this to determine token boundaries
/// without allocating; [`parse_quotation`] uses it to locate the
/// terminator before decoding.
pub(crate) fn find_quotation_end(pos: Position, input: &str, terminator: char) -> Result<usize> {
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            parse_escaped_char(pos.clone() + 1 + i, &mut chars)?;
        } else if c == terminator {
            return Ok(i);
        }
    }
    Err(Error::no_closing_quotation(pos))
}

pub fn parse_quotation(
    pos: Position,
    input: &str,
    terminator: char,
) -> Result<(Cow<'_, str>, usize)> {
    let end = find_quotation_end(pos.clone(), input, terminator)?;
    let inner = unsafe { input.get_unchecked(0..end) };
    if inner.contains('\\') {
        let decoded = decode_quotation_content(pos, inner);
        Ok((Cow::Owned(decoded), end))
    } else {
        Ok((Cow::Borrowed(inner), end))
    }
}

/// Decode a quoted region's escaped content into an owned string. Assumes
/// the input has already been validated by [`find_quotation_end`] (i.e.,
/// every `\` introduces a well-formed escape sequence and the input does
/// not contain the terminator character unescaped).
fn decode_quotation_content(pos: Position, input: &str) -> String {
    let mut buf = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            let c = parse_escaped_char(pos.clone() + 1 + i, &mut chars)
                .expect("scanner already validated escape");
            buf.push(c);
        } else {
            buf.push(c);
        }
    }
    buf
}

// https://www.erlang.org/doc/system/data_types.html#escape-sequences
pub fn parse_escaped_char<I>(pos: Position, chars: &mut Peekable<I>) -> Result<char>
where
    I: Iterator<Item = (usize, char)>,
{
    let error = || Error::invalid_escaped_char(pos.clone());
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

/// Appends the Erlang-valid escape sequence for `c` to `buf`.
///
/// The output can be parsed back by [`parse_escaped_char`]. It is based on
/// `char::escape_debug` with two rewrites:
///
/// - `\0` is rewritten to `\x{0}`. A `\0` followed by an octal digit would
///   merge into a single escape (`\01` parses as one character); the same
///   merge can also happen across token boundaries in a token stream
///   (`$\0` followed by `7` rescans as `$\07`), so the unambiguous form is
///   emitted unconditionally.
/// - `\u{...}` is rewritten to `\x{...}`, since Erlang has no `\u{...}`
///   escape.
pub fn push_escaped_char(buf: &mut String, c: char) {
    let start = buf.len();
    buf.extend(c.escape_debug());
    if &buf[start..] == "\\0" {
        buf.truncate(start);
        buf.push_str("\\x{0}");
    } else if buf[start..].starts_with("\\u{") {
        buf.replace_range(start..start + 2, "\\x");
    }
}

/// Strip underscore separators from a numeric literal chunk that the
/// scanner has already validated. The result is safe to feed to
/// [`str::parse`] / [`i64::from_str_radix`] / etc.
pub(crate) fn strip_underscores(s: &str) -> String {
    s.chars().filter(|c| *c != '_').collect()
}
