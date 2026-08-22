//! Whitespace aggregation invariants.
//!
//! For any generated whitespace-only source, the scanner's token
//! boundaries must equal the boundaries produced by an independent
//! model of `erl_scan`'s five origin-specific rules (space run capped
//! at 16, tab run capped at 10, CR alone, LF variants with `\n\r` /
//! `\n\f` fixed pairs and space/tab caps of 17/11, and the unlimited
//! non-LF `scan_white_space` / `scan_nl_white_space` runs). The
//! concatenation of all token texts must equal the source, and every
//! token holds at most one LF at its very start.

#[expect(dead_code, reason = "shared helpers; this binary uses only a subset")]
mod pbt_harness;
use pbt_harness::{CASES, Counter, SEED_ENV, sample_whitespace_sequence, step_position};

#[test]
fn whitespace_aggregation_invariants() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);

    let empty_cases = Counter::default();
    let nonempty_cases = Counter::default();
    let saw_leading_lf = Counter::default();
    let saw_consecutive_lf = Counter::default();
    let saw_cr_lf = Counter::default();
    let saw_nbsp = Counter::default();

    runner.run(CASES, |ctx| {
        let src = sample_whitespace_sequence(ctx);
        if src.is_empty() {
            empty_cases.hit();
        } else {
            nonempty_cases.hit();
        }
        if src.contains('\u{a0}') {
            saw_nbsp.hit();
        }
        if src.as_bytes().windows(2).any(|w| w == b"\r\n") {
            saw_cr_lf.hit();
        }

        let mut tokens: Vec<(usize, usize)> = Vec::new();
        let mut pos = erl_tokenize::Position::new();
        let mut concat = String::with_capacity(src.len());
        while let Some(token) = erl_tokenize::scan_token(&src, pos)? {
            assert_eq!(
                token.kind(),
                erl_tokenize::TokenKind::Whitespace,
                "non-whitespace token in {src:?}"
            );
            let text = token.text(&src);
            let lfs = text.matches('\n').count();
            assert!(lfs <= 1, "token has {lfs} LFs: {text:?} in {src:?}");
            if lfs == 1 {
                assert!(
                    text.starts_with('\n'),
                    "LF not at start: {text:?} in {src:?}"
                );
                saw_leading_lf.hit();
            }
            concat.push_str(text);
            tokens.push((token.start().offset(), token.end().offset()));
            pos = token.end();
        }
        assert_eq!(concat, src, "concat mismatch for {src:?}");
        assert_eq!(pos.offset(), src.len(), "did not reach EOF for {src:?}");

        // The independent model reproduces erl_scan's five
        // origin-specific rules. A drift between the scanner and the
        // model localizes the divergence to a specific boundary rather
        // than a coarse "concat mismatched" failure.
        let expected_ends: Vec<usize> = model_whitespace_ends(&src);
        let actual_ends: Vec<usize> = tokens.iter().map(|(_, end)| *end).collect();
        assert_eq!(
            actual_ends, expected_ends,
            "token boundaries diverged from erl_scan model for {src:?}"
        );

        let expected = step_position((0, 1, 1), &src);
        assert_eq!(pos.offset(), expected.0, "model offset for {src:?}");
        assert_eq!(pos.line().get(), expected.1, "model line for {src:?}");
        assert_eq!(pos.column().get(), expected.2, "model column for {src:?}");

        for pair in tokens.windows(2) {
            let (_, first_end) = pair[0];
            let (second_start, _) = pair[1];
            assert_eq!(first_end, second_start, "gap between tokens in {src:?}");
        }
        if tokens.windows(2).any(|pair| {
            let (_, end) = pair[0];
            src[end..].starts_with('\n')
        }) && src.matches('\n').count() >= 2
        {
            saw_consecutive_lf.hit();
        }

        Ok(())
    })?;

    assert_eq!(runner.stats().rejected_cases, 0, "no rejects\n{runner}");
    assert!(empty_cases.get() > 0, "no empty source\n{runner}");
    assert!(nonempty_cases.get() > 0, "no nonempty source\n{runner}");
    assert!(saw_leading_lf.get() > 0, "no leading LF token\n{runner}");
    assert!(
        saw_consecutive_lf.get() > 0,
        "no consecutive LF case\n{runner}"
    );
    assert!(saw_cr_lf.get() > 0, "no CRLF case\n{runner}");
    assert!(saw_nbsp.get() > 0, "no NBSP case\n{runner}");
    Ok(())
}

/// Independent model of `erl_scan`'s origin-specific whitespace
/// aggregation. Panics if `src` contains a non-`?WHITE_SPACE`
/// character (the caller is expected to hand in a whitespace-only
/// source).
fn model_whitespace_ends(src: &str) -> Vec<usize> {
    let bytes = src.as_bytes();
    let mut ends = Vec::new();
    let mut i = 0;
    while i < src.len() {
        let head = src[i..].chars().next().expect("i < src.len()");
        assert!(is_ws(head), "non-whitespace {head:?} at offset {i}");
        let end = match head {
            ' ' => cap_run(bytes, i + 1, i + 16, b' '),
            '\t' => cap_run(bytes, i + 1, i + 10, b'\t'),
            '\r' => i + 1,
            '\n' => match src[i + 1..].chars().next() {
                Some(' ') => cap_run(bytes, i + 2, i + 17, b' '),
                Some('\t') => cap_run(bytes, i + 2, i + 11, b'\t'),
                Some('\r') | Some('\u{0C}') => i + 2,
                Some(c) if c != '\n' && is_ws(c) => walk_non_lf_ws(src, i + 1),
                _ => i + 1,
            },
            c => walk_non_lf_ws(src, i + c.len_utf8()),
        };
        ends.push(end);
        i = end;
    }
    ends
}

fn cap_run(bytes: &[u8], start: usize, cap: usize, target: u8) -> usize {
    let mut e = start;
    while e < cap && bytes.get(e).copied() == Some(target) {
        e += 1;
    }
    e
}

fn walk_non_lf_ws(src: &str, start: usize) -> usize {
    let mut e = start;
    for c in src[start..].chars() {
        if c == '\n' || !is_ws(c) {
            break;
        }
        e += c.len_utf8();
    }
    e
}

fn is_ws(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{20}' | '\u{80}'..='\u{A0}')
}
