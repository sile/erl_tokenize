//! Internal lexical scanning.
//!
//! This module is the single source of truth for validating the lexical
//! shape of a token: which token kind starts at a byte position, where the
//! token ends, whether escape sequences and numeric grammar are well-formed,
//! and — for keywords, symbols, and whitespace — the enum value that the
//! textual form maps to.
//!
//! The scanners never allocate on the happy path: they return only a
//! [`Scanned`] value describing the token kind and its UTF-8 byte length.
//! Value construction (owned strings, decoded escapes, integer/float
//! parsing) is the caller's responsibility and happens against the slice
//! whose length is returned here.
//!
//! Public tokenization API in [`crate::token`] and [`crate::tokens`] wires
//! its entry points through this module so that no other place in the
//! crate re-implements the same lexical rules.

use std::borrow::Cow;

use crate::keyword::Keyword;
use crate::symbol::Symbol;
use crate::util;
use crate::{Error, Position, Result};

/// Kind of the token that a scanner recognized at the start of a source
/// slice.
///
/// Variants that carry an enum value expose the fact that keywords,
/// symbols, and whitespace are fully determined by their textual form: the
/// caller does not need to inspect the text again to recover the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScanKind {
    Atom,
    Char,
    Comment,
    Float,
    Integer,
    Keyword(Keyword),
    SigilString,
    String,
    Symbol(Symbol),
    Variable,
    Whitespace,
}

/// Result of scanning one token at the start of a source slice.
///
/// Only holds `Copy` data: no owned strings, no vectors, no boxed values.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Scanned {
    pub kind: ScanKind,
    pub len: usize,
}

impl Scanned {
    fn new(kind: ScanKind, len: usize) -> Self {
        Self { kind, len }
    }
}

/// Scan a single token at the start of `source`.
///
/// Any error returned from this function carries the resume position that
/// [`crate::scan_token`] passes back to the caller (`pos.step_by_char(first_char)`
/// on non-empty input, `pos` on empty).
pub(crate) fn scan_one(source: &str, pos: Position) -> Result<Scanned> {
    scan_one_impl(source, pos).map_err(|e| e.with_resume(resume_from(source, pos)))
}

/// Compute the resume position from the scan-start position and the first
/// character of the input at that position.
///
/// On non-empty input this advances by exactly one Unicode scalar value,
/// updating line and column via [`Position::step_by_char`]. On empty
/// input it returns `pos` unchanged (there is nothing to advance past).
fn resume_from(source: &str, pos: Position) -> Position {
    match source.chars().next() {
        Some(c) => pos.step_by_char(c),
        None => pos,
    }
}

fn scan_one_impl(source: &str, pos: Position) -> Result<Scanned> {
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::missing_token(pos))?;
    match head {
        ' ' | '\t' | '\r' | '\n' | '\u{A0}' => scan_whitespace(source, pos),
        'A'..='Z' | '_' => scan_variable(source, pos),
        '0'..='9' => {
            if looks_like_float(source) {
                scan_float(source, pos)
            } else {
                scan_integer(source, pos)
            }
        }
        '$' => scan_char(source, pos),
        '"' => scan_string(source, pos),
        '\'' => scan_atom(source, pos),
        '%' => scan_comment(source, pos),
        '~' => scan_sigil_string(source, pos),
        _ => {
            // `is_alphabetic` mirrors the historical top-level dispatch:
            // any non-lowercase alphabetic reaches `scan_atom` and then
            // fails with `InvalidAtomToken` inside it. Other non-atom-head
            // characters (digits, symbols, punctuation) go to
            // `scan_symbol`.
            if head.is_alphabetic() {
                let atom = scan_atom(source, pos)?;
                let text = &source[..atom.len];
                if let Some(k) = keyword_from_text(text) {
                    Ok(Scanned::new(ScanKind::Keyword(k), atom.len))
                } else {
                    Ok(atom)
                }
            } else {
                scan_symbol(source, pos)
            }
        }
    }
}

