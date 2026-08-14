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
use crate::{Error, ErrorKind, Position, Result};

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
        .ok_or_else(|| Error::new(ErrorKind::MissingToken, pos))?;
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
            // `erl_scan` routes Latin-1 uppercase letters (`À..Þ` minus
            // `×`) to variables and Latin-1 lowercase letters to atoms;
            // everything else non-alphabetic falls through to `scan_symbol`.
            if util::is_variable_head_char(head) {
                scan_variable(source, pos)
            } else if head.is_alphabetic() {
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
    // Decimal float: `<digits>.<namechar>` (digits are namechars too).
    if bytes.get(i) == Some(&b'.') {
        return bytes.get(i + 1).copied().is_some_and(is_ascii_namechar);
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
            && bytes.get(j + 1).copied().is_some_and(is_ascii_namechar);
    }
    false
}

/// Byte-level fast path for [`util::is_namechar`]: every namechar is
/// ASCII, so a byte comparison suffices here.
fn is_ascii_namechar(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'@')
}

/// Validate an atom token at the start of `source` and return its length.
pub(crate) fn scan_atom(source: &str, pos: Position) -> Result<Scanned> {
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidAtomToken, pos))?;
    if head == '\'' {
        let inner_end = util::find_quotation_end(pos, &source[1..], '\'')?;
        Ok(Scanned::new(ScanKind::Atom, 1 + inner_end + 1))
    } else {
        if !util::is_atom_head_char(head) {
            return Err(Error::new(ErrorKind::InvalidAtomToken, pos));
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
    let (_, dollar) = chars
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidCharToken, pos))?;
    if dollar != '$' {
        return Err(Error::new(ErrorKind::InvalidCharToken, pos));
    }
    let (i, c) = chars
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidCharToken, pos))?;
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
        return Err(Error::new(ErrorKind::InvalidCommentToken, pos));
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
        && util::is_namechar(c)
    {
        return Err(Error::new(ErrorKind::InvalidIntegerToken, pos));
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
    let (end, is_triple) = scan_string_body(source, pos, None)?;
    // Adjacent string literals without intervening whitespace are rejected
    // only for the ordinary (non-triple) form to match `erl_scan`.
    if !is_triple && source.get(end..end + 1) == Some("\"") {
        let pos = pos.step_by_text(&source[0..end]);
        return Err(Error::new(ErrorKind::AdjacentStringLiterals, pos));
    }
    Ok(Scanned::new(ScanKind::String, end))
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
///   verbatim. See [`is_verbatim_sigil_prefix`].
/// * Triple-quoted `"""..."""`: only `s` and `b` are non-verbatim; the
///   empty prefix, every other prefix, and a plain string are all verbatim
///   (`erl_scan`'s `scan_tqstring` classifies `SigilType` `b`/`s` as
///   non-verbatim and everything else as verbatim).
fn scan_string_body(source: &str, pos: Position, prefix: Option<&str>) -> Result<(usize, bool)> {
    if source.is_empty() {
        return Err(Error::new(ErrorKind::InvalidStringToken, pos));
    }
    if source.starts_with(r#"""""#) {
        let verbatim = !matches!(prefix, Some("b") | Some("s"));
        Ok((scan_triple_quoted(source, pos, verbatim)?, true))
    } else {
        if !source.starts_with('"') {
            return Err(Error::new(ErrorKind::InvalidStringToken, pos));
        }
        let verbatim = prefix.is_some_and(is_verbatim_sigil_prefix);
        let inner_end = if verbatim {
            util::find_verbatim_quotation_end(pos, &source[1..], '"')?
        } else {
            util::find_quotation_end(pos, &source[1..], '"')?
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
                util::validate_escapes(pos, util::strip_line_continuation(stripped))?;
            }
        }
    }

    Ok(end_line_end)
}

