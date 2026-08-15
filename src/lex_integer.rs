use crate::TokenKind;
use crate::charset;
use crate::lex::Scanned;
use crate::{Error, ErrorKind, Position, Result};

/// Validate an integer literal at the start of `source` and return its
/// length.
pub(crate) fn scan_integer(source: &str, pos: Position) -> Result<Scanned> {
    let mut end = 0;
    let mut has_radix = false;
    let mut radix: u32 = 10;
    let mut needs_digit = true;
    let mut radix_digits: u32 = 0;
    for (i, c) in source.char_indices() {
        if c == '#' && !has_radix && !needs_digit {
            if !(1 < radix_digits && radix_digits < 37) {
                return Err(Error::new(ErrorKind::InvalidIntegerToken, pos));
            }
            radix = radix_digits;
            has_radix = true;
            needs_digit = true;
            end = i + 1;
        } else if c.is_digit(radix) {
            if !has_radix {
                let d = c.to_digit(10).expect("ascii digit");
                radix_digits = radix_digits
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(d))
                    .unwrap_or(u32::MAX);
            }
            needs_digit = false;
            end = i + c.len_utf8();
        } else if c == '_' && !needs_digit {
            needs_digit = true;
            end = i + 1;
        } else {
            break;
        }
    }
    if needs_digit {
        return Err(Error::new(ErrorKind::InvalidIntegerToken, pos));
    }
    // Reject a trailing namechar: erl_scan's `scan_number` and
    // `scan_based_num` both return `{illegal,integer}` when the digit
    // run is followed by `?NAMECHAR` (`12abc`, `1e2`, `16#Fg`, ...).
    if let Some(c) = source[end..].chars().next()
        && charset::is_namechar(c)
    {
        return Err(Error::new(ErrorKind::InvalidIntegerToken, pos));
    }
    Ok(Scanned::new(TokenKind::Integer, end))
}

/// Decode an integer token's value from its validated text.
///
/// Erlang integer literals are always non-negative (unary `-` is a
/// separate token), so the value fits in `u64` whenever it does not
/// overflow. Returns `Some(value)` in range, `None` when it exceeds
/// `u64::MAX` (checked, never wrapped).
pub(crate) fn decode_integer(text: &str) -> Option<u64> {
    let (radix, digits_slice) = if let Some(hash) = text.find('#') {
        let radix: u32 = strip_underscores(&text[..hash])
            .parse()
            .expect("scanner validated radix");
        (radix, &text[hash + 1..])
    } else {
        (10u32, text)
    };
    let cleaned = strip_underscores(digits_slice);
    if radix == 10 {
        cleaned.parse::<u64>().ok()
    } else {
        u64::from_str_radix(&cleaned, radix).ok()
    }
}

/// Strip underscore separators from a numeric literal chunk that the
/// scanner has already validated. The result is safe to feed to
/// [`str::parse`] / [`i64::from_str_radix`] / etc.
pub(crate) fn strip_underscores(s: &str) -> String {
    s.chars().filter(|c| *c != '_').collect()
}