/// Look ahead past digits and underscores to check whether an ASCII-digit
/// run introduces a float or an integer.
///
/// Recognises both `<digits>.<digit>` (decimal float) and
/// `<radix>#<radix-digits>.<radix-digit>` (radix float); anything else
/// (including `<digits>.` followed by a non-digit and `<digits>#<digits>`
/// with no dot) is left to the integer scanner.
fn looks_like_float(source: &str) -> bool {
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
    // Decimal float: `<digits>.<digit>`.
    if bytes.get(i) == Some(&b'.') {
        return bytes.get(i + 1).is_some_and(|c| c.is_ascii_digit());
    }
    // Radix float: `<digits>#<radix-digits>.<radix-digit>`.
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
            && bytes.get(j + 1).is_some_and(|c| c.is_ascii_alphanumeric());
    }
    false
}

/// Validate an atom token at the start of `source` and return its length.
pub(crate) fn scan_atom(source: &str, pos: Position) -> Result<Scanned> {
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::invalid_atom_token(pos))?;
    if head == '\'' {
        let inner_end = util::find_quotation_end(pos, &source[1..], '\'')?;
        Ok(Scanned::new(ScanKind::Atom, 1 + inner_end + 1))
    } else {
        if !util::is_atom_head_char(head) {
            return Err(Error::invalid_atom_token(pos));
        }
        let mut end = head.len_utf8();
        for c in source[end..].chars() {
            if !util::is_atom_non_head_char(c) {
                break;
            }
            end += c.len_utf8();
        }
        Ok(Scanned::new(ScanKind::Atom, end))
    }
}

/// Validate a character literal at the start of `source` and return its
/// length.
pub(crate) fn scan_char(source: &str, pos: Position) -> Result<Scanned> {
    let mut chars = source.char_indices();
    let (_, dollar) = chars.next().ok_or_else(|| Error::invalid_char_token(pos))?;
    if dollar != '$' {
        return Err(Error::invalid_char_token(pos));
    }
    let (i, c) = chars.next().ok_or_else(|| Error::invalid_char_token(pos))?;
    let end = if c == '\\' {
        let mut chars = chars.peekable();
        util::parse_escaped_char(pos.step_by_width(i + 1), &mut chars)?;
        chars.peek().map(|&(i, _)| i).unwrap_or(source.len())
    } else {
        i + c.len_utf8()
    };
    Ok(Scanned::new(ScanKind::Char, end))
}

/// Validate a comment at the start of `source` and return its length.
pub(crate) fn scan_comment(source: &str, pos: Position) -> Result<Scanned> {
    if !source.starts_with('%') {
        return Err(Error::invalid_comment_token(pos));
    }
    let end = source.find('\n').unwrap_or(source.len());
    Ok(Scanned::new(ScanKind::Comment, end))
}

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
                return Err(Error::invalid_integer_token(pos));
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
        return Err(Error::invalid_integer_token(pos));
    }
    Ok(Scanned::new(ScanKind::Integer, end))
}

/// Match a bare atom text against the reserved-word table.
pub(crate) fn keyword_from_text(text: &str) -> Option<Keyword> {
    Some(match text {
        "after" => Keyword::After,
        "and" => Keyword::And,
        "andalso" => Keyword::Andalso,
        "band" => Keyword::Band,
        "begin" => Keyword::Begin,
        "bnot" => Keyword::Bnot,
        "bor" => Keyword::Bor,
        "bsl" => Keyword::Bsl,
        "bsr" => Keyword::Bsr,
        "bxor" => Keyword::Bxor,
        "case" => Keyword::Case,
        "catch" => Keyword::Catch,
        "cond" => Keyword::Cond,
        "div" => Keyword::Div,
        "end" => Keyword::End,
        "fun" => Keyword::Fun,
        "if" => Keyword::If,
        "let" => Keyword::Let,
        "not" => Keyword::Not,
        "of" => Keyword::Of,
        "or" => Keyword::Or,
        "orelse" => Keyword::Orelse,
        "receive" => Keyword::Receive,
        "rem" => Keyword::Rem,
        "try" => Keyword::Try,
        "when" => Keyword::When,
        "xor" => Keyword::Xor,
        "maybe" => Keyword::Maybe,
        "else" => Keyword::Else,
        _ => return None,
    })
}

