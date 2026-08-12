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

pub fn parse_quotation(
    pos: Position,
    input: &str,
    terminator: char,
) -> Result<(Cow<'_, str>, usize)> {
    let maybe_end = input
        .find(terminator)
        .ok_or_else(|| Error::no_closing_quotation(pos.clone()))?;
    let maybe_escaped = unsafe { input.get_unchecked(0..maybe_end).contains('\\') };
    if maybe_escaped {
        let (s, end) = parse_quotation_owned(pos, input, terminator)?;
        Ok((Cow::Owned(s), end))
    } else {
        let slice = unsafe { input.get_unchecked(0..maybe_end) };
        Ok((Cow::Borrowed(slice), maybe_end))
    }
}

fn parse_quotation_owned(pos: Position, input: &str, terminator: char) -> Result<(String, usize)> {
    let mut buf = String::new();
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            let c = parse_escaped_char(pos.clone() + 1 + i, &mut chars)?;
            buf.push(c);
        } else if c == terminator {
            return Ok((buf, i));
        } else {
            buf.push(c);
        }
    }
    Err(Error::no_closing_quotation(pos))
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
            let buf = if c == '{' {
                let mut buf = String::new();
                let mut closed = false;
                for (_, c) in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    buf.push(c);
                }
                if !closed {
                    return Err(error());
                }
                if buf.is_empty() {
                    return Err(error());
                }
                buf
            } else {
                let mut buf = String::with_capacity(2);
                buf.push(c);
                buf.push(chars.next().map(|(_, c)| c).ok_or_else(error)?);
                buf
            };
            let code: u32 = u32::from_str_radix(&buf, 16).ok().ok_or_else(error)?;
            char::from_u32(code).ok_or_else(error)
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
