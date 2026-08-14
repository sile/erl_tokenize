//! Whitespace aggregation invariants.
//!
//! For any generated whitespace-only source, every scanned whitespace
//! token has at most one LF at its start, the concatenation of the
//! token texts equals the source, and no two adjacent whitespace tokens
//! could be re-merged without breaking the `erl_scan return_white_spaces`
//! rule (LF starts a new token; non-LF whitespace accumulates).

use erl_tokenize::{Position, TokenKind, scan_token};

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
        let mut pos = Position::new();
        let mut concat = String::with_capacity(src.len());
        while let Some(token) = scan_token(&src, pos)? {
            assert_eq!(
                token.kind(),
                TokenKind::Whitespace,
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

        let expected = step_position((0, 1, 1), &src);
        assert_eq!(pos.offset(), expected.0, "model offset for {src:?}");
        assert_eq!(pos.line().get(), expected.1, "model line for {src:?}");
        assert_eq!(pos.column().get(), expected.2, "model column for {src:?}");

        for pair in tokens.windows(2) {
            let (_, first_end) = pair[0];
            let (second_start, _) = pair[1];
            assert_eq!(first_end, second_start, "gap between tokens in {src:?}");
            let second_text = &src[second_start..pair[1].1];
            assert!(
                second_text.starts_with('\n'),
                "adjacent whitespace tokens would merge: {second_text:?} in {src:?}"
            );
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
