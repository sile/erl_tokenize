use crate::TokenKind;
use crate::charset;
use crate::lex::Scanned;
use crate::lex_integer;
use crate::{Error, ErrorKind, Position, Result};

/// Look ahead past digits and underscores to check whether an ASCII-digit
/// run introduces a float or an integer.
///
/// Recognises both `<digits>.<x>` (decimal float) and
/// `<radix>#<radix-digits>.<x>` (radix float), where `<x>` is either a
/// digit (a real fractional part) or a namechar (`_`, `@`, `A-Z`,
/// `a-z`). The namechar case is intentionally routed here even though
/// it never yields a valid float: `scan_float_decimal` /
/// `scan_float_radix` will fail via their "dot must be followed by a
/// digit" rule, producing `InvalidFloatToken` for shapes like `1.e2`
/// and `16#ff._` — matching erl_scan's `scan_number` /
/// `scan_based_num` `.`-then-`?NAMECHAR` reject clauses. Any other
/// shape (`<digits>.` followed by whitespace / symbol / EOF, or
/// `<digits>#<digits>` with no dot) is left to the integer scanner.
pub(crate) fn looks_like_float(source: &str) -> bool {
    let bytes = source.as_bytes();
    // Skip the initial digit / underscore run.
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        if b.is_ascii_digit() || b == b'_' {
            i += 1;
        } else {
            break;
        }
    }
    // Decimal float: `<digits>.<namechar>` (digits are namechars too).
    if bytes.get(i) == Some(&b'.') {
        return bytes
            .get(i + 1)
            .copied()
            .is_some_and(charset::is_ascii_namechar);
    }
    // Radix float: `<digits>#<radix-digits>.<namechar>`.
    if bytes.get(i) == Some(&b'#') {
        let mut j = i + 1;
        while let Some(&b) = bytes.get(j) {
            if b.is_ascii_alphanumeric() || b == b'_' {
                j += 1;
            } else {
                break;
            }
        }
        return bytes.get(j) == Some(&b'.')
            && bytes
                .get(j + 1)
                .copied()
                .is_some_and(charset::is_ascii_namechar);
    }
    false
}

