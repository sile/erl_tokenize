//! erl_tokenize::Position transition model.
//!
//! Maintain an independent `(offset, line, column)` model in the test
//! and assert `erl_tokenize::Token::start()` / `erl_tokenize::Token::end()` match at every step.
//! Coverage gates require LF, multi-byte, and multi-line branches to be
//! actually exercised.

#[expect(dead_code, reason = "shared helpers; this binary uses only a subset")]
mod pbt_harness;
use pbt_harness::{
    CASES, Counter, SEED_ENV, join_tokens, sample_token_count, sample_valid_token_text,
    step_position,
};

#[test]
fn position_model_matches_scan_token() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);

    let saw_lf = Counter::default();
    let saw_multibyte = Counter::default();
    let saw_multiline = Counter::default();
    let saw_crlf = Counter::default();

    runner.run(CASES, |ctx| {
        let n = sample_token_count(ctx);
        let mut pieces: Vec<String> = Vec::with_capacity(n);
        for _ in 0..n {
            pieces.push(sample_valid_token_text(ctx));
        }
        let src = join_tokens(ctx, &pieces);
        if src.contains("\r\n") {
            saw_crlf.hit();
        }

        let mut pos = erl_tokenize::Position::new();
        let mut model = (0usize, 1usize, 1usize);
        let step_ceiling = src.chars().count() * 2 + 8;
        let mut steps = 0usize;
        while let Some(token) = erl_tokenize::scan_token(&src, pos)? {
            let text = token.text(&src);
            assert_eq!(
                token.start().offset(),
                model.0,
                "start offset for {src:?} at {text:?}"
            );
            assert_eq!(
                token.start().line().get(),
                model.1,
                "start line for {src:?} at {text:?}"
            );
            assert_eq!(
                token.start().column().get(),
                model.2,
                "start column for {src:?} at {text:?}"
            );
            let after = step_position(model, text);
            assert_eq!(
                token.end().offset(),
                after.0,
                "end offset for {src:?} at {text:?}"
            );
            assert_eq!(
                token.end().line().get(),
                after.1,
                "end line for {src:?} at {text:?}"
            );
            assert_eq!(
                token.end().column().get(),
                after.2,
                "end column for {src:?} at {text:?}"
            );
            if text.contains('\n') {
                saw_lf.hit();
            }
            if text.chars().any(|c| c.len_utf8() > 1) {
                saw_multibyte.hit();
            }
            if token.end().line().get() > 1 {
                saw_multiline.hit();
            }
            model = after;
            pos = token.end();
            steps += 1;
            assert!(steps <= step_ceiling, "exceeded step ceiling for {src:?}");
        }
        assert_eq!(pos.offset(), src.len(), "did not reach EOF for {src:?}");
        Ok(())
    })?;

    assert_eq!(runner.stats().rejected_cases, 0, "no rejects\n{runner}");
    assert!(saw_lf.get() > 0, "no LF-carrying token\n{runner}");
    assert!(saw_multibyte.get() > 0, "no multi-byte token\n{runner}");
    assert!(saw_multiline.get() > 0, "no multi-line source\n{runner}");
    assert!(saw_crlf.get() > 0, "no CRLF sequence\n{runner}");
    Ok(())
}
