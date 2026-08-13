//! Position transition model.
//!
//! Maintain an independent `(offset, line, column)` model in the test
//! and assert `Token::start()` / `Token::end()` match at every step.
//! Coverage gates require LF, multi-byte, and multi-line branches to be
//! actually exercised.

use erl_tokenize::{Position, scan_token};

mod pbt_harness;
use pbt_harness::{
    CASES, Counter, SEED_ENV, insert_separator, sample_bare_atom, sample_char_literal,
    sample_comment, sample_decimal_float, sample_decimal_integer, sample_keyword,
    sample_quoted_atom, sample_regular_string, sample_sigil_string, sample_symbol, sample_variable,
    step_char, step_position,
};

#[test]
fn position_model_matches_scan_token() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);

    let saw_lf = Counter::default();
    let saw_multibyte = Counter::default();
    let saw_multiline = Counter::default();

    runner.run(CASES, |ctx| {
        let n = noprop::sample_usize_in(ctx, 0..=8);
        let mut pieces: Vec<String> = Vec::with_capacity(n);
        for _ in 0..n {
            let kind = noprop::sample_usize_in(ctx, 0..10);
            let text = match kind {
                0 => sample_bare_atom(ctx),
                1 => sample_quoted_atom(ctx).0,
                2 => sample_char_literal(ctx).0,
                3 => sample_comment(ctx),
                4 => sample_decimal_integer(ctx).0,
                5 => sample_decimal_float(ctx).0,
                6 => sample_keyword(ctx).0.to_owned(),
                7 => sample_symbol(ctx).to_owned(),
                8 => sample_regular_string(ctx).0,
                _ => {
                    if noprop::sample_bool(ctx) {
                        sample_variable(ctx)
                    } else {
                        sample_sigil_string(ctx).0
                    }
                }
            };
            pieces.push(text);
        }

        let mut src = String::new();
        let mut prev: Option<&str> = None;
        for text in &pieces {
            if let Some(p) = prev {
                src.push(insert_separator(ctx, p));
            }
            src.push_str(text);
            prev = Some(text);
        }

        let mut pos = Position::new();
        let mut model = (0usize, 1usize, 1usize);
        let step_ceiling = src.chars().count() * 2 + 8;
        let mut steps = 0usize;
        while let Some(token) = scan_token(&src, pos)? {
            let text = token.text(&src);
            assert_eq!(token.start().offset(), model.0, "start offset");
            assert_eq!(token.start().line(), model.1, "start line");
            assert_eq!(token.start().column(), model.2, "start column");
            let after = step_position(model, text);
            assert_eq!(token.end().offset(), after.0, "end offset");
            assert_eq!(token.end().line(), after.1, "end line");
            assert_eq!(token.end().column(), after.2, "end column");
            if text.contains('\n') {
                saw_lf.hit();
            }
            if text.chars().any(|c| c.len_utf8() > 1) {
                saw_multibyte.hit();
            }
            if token.end().line() > 1 {
                saw_multiline.hit();
            }
            model = after;
            pos = token.end();
            steps += 1;
            assert!(steps <= step_ceiling, "exceeded step ceiling for {src:?}");
        }
        assert_eq!(pos.offset(), src.len(), "did not reach EOF");
        Ok(())
    })?;

    // Also drive an all-error case through `step_char` so `step_char` is
    // exercised even when no property panic occurs.
    let _ = step_char((0, 1, 1), '\n');

    assert_eq!(runner.stats().rejected_cases, 0, "no rejects\n{runner}");
    assert!(saw_lf.get() > 0, "no LF-carrying token\n{runner}");
    assert!(saw_multibyte.get() > 0, "no multi-byte token\n{runner}");
    assert!(saw_multiline.get() > 0, "no multi-line source\n{runner}");
    Ok(())
}