/// Validate a string literal at the start of `source` and return its
/// length.
///
/// Accepts both ordinary `"..."` and triple-quoted `"""..."""` forms.
pub(crate) fn scan_string(source: &str, pos: Position) -> Result<Scanned> {
    if source.is_empty() {
        return Err(Error::invalid_string_token(pos));
    }
    let (end, is_triple) = if source.starts_with(r#"""""#) {
        (scan_triple_quoted(source, pos)?, true)
    } else {
        if !source.starts_with('"') {
            return Err(Error::invalid_string_token(pos));
        }
        let inner_end = util::find_quotation_end(pos, &source[1..], '"')?;
        (1 + inner_end + 1, false)
    };
    // Adjacent string literals without intervening whitespace are rejected
    // only for the ordinary (non-triple) form to match the historical
    // behaviour.
    if !is_triple && source.get(end..end + 1) == Some("\"") {
        let pos = pos.step_by_text(&source[0..end]);
        return Err(Error::adjacent_string_literals(pos));
    }
    Ok(Scanned::new(ScanKind::String, end))
}

/// Scan a triple-quoted string literal and return its byte length.
///
/// Mirrors [`crate::tokens::StringToken::parse_triple_quoted`] but without
/// building the decoded content.
fn scan_triple_quoted(source: &str, pos: Position) -> Result<usize> {
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
            return Err(Error::invalid_string_token(pos));
        }
    }
    if !start_line_end_found {
        return Err(Error::no_closing_quotation(pos));
    }

    let mut indent = 0usize;
    let mut maybe_end_line = true;
    let mut remaining_quote_count = quote_count;
    let mut end_line_start = start_line_end;
    let mut end_line_end = start_line_end;
    for c in source[start_line_end..].chars() {
        end_line_end += c.len_utf8();
        if c == '\n' {
            indent = 0;
            maybe_end_line = true;
            remaining_quote_count = quote_count;
            end_line_start = end_line_end;
        } else if c.is_ascii_whitespace() {
            indent += 1;
        } else if maybe_end_line && c == '"' {
            remaining_quote_count -= 1;
            if remaining_quote_count == 0 {
                break;
            }
        } else {
            maybe_end_line = false;
        }
    }
    if remaining_quote_count != 0 {
        return Err(Error::no_closing_quotation(pos));
    }

    // An indented closer with no body lines has `end_line_start ==
    // start_line_end`; `saturating_sub` keeps the range well-formed
    // (decode_triple_quoted uses the same formula).
    if indent > 0 {
        let body_end = end_line_start.saturating_sub(1).max(start_line_end);
        for line in source[start_line_end..body_end].lines() {
            if line == "\n" {
                continue;
            }
            let mut valid_line = false;
            for (i, c) in line.chars().enumerate() {
                if i < indent {
                    if c.is_ascii_whitespace() {
                        continue;
                    } else {
                        return Err(Error::invalid_string_token(pos));
                    }
                }
                valid_line = true;
                break;
            }
            if !valid_line {
                return Err(Error::invalid_string_token(pos));
            }
        }
    }

    Ok(end_line_end)
}

/// Validate a sigil string literal at the start of `source` and return its
/// length.
pub(crate) fn scan_sigil_string(source: &str, pos: Position) -> Result<Scanned> {
    if !source.starts_with('~') {
        return Err(Error::invalid_sigil_string_token(pos));
    }
    let mut offset = 1;
    for c in source[offset..].chars() {
        if !util::is_atom_non_head_char(c) {
            break;
        }
        offset += c.len_utf8();
    }
    let open = source[offset..]
        .chars()
        .next()
        .ok_or_else(|| Error::invalid_sigil_string_token(pos))?;
    let content_end = if open == '"' {
        let inner = scan_string(&source[offset..], pos.step_by_width(offset))?;
        offset + inner.len
    } else {
        let close = match open {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '<' => '>',
            '/' | '|' | '\'' | '`' | '#' => open,
            _ => return Err(Error::invalid_sigil_string_token(pos)),
        };
        let inner_end =
            util::find_quotation_end(pos.step_by_width(offset + 1), &source[offset + 1..], close)?;
        offset + 1 + inner_end + 1
    };
    let mut end = content_end;
    for c in source[end..].chars() {
        if !util::is_atom_non_head_char(c) {
            break;
        }
        end += c.len_utf8();
    }
    Ok(Scanned::new(ScanKind::SigilString, end))
}