/// Validate a sigil string literal at the start of `source` and return its
/// length.
pub(crate) fn scan_sigil_string(source: &str, pos: Position) -> Result<Scanned> {
    if !source.starts_with('~') {
        return Err(Error::new(ErrorKind::InvalidSigilStringToken, pos));
    }
    let mut offset = 1;
    for c in source[offset..].chars() {
        if !util::is_atom_non_head_char(c) {
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
        let (len, _is_triple) =
            scan_string_body(&source[offset..], pos.step_by_width(offset), Some(prefix))?;
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
            util::find_verbatim_quotation_end(inner_pos, content, close)?
        } else {
            util::find_quotation_end(inner_pos, content, close)?
        };
        offset + 1 + inner_end + 1
    };
    let mut end = content_end;
    for c in source[end..].chars() {
        if !util::is_atom_non_head_char(c) {
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
    Ok(Scanned::new(ScanKind::SigilString, end))
}

/// Return `true` when the sigil prefix indicates a verbatim string, i.e.
/// escape sequences inside the content are preserved as-is.
///
/// `erl_scan` classifies the empty prefix (`~"..."`), `b` (`~b"..."`),
/// and `s` (`~s"..."`) as non-verbatim; every other prefix — `~B`, `~S`,
/// `~foo`, `~X`, and so on — is verbatim.
fn is_verbatim_sigil_prefix(prefix: &str) -> bool {
    !matches!(prefix, "" | "b" | "s")
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
            b"?=" => Some(Symbol::MaybeMatch),
            b"#_" => Some(Symbol::WildcardRecord),
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
        None => Err(Error::new(ErrorKind::InvalidSymbolToken, pos)),
    }
}

/// Validate a variable at the start of `source` and return its length.
pub(crate) fn scan_variable(source: &str, pos: Position) -> Result<Scanned> {
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidVariableToken, pos))?;
    if !util::is_variable_head_char(head) {
        return Err(Error::new(ErrorKind::InvalidVariableToken, pos));
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
        .ok_or_else(|| Error::new(ErrorKind::InvalidWhitespaceToken, pos))?;
    if !is_whitespace_char(head) {
        return Err(Error::new(ErrorKind::InvalidWhitespaceToken, pos));
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
        && util::is_namechar(c)
    {
        return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
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
            && util::is_namechar(c)
        {
            return Err(Error::new(ErrorKind::InvalidFloatToken, pos));
        }
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
/// Erlang integer literals are always non-negative (unary `-` is a
/// separate token), so the value fits in `u64` whenever it does not
/// overflow. Returns `Some(value)` in range, `None` when it exceeds
/// `u64::MAX` (checked, never wrapped).
pub(crate) fn decode_integer(text: &str) -> Option<u64> {
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
        cleaned.parse::<u64>().ok()
    } else {
        u64::from_str_radix(&cleaned, radix).ok()
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
        let Ok(exp) = util::strip_underscores(exp_str).parse::<i32>() else {
            // The exponent overflows `i32`, so the magnitude is beyond the
            // finite f64 range. Returning infinity lets `scan_float`'s
            // `is_finite()` guard report `InvalidFloatToken` instead of
            // panicking on an otherwise-valid input (public API contract:
            // panic only on caller contract violation).
            return f64::INFINITY;
        };
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
        decode_triple_quoted(text, true)
    } else {
        let after_open = &text[1..];
        let (v, _) = util::parse_quotation(Position::new(), after_open, '"')
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
fn decode_triple_quoted(text: &str, verbatim: bool) -> Cow<'_, str> {
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
            let stripped = util::strip_line_continuation(stripped);
            let decoded = util::decode_quotation_content(Position::new(), stripped);
            value.push_str(&decoded);
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
    let verbatim = is_verbatim_sigil_prefix(prefix);
    let open = text[prefix_end..]
        .chars()
        .next()
        .expect("scanner validated sigil delimiter");
    let (content, content_end) = if open == '"' {
        // The content is itself a full (regular or triple-quoted) string.
        let sub = &text[prefix_end..];
        let (len, is_triple) = scan_string_body(sub, Position::new(), Some(prefix))
            .expect("scanner validated sigil string");
        let body = &sub[..len];
        let value = if is_triple {
            // Triple-quoted sigils follow a different verbatim rule than
            // single-quoted ones: only `b`/`s` are non-verbatim, and the
            // empty prefix is verbatim (matching `erl_scan`'s
            // `scan_tqstring`).
            decode_triple_quoted(body, !matches!(prefix, "b" | "s"))
        } else {
            decode_regular_string(&body[1..len - 1], verbatim)
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
            util::find_verbatim_quotation_end(Position::new(), &text[content_start..], close)
                .expect("scanner validated sigil close")
        } else {
            util::find_quotation_end(Position::new(), &text[content_start..], close)
                .expect("scanner validated sigil close")
        };
        let inner = &text[content_start..content_start + content_len];
        let value = decode_regular_string(inner, verbatim);
        (value, content_start + content_len + 1)
    };
    let suffix = &text[content_end..];
    (prefix, content, suffix)
}

/// Decode the content between the delimiters of an ordinary (non-triple)
/// quoted region. Non-verbatim content borrows when it has no `\` escapes
/// and is otherwise decoded into an owned `String`; verbatim content
/// always borrows the raw slice.
fn decode_regular_string(inner: &str, verbatim: bool) -> Cow<'_, str> {
    if verbatim || !inner.contains('\\') {
        Cow::Borrowed(inner)
    } else {
        Cow::Owned(util::decode_quotation_content(Position::new(), inner))
    }
}
