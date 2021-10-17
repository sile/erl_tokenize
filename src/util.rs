use crate::{Error, Position, Result};
use num::Num;
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

// http://erlang.org/doc/reference_manual/data_types.html#id76758
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
            Ok((c as u32 % 32) as u8 as char)
        }
        'x' => {
            let (_, c) = chars.next().ok_or_else(error)?;
            let buf = if c == '{' {
                chars.map(|(_, c)| c).take_while(|c| *c != '}').collect()
            } else {
                let mut buf = String::with_capacity(2);
                buf.push(c);
                buf.push(chars.next().map(|(_, c)| c).ok_or_else(error)?);
                buf
            };
            let code: u32 = Num::from_str_radix(&buf, 16).ok().ok_or_else(error)?;
            char::from_u32(code).ok_or_else(error)
        }
        c @ '0'..='7' => {
            let mut limit = 2;
            let mut n = c.to_digit(8).expect("unreachable");
            while let Some((_, '0'..='7')) = chars.peek().cloned() {
                n = (n * 8) + c.to_digit(8).expect("unreachable");
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