/// Validate a symbol at the start of `source` and return its length.
pub(crate) fn scan_symbol(source: &str, pos: Position) -> Result<Scanned> {
    let bytes = source.as_bytes();
    let mut symbol = if bytes.len() >= 3 {
        match &bytes[0..3] {
            b"=:=" => Some(Symbol::ExactEq),
            b"=/=" => Some(Symbol::ExactNotEq),
            b"..." => Some(Symbol::TripleDot),
            b"<:-" => Some(Symbol::StrictLeftArrow),
            b"<:=" => Some(Symbol::StrictDoubleLeftArrow),
            _ => None,
        }
    } else {
        None
    };
    let mut len = if symbol.is_some() { 3 } else { 0 };
    if symbol.is_none() && bytes.len() >= 2 {
        symbol = match &bytes[0..2] {
            b"::" => Some(Symbol::DoubleColon),
            b":=" => Some(Symbol::MapMatch),
            b"||" => Some(Symbol::DoubleVerticalBar),
            b"--" => Some(Symbol::MinusMinus),
            b"++" => Some(Symbol::PlusPlus),
            b"->" => Some(Symbol::RightArrow),
            b"<-" => Some(Symbol::LeftArrow),
            b"=>" => Some(Symbol::DoubleRightArrow),
            b"<=" => Some(Symbol::DoubleLeftArrow),
            b">>" => Some(Symbol::DoubleRightAngle),
            b"<<" => Some(Symbol::DoubleLeftAngle),
            b"==" => Some(Symbol::Eq),
            b"/=" => Some(Symbol::NotEq),
            b">=" => Some(Symbol::GreaterEq),
            b"=<" => Some(Symbol::LessEq),
            b"??" => Some(Symbol::DoubleQuestion),
            b"?=" => Some(Symbol::MaybeMatch),
            b".." => Some(Symbol::DoubleDot),
            b"&&" => Some(Symbol::DoubleAmpersand),
            _ => None,
        };
        if symbol.is_some() {
            len = 2;
        }
    }
    if symbol.is_none() && !bytes.is_empty() {
        symbol = match bytes[0] {
            b'[' => Some(Symbol::OpenSquare),
            b']' => Some(Symbol::CloseSquare),
            b'(' => Some(Symbol::OpenParen),
            b')' => Some(Symbol::CloseParen),
            b'{' => Some(Symbol::OpenBrace),
            b'}' => Some(Symbol::CloseBrace),
            b'#' => Some(Symbol::Sharp),
            b'/' => Some(Symbol::Slash),
            b'.' => Some(Symbol::Dot),
            b',' => Some(Symbol::Comma),
            b':' => Some(Symbol::Colon),
            b';' => Some(Symbol::Semicolon),
            b'=' => Some(Symbol::Match),
            b'|' => Some(Symbol::VerticalBar),
            b'?' => Some(Symbol::Question),
            b'!' => Some(Symbol::Bang),
            b'-' => Some(Symbol::Hyphen),
            b'+' => Some(Symbol::Plus),
            b'*' => Some(Symbol::Multiply),
            b'>' => Some(Symbol::Greater),
            b'<' => Some(Symbol::Less),
            _ => None,
        };
        if symbol.is_some() {
            len = 1;
        }
    }
    match symbol {
        Some(s) => Ok(Scanned::new(ScanKind::Symbol(s), len)),
        None => Err(Error::invalid_symbol_token(pos)),
    }
}

