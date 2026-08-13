//! Property-based tests using noprop.
//!
//! All tests are seeded from `ERL_TOKENIZE_PBT_SEED` when set; otherwise
//! a fresh time-derived seed is used so every run explores new inputs.
//! A failing case prints its seed (hex) and case index; re-run with
//! `ERL_TOKENIZE_PBT_SEED=<seed>` to reproduce it.
//!
//! Coverage here is a floor: the aim is to detect regressions in the
//! prefix invariant of `scan_token` and in the position bookkeeping of
//! error recovery. Broader property and fuzz coverage lives elsewhere.

use std::cell::Cell;

use erl_tokenize::{Position, scan_token};

const CASES: usize = 256;
const MAX_LEN: usize = 64;
const SEED_ENV: &str = "ERL_TOKENIZE_PBT_SEED";

/// Sample a text length biased to the 0 / 1 / MAX_LEN boundaries.
fn sample_len(ctx: &mut noprop::TestCaseContext) -> usize {
    noprop::sample_with_boundaries(
        ctx,
        &[0usize, 1, MAX_LEN],
        noprop::Ratio::one_nth(5),
        |ctx| noprop::sample_usize_in(ctx, 0..=MAX_LEN),
    )
}

/// Sample an arbitrary text, mixing escape-relevant specials, printable
/// ASCII, and arbitrary Unicode.
fn sample_text(ctx: &mut noprop::TestCaseContext) -> String {
    const SPECIALS: [char; 12] = [
        '"', '\'', '\\', '\n', '\t', '\r', '\0', '\u{1}', '\u{7f}', '\u{80}', '\u{a0}', '\u{2028}',
    ];
    let len = sample_len(ctx);
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        match noprop::sample_usize_in(ctx, 0..3) {
            0 => s.push(noprop::sample_choice(ctx, &SPECIALS)),
            1 => s.push(noprop::sample_ascii_printable_char(ctx)),
            _ => s.push(noprop::sample_char(ctx)),
        }
    }
    s
}

/// Advance the (offset, line, column) model by `text`, using the same
/// LF-driven line/column rules as `Position`.
fn step_position(
    mut offset: usize,
    mut line: usize,
    mut column: usize,
    text: &str,
) -> (usize, usize, usize) {
    let mut rest = text;
    while let Some(i) = rest.find('\n') {
        offset += i + 1;
        line += 1;
        column = 1;
        rest = &rest[i + 1..];
    }
    offset += rest.len();
    column += rest.len();
    (offset, line, column)
}

/// Advance by a single character, respecting LF.
fn step_char(
    mut offset: usize,
    mut line: usize,
    mut column: usize,
    c: char,
) -> (usize, usize, usize) {
    let n = c.len_utf8();
    offset += n;
    if c == '\n' {
        line += 1;
        column = 1;
    } else {
        column += n;
    }
    (offset, line, column)
}

// ============================================================
// Prefix invariant
// ============================================================

/// For any text, the first successful token's text must be a prefix of
/// that text.
#[test]
fn scan_token_prefix_invariant() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let text = sample_text(ctx);
        if let Ok(Some(token)) = scan_token(&text, Position::new()) {
            let t = token.text(&text);
            assert!(!t.is_empty(), "empty token text for {text:?}");
            assert!(text.starts_with(t), "{t:?} not a prefix of {text:?}");
        }
        Ok(())
    })?;

    Ok(())
}

// ============================================================
// Position model with error recovery
// ============================================================

/// Drive `scan_token` from `Position::new()` to EOF, using
/// `Error::resume_position` to advance past bad tokens; assert the
/// resulting positions match a separately maintained (offset, line,
/// column) model at every step.
///
/// This catches boundary-bookkeeping bugs (e.g. landing in the middle of
/// a multi-byte character) and any token text that fails to be a prefix
/// of the remaining input.
#[test]
fn scan_token_position_model_with_recovery() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let ok_cases = Cell::new(0usize);
    let error_cases = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let text = sample_text(ctx);
        let src = text.as_str();
        let mut pos = Position::new();
        let mut offset = 0usize;
        let mut line = 1usize;
        let mut column = 1usize;
        let mut saw_ok = false;
        let mut saw_err = false;

        loop {
            assert_eq!(pos.offset(), offset, "offset mismatch");
            assert_eq!(pos.line(), line, "line mismatch");
            assert_eq!(pos.column(), column, "column mismatch");
            match scan_token(src, pos) {
                Ok(None) => break,
                Ok(Some(token)) => {
                    saw_ok = true;
                    let t = token.text(src);
                    let rest = &src[offset..];
                    assert!(
                        rest.starts_with(t),
                        "token text {t:?} is not a prefix of {rest:?}"
                    );
                    (offset, line, column) = step_position(offset, line, column, t);
                    pos = token.end();
                    assert_eq!(pos.offset(), offset, "end offset mismatch");
                    assert_eq!(pos.line(), line, "end line mismatch");
                    assert_eq!(pos.column(), column, "end column mismatch");
                }
                Err(err) => {
                    saw_err = true;
                    let c = src[offset..]
                        .chars()
                        .next()
                        .expect("error at a non-EOF position");
                    (offset, line, column) = step_char(offset, line, column, c);
                    pos = err.resume_position();
                }
            }
        }
        assert_eq!(offset, src.len(), "scan stopped before EOF");
        if saw_ok {
            ok_cases.set(ok_cases.get() + 1);
        }
        if saw_err {
            error_cases.set(error_cases.get() + 1);
        }
        Ok(())
    })?;

    assert!(ok_cases.get() > 0, "no case produced a token\n{runner}");
    assert!(
        error_cases.get() > 0,
        "no case exercised the error recovery path\n{runner}"
    );
    Ok(())
}
