use crate::TokenKind;
use crate::lex::Scanned;
use crate::{Error, ErrorKind, Position, Result};

/// Return `true` for the characters `erl_scan` classifies as
/// `?WHITE_SPACE`: bytes in `0x00..=0x20` (control chars up to and
/// including space) and `0x80..=0xA0` (Latin-1 control chars up to and
/// including NBSP). DEL (0x7F) is intentionally excluded — `erl_scan`'s
/// macro range does not cover it.
fn is_whitespace_char(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{20}' | '\u{80}'..='\u{A0}')
}

/// Aggregate a whitespace run at the start of `source`, following
/// `erl_scan`'s five origin-specific rules from `scan1` /
/// `scan_spcs` / `scan_tabs` / `scan_newline` / `scan_nl_spcs` /
/// `scan_nl_tabs` / `scan_nl_white_space` / `scan_white_space`:
///
/// - **space origin**: run of space, capped at 16.
/// - **tab origin**: run of tab, capped at 10.
/// - **CR origin**: single `\r`.
/// - **LF origin**: dispatch on the next character —
///   - `\n<space>` extends with spaces (total capped at 17);
///   - `\n<tab>` extends with tabs (total capped at 11);
///   - `\n\r` and `\n\f` are two-character fixed pairs;
///   - `\n<other ?WHITE_SPACE>` accumulates any non-LF `?WHITE_SPACE`
///     with no length limit;
///   - `\n<anything else>` stops at the LF alone.
/// - **other `?WHITE_SPACE` origin** (`\f`, `\v`, `\b`, NUL, NBSP,
///   Latin-1 controls in `0x80..=0x9F`, …): accumulates any non-LF
///   `?WHITE_SPACE` with no length limit, matching `scan_white_space`.
///
/// Every token contains at most one LF, always at the very start.
pub(crate) fn scan_whitespace(source: &str, pos: Position) -> Result<Scanned> {
    let bytes = source.as_bytes();
    let head = source
        .chars()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidWhitespaceToken, pos))?;
    let end = match head {
        ' ' => run_ascii(bytes, 1, 16, b' '),
        '\t' => run_ascii(bytes, 1, 10, b'\t'),
        '\r' => 1,
        '\n' => match source[1..].chars().next() {
            Some(' ') => run_ascii(bytes, 2, 17, b' '),
            Some('\t') => run_ascii(bytes, 2, 11, b'\t'),
            Some('\r') | Some('\u{0C}') => 2,
            Some(c) if c != '\n' && is_whitespace_char(c) => scan_non_lf_ws(source, 1),
            _ => 1,
        },
        c if is_whitespace_char(c) => scan_non_lf_ws(source, c.len_utf8()),
        _ => return Err(Error::new(ErrorKind::InvalidWhitespaceToken, pos)),
    };
    Ok(Scanned::new(TokenKind::Whitespace, end))
}

/// Walk consecutive `target` bytes starting at `start`, stopping when a
/// non-matching byte is seen or when the run would grow the token past
/// `limit` bytes. Mirrors erl_scan's capped `scan_spcs` / `scan_tabs` /
/// `scan_nl_spcs` / `scan_nl_tabs` runs (`N < 16` / `N < 10` / `N < 17`
/// / `N < 11` respectively).
fn run_ascii(bytes: &[u8], start: usize, limit: usize, target: u8) -> usize {
    let mut end = start;
    while end < limit && bytes.get(end).copied() == Some(target) {
        end += 1;
    }
    end
}

/// Walk any non-LF `?WHITE_SPACE` characters starting at `start`, with
/// no length limit. Mirrors `scan_white_space` / `scan_nl_white_space`
/// (both stop only at LF or a non-`?WHITE_SPACE` character).
fn scan_non_lf_ws(source: &str, start: usize) -> usize {
    let mut end = start;
    for c in source[start..].chars() {
        if c == '\n' || !is_whitespace_char(c) {
            break;
        }
        end += c.len_utf8();
    }
    end
}

/// Return `true` for the characters `erl_scan` classifies as
/// `?WHITE_SPACE`. Exposed to the dispatcher so it can steer the head
/// character to [`scan_whitespace`].
pub(crate) fn is_whitespace_head(c: char) -> bool {
    is_whitespace_char(c)
}