/// Validate a float literal at the start of `source` and return its length.
///
/// Rejects overflow (a value that decodes to a non-finite `f64`) with
/// the same [`ErrorKind::InvalidFloatToken`] as syntactic errors,
/// matching `erl_scan`'s behavior. Underflow is not an error: it
/// collapses to `0.0`, which is finite.
pub(crate) fn scan_float(source: &str, pos: Position) -> Result<Scanned> {
    let scanned = if is_based(source) {
        scan_float_radix(source, pos)?
    } else {
        scan_float_decimal(source, pos)?
    };
    if !decode_float(&source[..scanned.len]).is_finite() {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    Ok(scanned)
}

fn scan_float_decimal(source: &str, pos: Position) -> Result<Scanned> {
    let mut idx = read_digit_run(source, 0, pos)?;
    let after_int = &source[idx..];
    let mut chars = after_int.chars();
    if chars.next() != Some('.') {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    idx += 1;
    idx = read_digit_run(source, idx, pos)?;
    if matches!(source[idx..].chars().next(), Some('e' | 'E')) {
        idx += 1;
        if matches!(source[idx..].chars().next(), Some('+' | '-')) {
            idx += 1;
        }
        idx = read_digit_run(source, idx, pos)?;
    }
    // Reject a trailing namechar: erl_scan's `scan_fraction` and
    // `scan_exponent` both return `{illegal,float}` when the run ends
    // on `?NAMECHAR` (`1.5a`, `1.5e2a`, ...).
    if let Some(c) = source[idx..].chars().next()
        && charset::is_namechar(c)
    {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    Ok(Scanned::new(TokenKind::Float, idx))
}

fn is_based(source: &str) -> bool {
    for (i, c) in source.char_indices() {
        if matches!(c, '0'..='9' | '_') {
            continue;
        }
        if i > 0 && c == '#' {
            return true;
        }
        break;
    }
    false
}

/// Read a run of ASCII decimal digits with underscore separators, starting
/// at `start`. The run must start and end on a digit; a trailing or double
/// underscore is rejected.
fn read_digit_run(source: &str, start: usize, pos: Position) -> Result<usize> {
    let mut idx = start;
    let mut needs_digit = true;
    for (i, c) in source[start..].char_indices() {
        let at = start + i;
        match c {
            '0'..='9' => {
                needs_digit = false;
                idx = at + 1;
            }
            '_' => {
                if needs_digit {
                    return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
                }
                needs_digit = true;
                idx = at + 1;
            }
            _ => break,
        }
    }
    if needs_digit {
        Err(Error::new(ErrorKind::InvalidFloatToken, pos))
    } else {
        Ok(idx)
    }
}

/// Radix-based float form: `<radix>#<digits>.<digits>[#e<exp>]`.
fn scan_float_radix(source: &str, pos: Position) -> Result<Scanned> {
    let hash = source.find('#').expect("looks_like_float / is_based guard");
    let radix = parse_radix_digits(&source[..hash], pos)?;
    if !(1 < radix && radix < 37) {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    let mut idx = hash + 1;
    if idx >= source.len() {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    let (int_end, saw_dot) = read_radix_digit_run(source, idx, radix, pos, true)?;
    idx = int_end;
    if !saw_dot {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    let (frac_end, has_exp) = read_radix_digit_run(source, idx, radix, pos, false)?;
    idx = frac_end;
    if has_exp {
        if !source[idx..].starts_with(['e', 'E']) {
            return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
        }
        idx += 1;
        idx = read_exp_digit_run(source, idx, pos)?;
        // erl_scan's `scan_based_exponent` has no `?NAMECHAR` clause,
        // so a namechar after the exponent digits terminates the token
        // and starts the next one (`16#ff.ff#e1a` → `Float, Atom("a")`).
        // Skip the trailing-namechar check on this branch.
    } else {
        // Reject a trailing namechar on the fractional part when no
        // exponent follows: erl_scan's `scan_based_fraction` returns
        // `{illegal,float}` when the fractional run ends on `?NAMECHAR`
        // that is not a base digit (`16#ff._`, `16#ff.@`, ...).
        if let Some(c) = source[idx..].chars().next()
            && charset::is_namechar(c)
        {
            return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
        }
    }
    Ok(Scanned::new(TokenKind::Float, idx))
}

/// Parse the radix prefix (before `#`) as a decimal integer, allowing
/// underscore separators between digits.
fn parse_radix_digits(text: &str, pos: Position) -> Result<u32> {
    let mut value: u32 = 0;
    let mut has_digit = false;
    let mut prev_digit = false;
    for c in text.chars() {
        if let Some(d) = c.to_digit(10) {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(d))
                .unwrap_or(u32::MAX);
            has_digit = true;
            prev_digit = true;
        } else if c == '_' && prev_digit {
            prev_digit = false;
        } else {
            return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
        }
    }
    if !has_digit || !prev_digit {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    Ok(value)
}

/// Consume digits (with underscore separators) in a radix run. Returns the
/// byte index just past the last consumed digit and a flag indicating
/// whether the run ended on the terminator character (`.` for the integer
/// part on `expect_dot`, `#` for the fractional part).
fn read_radix_digit_run(
    source: &str,
    start: usize,
    radix: u32,
    pos: Position,
    expect_dot: bool,
) -> Result<(usize, bool)> {
    let mut idx = start;
    let mut is_prev_digit = false;
    let mut terminator = false;
    for (i, c) in source[start..].char_indices() {
        let at = start + i;
        if is_prev_digit && c == '_' {
            is_prev_digit = false;
            idx = at + 1;
            continue;
        }
        if expect_dot && is_prev_digit && c == '.' {
            idx = at + 1;
            terminator = true;
            break;
        }
        if !expect_dot && is_prev_digit && c == '#' {
            idx = at + 1;
            terminator = true;
            break;
        }
        if c.is_digit(radix) {
            is_prev_digit = true;
            idx = at + c.len_utf8();
        } else {
            break;
        }
    }
    if !is_prev_digit && !terminator {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    Ok((idx, terminator))
}

/// Consume the exponent digits (with a leading optional `-` and underscore
/// separators).
fn read_exp_digit_run(source: &str, start: usize, pos: Position) -> Result<usize> {
    let mut idx = start;
    let mut is_prev_digit = false;
    let mut saw_any = false;
    for (i, c) in source[start..].char_indices() {
        let at = start + i;
        if i == 0 && matches!(c, '-' | '+') {
            idx = at + 1;
            saw_any = true;
        } else if c.is_ascii_digit() {
            is_prev_digit = true;
            saw_any = true;
            idx = at + 1;
        } else if is_prev_digit && c == '_' {
            is_prev_digit = false;
            idx = at + 1;
        } else {
            break;
        }
    }
    if !saw_any || !is_prev_digit {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
    }
    Ok(idx)
}

/// Decode a float token's value from its validated text (either decimal
/// or radix-prefixed).
pub(crate) fn decode_float(text: &str) -> f64 {
    if let Some(hash) = text.find('#') {
        decode_radix_float(text, hash)
    } else {
        lex_integer::strip_underscores(text)
            .parse::<f64>()
            .expect("scanner validated decimal float")
    }
}

fn decode_radix_float(slice: &str, hash: usize) -> f64 {
    let radix: u32 = lex_integer::strip_underscores(&slice[..hash])
        .parse()
        .expect("scanner validated radix");
    let rest = &slice[hash + 1..];
    let dot = rest.find('.').expect("scanner validated dot");
    let int_part = &rest[..dot];
    let after_dot = &rest[dot + 1..];
    let (frac_part, exp_opt) = if let Some(second_hash) = after_dot.find('#') {
        (
            &after_dot[..second_hash],
            // Skip both `#` and the mandatory `e`.
            Some(&after_dot[second_hash + 2..]),
        )
    } else {
        (after_dot, None)
    };

    // erl_scan's `based_float_end` branches on the base: B=10 hands the
    // reconstructed decimal to `list_to_float` (equivalent to Rust's
    // `f64::from_str`), while B != 10 goes through
    // `N * math:pow(B, Exp - D)`. Mirror that split so both branches
    // match erl_scan bit-exactly.
    if radix == 10 {
        decode_radix_ten(int_part, frac_part, exp_opt)
    } else {
        decode_radix_other(int_part, frac_part, exp_opt, radix)
    }
}

/// Decode the B=10 branch by rebuilding `<int>.<frac>[e<exp>]` and
/// deferring to `f64::from_str`, mirroring erl_scan's
/// `Fcs = Ncs ++ ECs1` (which drops the `#` between the fraction and
/// the `e` while keeping the `e` itself) then `list_to_float(Fcs)`.
fn decode_radix_ten(int_part: &str, frac_part: &str, exp_opt: Option<&str>) -> f64 {
    let mut decimal = String::with_capacity(int_part.len() + 1 + frac_part.len() + 5);
    decimal.push_str(int_part);
    decimal.push('.');
    decimal.push_str(frac_part);
    if let Some(exp_str) = exp_opt {
        decimal.push('e');
        decimal.push_str(exp_str);
    }
    lex_integer::strip_underscores(&decimal)
        .parse::<f64>()
        .unwrap_or(f64::INFINITY)
}

/// Decode the B != 10 branch as `N * pow(B, Exp - D)`, first trimming
/// the fraction's trailing zeros and the integer's leading zeros the
/// same way erl_scan's `trim_float_zeros` does (a lone `0` on either
/// side is preserved, so `0.5` stays `0.5` and `1.0` stays `1.0`).
fn decode_radix_other(int_part: &str, frac_part: &str, exp_opt: Option<&str>, radix: u32) -> f64 {
    let int_stripped = lex_integer::strip_underscores(int_part);
    let frac_stripped = lex_integer::strip_underscores(frac_part);
    let int_trimmed = trim_leading_zeros_preserve_one(&int_stripped);
    let frac_trimmed = trim_trailing_zeros_preserve_one(&frac_stripped);
    let d = frac_trimmed.len();

    // erl_scan reads N from `list_to_integer(lists:delete($., Ncs1), B)`,
    // i.e. the trimmed digit run with `.` removed.
    let mut digits = String::with_capacity(int_trimmed.len() + frac_trimmed.len());
    digits.push_str(int_trimmed);
    digits.push_str(frac_trimmed);
    let Ok(mantissa) = u128::from_str_radix(&digits, radix) else {
        // Mantissa overflows u128. erl_scan uses arbitrary-precision
        // integers here, so this is a deliberate divergence: inputs
        // whose trimmed mantissa exceeds u128 surface as
        // `InvalidFloatToken` via `scan_float`'s `is_finite()` guard
        // instead of being decoded to a possibly-finite f64.
        return f64::INFINITY;
    };

    let exp_value: i32 = match exp_opt {
        Some(exp_str) => {
            let Ok(v) = lex_integer::strip_underscores(exp_str).parse::<i32>() else {
                // Same rationale as the mantissa overflow above.
                return f64::INFINITY;
            };
            v
        }
        None => 0,
    };
    let Some(scale_exp) = i32::try_from(d).ok().and_then(|d| exp_value.checked_sub(d)) else {
        // The trimmed fraction outran `i32::MAX`, or `Exp - D`
        // underflowed `i32::MIN`. Either way the magnitude collapses
        // to 0.0.
        return 0.0;
    };

    // `powf` mirrors erl_scan's `math:pow` (which routes through libm
    // `pow`); `powi` uses repeated squaring and can differ by an ULP.
    mantissa as f64 * (radix as f64).powf(f64::from(scale_exp))
}

fn trim_leading_zeros_preserve_one(s: &str) -> &str {
    let trimmed = s.trim_start_matches('0');
    if trimmed.is_empty() { "0" } else { trimmed }
}

fn trim_trailing_zeros_preserve_one(s: &str) -> &str {
    let trimmed = s.trim_end_matches('0');
    if trimmed.is_empty() { "0" } else { trimmed }
}