/// Validate a variable at the start of `source` and return its length.
pub(crate) fn scan_variable(source: &str, pos: Position) -> Result<Scanned> {
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::invalid_variable_token(pos))?;
    if !util::is_variable_head_char(head) {
        return Err(Error::invalid_variable_token(pos));
    }
    let mut end = head.len_utf8();
    for c in source[end..].chars() {
        if !util::is_variable_non_head_char(c) {
            break;
        }
        end += c.len_utf8();
    }
    Ok(Scanned::new(ScanKind::Variable, end))
}

/// Return `true` for the ASCII/NBSP whitespace characters recognised by
/// the tokenizer. LF (`\n`) is intentionally included.
fn is_whitespace_char(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n' | '\u{A0}')
}

/// Same as [`is_whitespace_char`] but excludes LF; used to walk the
/// non-newline half of an aggregated whitespace token.
fn is_non_lf_whitespace_char(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\u{A0}')
}

/// Aggregate a whitespace run at the start of `source`, following the
/// `erl_scan` `return_white_spaces` rules:
///
/// - a leading LF starts a token, followed by non-LF whitespace up to
///   (but not including) the next LF or non-whitespace character;
/// - a leading non-LF whitespace starts a token, followed by more non-LF
///   whitespace up to (but not including) the next LF or non-whitespace
///   character;
/// - each token contains at most one LF, always at the very start.
pub(crate) fn scan_whitespace(source: &str, pos: Position) -> Result<Scanned> {
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::invalid_whitespace_token(pos))?;
    if !is_whitespace_char(head) {
        return Err(Error::invalid_whitespace_token(pos));
    }
    let mut end = head.len_utf8();
    for c in source[end..].chars() {
        if !is_non_lf_whitespace_char(c) {
            break;
        }
        end += c.len_utf8();
    }
    Ok(Scanned::new(ScanKind::Whitespace, end))
}

/// Validate a float literal at the start of `source` and return its length.
pub(crate) fn scan_float(source: &str, pos: Position) -> Result<Scanned> {
    if is_based(source) {
        return scan_float_radix(source, pos);
    }
    let mut idx = read_digit_run(source, 0, pos)?;
    let after_int = &source[idx..];
    let mut chars = after_int.chars();
    if chars.next() != Some('.') {
        return Err(Error::invalid_float_token(pos));
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
    Ok(Scanned::new(ScanKind::Float, idx))
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
                    return Err(Error::invalid_float_token(pos));
                }
                needs_digit = true;
                idx = at + 1;
            }
            _ => break,
        }
    }
    if needs_digit {
        Err(Error::invalid_float_token(pos))
    } else {
        Ok(idx)
    }
}

/// Radix-based float form: `<radix>#<digits>.<digits>[#e<exp>]`.
fn scan_float_radix(source: &str, pos: Position) -> Result<Scanned> {
    let hash = source.find('#').expect("looks_like_float / is_based guard");
    let radix = parse_radix_digits(&source[..hash], pos)?;
    if !(1 < radix && radix < 37) {
        return Err(Error::invalid_float_token(pos));
    }
    let mut idx = hash + 1;
    if idx >= source.len() {
        return Err(Error::invalid_float_token(pos));
    }
    let (int_end, saw_dot) = read_radix_digit_run(source, idx, radix, pos, true)?;
    idx = int_end;
    if !saw_dot {
        return Err(Error::invalid_float_token(pos));
    }
    let (frac_end, has_exp) = read_radix_digit_run(source, idx, radix, pos, false)?;
    idx = frac_end;
    if has_exp {
        if !source[idx..].starts_with('e') {
            return Err(Error::invalid_float_token(pos));
        }
        idx += 1;
        idx = read_exp_digit_run(source, idx, pos)?;
    }
    Ok(Scanned::new(ScanKind::Float, idx))
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
            return Err(Error::invalid_float_token(pos));
        }
    }
    if !has_digit || !prev_digit {
        return Err(Error::invalid_float_token(pos));
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
        return Err(Error::invalid_float_token(pos));
    }
    if terminator && !expect_dot {
        // The fractional part terminated on `#`, so the caller must find
        // an `e` next.
        Ok((idx, true))
    } else if terminator {
        // The integer part terminated on `.`.
        Ok((idx, true))
    } else {
        Ok((idx, false))
    }
}

