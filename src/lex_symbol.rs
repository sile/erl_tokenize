use crate::TokenKind;
use crate::lex::Scanned;
use crate::symbol::Symbol;
use crate::{Error, ErrorKind, Position, Result};

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
        Some(s) => Ok(Scanned::new(TokenKind::Symbol(s), len)),
        None => Err(Error::new(ErrorKind::InvalidSymbolToken, pos)),
    }
}
