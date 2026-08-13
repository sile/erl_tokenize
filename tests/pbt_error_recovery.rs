//! Error recovery and termination.
//!
//! For arbitrary UTF-8 inputs — biased toward anomaly-triggering
//! branches — the scan loop that uses `Token::end()` on success and
//! `Error::resume_position()` on failure must advance by exactly one
//! Unicode scalar on error, keep `(offset, line, column)` in lockstep
//! with an independent model, treat a successful token as a prefix of
//! the remaining input, and terminate at `Ok(None)` within an explicit
//! step ceiling.

use erl_tokenize::{Position, scan_token};

#[expect(dead_code, reason = "shared helpers; this binary uses only a subset")]
mod pbt_harness;
use pbt_harness::{
    CASES, Counter, SEED_ENV, sample_bare_atom, sample_regular_string, sample_text, step_char,
    step_position,
};

/// Bias the generator toward well-known anomaly triggers so the
/// recovery path is exercised in most cases.
fn sample_anomaly_source(ctx: &mut noprop::TestCaseContext) -> String {
    match noprop::sample_weighted_index(ctx, &[2, 2, 2, 2, 2, 2, 2, 2, 2, 3]) {
        0 => {
            let mut s = String::from("'");
            let len = noprop::sample_usize_in(ctx, 0..=8);
            for _ in 0..len {
                s.push(noprop::sample_choice(ctx, &['a', 'b', ' ', '\t']));
            }
            s
        }
        1 => {
            let mut s = String::from("\"");
            let len = noprop::sample_usize_in(ctx, 0..=8);
            for _ in 0..len {
                s.push(noprop::sample_choice(ctx, &['a', 'b', ' ']));
            }
            s
        }
        2 => {
            let mut s = String::from("~(");
            let len = noprop::sample_usize_in(ctx, 0..=8);
            for _ in 0..len {
                s.push(noprop::sample_choice(ctx, &['a', 'b']));
            }
            s
        }
        3 => noprop::sample_choice(
            ctx,
            &[
                "$\\",
                "$\\^!",
                "$\\x",
                "$\\x{",
                "$\\x{zz}",
                "$\\x{110000}",
                r"'\^!'",
                r"'\x'",
                r"'\x{'",
                r"'\x{zz}'",
            ],
        )
        .to_string(),
        4 => {
            let (a, _) = sample_regular_string(ctx);
            let (b, _) = sample_regular_string(ctx);
            format!("{a}{b}")
        }
        5 => noprop::sample_choice(
            ctx,
            &[
                "12_", "12__3", "10#", "37#0", "1#0", "12_.3", "12.3_", "1.0e", "1.0e+", "37#0.0",
                "1#0.0", "2#1.0#", "2#1.0#x", "2#1.0#e", "2#1.0#e-",
            ],
        )
        .to_string(),
        6 => {
            let invalid = noprop::sample_choice(ctx, &["\u{FFFC}", "\u{2603}", "\u{1F600}", "中"]);
            wrap_invalid(ctx, invalid)
        }
        7 => {
            let invalid = noprop::sample_choice(ctx, &["@", "`", "\\"]);
            wrap_invalid(ctx, invalid)
        }
        8 => noprop::sample_choice(
            ctx,
            &[
                // Non-whitespace between the opening quotes and the first LF.
                "\"\"\"foo\n\"\"\"",
                // Opening line OK, never closed.
                "\"\"\"\nfoo",
                // Indented closer, a body line shorter than the indent.
                "\"\"\"\n  foo\n bar\n  \"\"\"",
                // Indented closer, a blank body line.
                "\"\"\"\n  \n  \"\"\"",
            ],
        )
        .to_string(),
        _ => sample_text(ctx),
    }
}

fn wrap_invalid(ctx: &mut noprop::TestCaseContext, invalid: &str) -> String {
    match noprop::sample_weighted_index(ctx, &[2, 2, 1]) {
        0 => invalid.to_owned(),
        1 => format!("{invalid} {}", sample_bare_atom(ctx)),
        _ => format!(
            "{}{invalid}{}",
            sample_bare_atom(ctx),
            sample_bare_atom(ctx)
        ),
    }
}