/// Consume the exponent digits (with a leading optional `-` and underscore
/// separators).
fn read_exp_digit_run(source: &str, start: usize, pos: Position) -> Result<usize> {
    let mut idx = start;
    let mut is_prev_digit = false;
    let mut saw_any = false;
    for (i, c) in source[start..].char_indices() {
        let at = start + i;
        if i == 0 && c == '-' {
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
        return Err(Error::invalid_float_token(pos));
    }
    Ok(idx)
}

// =============================================================================
// Value decoders
//
// These helpers take a text slice that the scanners above have already
// validated and produce the decoded value that `Token::value(source)` (and,
// during the migration, `tokens::*Token::from_text`) hands to callers.
//
// Decoding is intentionally kept in this module so scanning and value
// extraction share the same tables of escape characters, sigil delimiter
// pairs, digit-separator handling, and triple-quoted indentation rules.
// =============================================================================

/// Decode an atom token's value from its validated text.
///
/// Bare atoms borrow the text directly. Quoted atoms drop the outer
/// quotes; the content is borrowed when no escape sequences appear and
/// owned when escape decoding is required.
pub(crate) fn decode_atom(text: &str) -> Cow<'_, str> {
    if let Some(after_open) = text.strip_prefix('\'') {
        // `after_open` still contains the closing quote; parse_quotation
        // walks up to it while decoding escapes.
        let (v, _) = util::parse_quotation(Position::new(), after_open, '\'')
            .expect("scanner validated atom quotation");
        v
    } else {
        Cow::Borrowed(text)
    }
}

/// Decode a character token's value (`$X` or `$\...`) from validated text.
pub(crate) fn decode_char(text: &str) -> char {
    let after_dollar = &text[1..];
    let mut chars = after_dollar.char_indices().peekable();
    let (_, first) = chars.next().expect("scanner validated char payload");
    if first == '\\' {
        util::parse_escaped_char(Position::new(), &mut chars).expect("scanner validated escape")
    } else {
        first
    }
}

/// Decode a comment token's value (the text after the leading `%`).
pub(crate) fn decode_comment(text: &str) -> &str {
    &text[1..]
}

/// Decode an integer token's value from its validated text.
///
/// Returns `Some(value)` when the value fits in `i64`, and `None` when it
/// overflows (checked, never wrapped).
pub(crate) fn decode_integer(text: &str) -> Option<i64> {
    let (radix, digits_slice) = if let Some(hash) = text.find('#') {
        let radix: u32 = util::strip_underscores(&text[..hash])
            .parse()
            .expect("scanner validated radix");
        (radix, &text[hash + 1..])
    } else {
        (10u32, text)
    };
    let cleaned = util::strip_underscores(digits_slice);
    if radix == 10 {
        cleaned.parse::<i64>().ok()
    } else {
        i64::from_str_radix(&cleaned, radix).ok()
    }
}

/// Decode a float token's value from its validated text (either decimal
/// or radix-prefixed).
pub(crate) fn decode_float(text: &str) -> f64 {
    if let Some(hash) = text.find('#') {
        decode_radix_float(text, hash)
    } else {
        util::strip_underscores(text)
            .parse::<f64>()
            .expect("scanner validated decimal float")
    }
}

fn decode_radix_float(slice: &str, hash: usize) -> f64 {
    let radix: u32 = util::strip_underscores(&slice[..hash])
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

    let mut value = 0.0_f64;
    for c in int_part.chars() {
        if c == '_' {
            continue;
        }
        let d = c.to_digit(radix).expect("scanner validated integer digit");
        value = value * radix as f64 + d as f64;
    }
    let mut j = 1_i32;
    for c in frac_part.chars() {
        if c == '_' {
            continue;
        }
        let d = c
            .to_digit(radix)
            .expect("scanner validated fractional digit");
        value += d as f64 / (radix as f64).powi(j);
        j += 1;
    }
    if let Some(exp_str) = exp_opt {
        let exp: i32 = util::strip_underscores(exp_str)
            .parse()
            .expect("scanner validated exponent");
        value *= (radix as f64).powi(exp);
    }
    value
}

