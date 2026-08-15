//! Internal lexical scanning.
//!
//! This module is the dispatcher: it routes the head character of a
//! source slice to the appropriate per-kind scanner (`lex_atom`,
//! `lex_string`, ...) and re-exports the value decoders those scanners
//! provide. The per-kind modules are the single source of truth for
//! validating each token's shape.
//!
//! The scanners never allocate on the happy path: they return only a
//! [`Scanned`] value describing the token kind and its UTF-8 byte length.
//! Value construction (owned strings, decoded escapes, integer/float
//! parsing) is the caller's responsibility and happens against the slice
//! whose length is returned here.
//!
//! Public tokenization API in [`crate::token`] wires its entry points
//! through this module so that no other place in the crate re-implements
//! the same lexical rules.

use crate::charset;
use crate::keyword::Keyword;
use crate::lex_atom::scan_atom;
use crate::lex_char::scan_char;
use crate::lex_comment::scan_comment;
use crate::lex_float::{looks_like_float, scan_float};
use crate::lex_integer::scan_integer;
use crate::lex_sigil::scan_sigil_string;
use crate::lex_string::scan_string;
use crate::lex_symbol::scan_symbol;
use crate::lex_variable::scan_variable;
use crate::lex_whitespace::{is_whitespace_head, scan_whitespace};
use crate::token::TokenKind;
use crate::{Error, ErrorKind, Position, Result};

pub(crate) use crate::lex_atom::decode_atom;
pub(crate) use crate::lex_char::decode_char;
pub(crate) use crate::lex_comment::decode_comment;
pub(crate) use crate::lex_float::decode_float;
pub(crate) use crate::lex_integer::decode_integer;
pub(crate) use crate::lex_sigil::decode_sigil;
pub(crate) use crate::lex_string::decode_string;

/// Result of scanning one token at the start of a source slice.
///
/// Only holds `Copy` data: no owned strings, no vectors, no boxed values.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Scanned {
    pub kind: TokenKind,
    pub len: usize,
}

impl Scanned {
    pub(crate) fn new(kind: TokenKind, len: usize) -> Self {
        Self { kind, len }
    }
}

/// Scan a single token at the start of `source`.
///
/// Any error returned from this function carries the resume position that
/// [`crate::scan_token`] passes back to the caller (`pos.step_by_char(first_char)`
/// on non-empty input, `pos` on empty).
pub(crate) fn scan_one(source: &str, pos: Position) -> Result<Scanned> {
    scan_one_impl(source, pos).map_err(|mut e| {
        e.resume_position = resume_from(source, pos);
        e
    })
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
        c if is_whitespace_head(c) => scan_whitespace(source, pos),
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
            if charset::is_variable_head_char(head) {
                scan_variable(source, pos)
            } else if head.is_alphabetic() {
                let atom = scan_atom(source, pos)?;
                let text = &source[..atom.len];
                if let Some(k) = Keyword::from_text(text) {
                    Ok(Scanned::new(TokenKind::Keyword(k), atom.len))
                } else {
                    Ok(atom)
                }
            } else {
                scan_symbol(source, pos)
            }
        }
    }
}