#[test]
fn scan_loop_terminates_and_advances_on_error() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);

    let ok_hits = Counter::default();
    let err_hits = Counter::default();
    let ascii_err_hits = Counter::default();
    let multibyte_err_hits = Counter::default();
    let inside_token_err_hits = Counter::default();

    runner.run(CASES, |ctx| {
        let src = sample_anomaly_source(ctx);
        let mut pos = Position::new();
        let mut model = (0usize, 1usize, 1usize);
        let mut steps = 0usize;
        let step_ceiling = src.chars().count() + 8;

        loop {
            let before = pos.offset();
            assert_eq!(pos.offset(), model.0, "offset drift for {src:?}");
            assert_eq!(pos.line(), model.1, "line drift for {src:?}");
            assert_eq!(pos.column(), model.2, "column drift for {src:?}");
            match scan_token(&src, pos) {
                Ok(None) => break,
                Ok(Some(token)) => {
                    ok_hits.hit();
                    let text = token.text(&src);
                    let rest = &src[before..];
                    assert!(
                        rest.starts_with(text),
                        "token {text:?} is not a prefix of {rest:?} in {src:?}"
                    );
                    assert!(
                        token.end().offset() > before,
                        "success did not advance offset for {src:?}"
                    );
                    assert_eq!(token.start().offset(), model.0, "start offset for {src:?}");
                    assert_eq!(token.start().line(), model.1, "start line for {src:?}");
                    assert_eq!(token.start().column(), model.2, "start column for {src:?}");
                    let after = step_position(model, text);
                    assert_eq!(token.end().offset(), after.0, "end offset for {src:?}");
                    assert_eq!(token.end().line(), after.1, "end line for {src:?}");
                    assert_eq!(token.end().column(), after.2, "end column for {src:?}");
                    model = after;
                    pos = token.end();
                }
                Err(err) => {
                    err_hits.hit();
                    let c = src[before..]
                        .chars()
                        .next()
                        .expect("non-EOF at error position");
                    let after = step_char(model, c);
                    let resume = err.resume_position();
                    assert_eq!(
                        resume.offset(),
                        after.0,
                        "resume offset is not one scalar for {src:?}"
                    );
                    assert_eq!(resume.line(), after.1, "resume line mismatch for {src:?}");
                    assert_eq!(
                        resume.column(),
                        after.2,
                        "resume column mismatch for {src:?}"
                    );
                    assert!(
                        resume.offset() <= src.len(),
                        "resume beyond source end for {src:?}"
                    );
                    assert!(
                        src.is_char_boundary(resume.offset()),
                        "resume off a UTF-8 boundary for {src:?}"
                    );
                    if c.len_utf8() == 1 {
                        ascii_err_hits.hit();
                    } else {
                        multibyte_err_hits.hit();
                    }
                    if err.position().offset() > before {
                        inside_token_err_hits.hit();
                    }
                    model = after;
                    pos = resume;
                }
            }
            steps += 1;
            assert!(steps <= step_ceiling, "step ceiling exceeded for {src:?}");
        }
        assert_eq!(pos.offset(), src.len(), "did not reach EOF for {src:?}");
        assert_eq!(model.0, src.len(), "model did not reach EOF for {src:?}");
        Ok(())
    })?;

    assert_eq!(runner.stats().rejected_cases, 0, "no rejects\n{runner}");
    assert!(ok_hits.get() > 0, "no success path\n{runner}");
    assert!(err_hits.get() > 0, "no error path\n{runner}");
    assert!(ascii_err_hits.get() > 0, "no ASCII error path\n{runner}");
    assert!(
        multibyte_err_hits.get() > 0,
        "no multi-byte error path\n{runner}"
    );
    assert!(
        inside_token_err_hits.get() > 0,
        "no in-token error path\n{runner}"
    );
    Ok(())
}