/// Decode a string token's value from its validated text.
///
/// Handles both the regular `"..."` form (borrowed when the content has
/// no escape sequences) and the triple-quoted `"""..."""` form (borrowed
/// when no indentation stripping is required).
pub(crate) fn decode_string(text: &str) -> Cow<'_, str> {
    if text.starts_with(r#"""""#) {
        decode_triple_quoted(text)
    } else {
        let after_open = &text[1..];
        let (v, _) = util::parse_quotation(Position::new(), after_open, '"')
            .expect("scanner validated string quotation");
        v
    }
}

/// Decode a triple-quoted string's body from validated text.
///
/// Borrowed when the closing line has no indentation (the body is a
/// contiguous slice of the source); owned when indentation must be
/// stripped from each content line.
fn decode_triple_quoted(text: &str) -> Cow<'_, str> {
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

    // Walk the body to find the closing indentation line.
    let mut indent = 0usize;
    let mut maybe_end_line = true;
    let mut remaining = quote_count;
    let mut end_line_start = start_line_end;
    let mut end_line_end = start_line_end;
    for c in text[start_line_end..].chars() {
        end_line_end += c.len_utf8();
        if c == '\n' {
            indent = 0;
            maybe_end_line = true;
            remaining = quote_count;
            end_line_start = end_line_end;
        } else if c.is_ascii_whitespace() {
            indent += 1;
        } else if maybe_end_line && c == '"' {
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        } else {
            maybe_end_line = false;
        }
    }

    let body_end = end_line_start.saturating_sub(1).max(start_line_end);
    let body = &text[start_line_end..body_end];
    if indent == 0 {
        return Cow::Borrowed(body);
    }
    let mut value = String::with_capacity(body.len());
    let mut first = true;
    for line in body.split('\n') {
        if !first {
            value.push('\n');
        }
        first = false;
        for (i, c) in line.chars().enumerate() {
            if i < indent {
                continue;
            }
            value.push(c);
        }
    }
    Cow::Owned(value)
}

/// Split a sigil string's validated text into its `~<prefix><open>content
/// <close><suffix>` pieces.
///
/// `prefix` and `suffix` always borrow from `text`; `content` borrows when
/// no escape sequences or triple-quoted indentation appear inside it.
pub(crate) fn decode_sigil(text: &str) -> (&str, Cow<'_, str>, &str) {
    let mut prefix_end = 1; // skip leading `~`
    for c in text[prefix_end..].chars() {
        if !util::is_atom_non_head_char(c) {
            break;
        }
        prefix_end += c.len_utf8();
    }
    let prefix = &text[1..prefix_end];
    let open = text[prefix_end..]
        .chars()
        .next()
        .expect("scanner validated sigil delimiter");
    let (content, content_end) = if open == '"' {
        // The content is itself a full (regular or triple-quoted) string.
        let sub = &text[prefix_end..];
        let scanned = scan_string(sub, Position::new()).expect("scanner validated sigil string");
        (decode_string(&sub[..scanned.len]), prefix_end + scanned.len)
    } else {
        let close = match open {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '<' => '>',
            other => other,
        };
        let content_start = prefix_end + 1;
        let content_len = util::find_quotation_end(Position::new(), &text[content_start..], close)
            .expect("scanner validated sigil close");
        let inner = &text[content_start..content_start + content_len];
        let value = if inner.contains('\\') {
            let (v, _) = util::parse_quotation(Position::new(), &text[content_start..], close)
                .expect("scanner validated sigil close");
            v
        } else {
            Cow::Borrowed(inner)
        };
        (value, content_start + content_len + 1)
    };
    let suffix = &text[content_end..];
    (prefix, content, suffix)
}
