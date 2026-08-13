//! Property 3: error recovery and termination.
//!
//! For arbitrary UTF-8 inputs — biased toward anomaly-triggering
//! branches — the scan loop that uses `Token::end()` on success and
//! `Error::resume_position()` on failure must strictly advance offset at
//! every step and terminate at `Ok(None)` within an explicit step
//! ceiling.

use erl_tokenize::{Position, scan_token};

mod pbt_harness;
use pbt_harness::{CASES, Counter, SEED_ENV, sample_text};

/// Bias the generator toward well-known anomaly triggers so the
/// recovery path is exercised in most cases.
fn sample_anomaly_source(ctx: &mut noprop::TestCaseContext) -> String {
    match noprop::sample_usize_in(ctx, 0..8) {
        0 => {
            // Unterminated quoted atom.
            let mut s = String::from("'");
            for _ in 0..noprop::sample_usize_in(ctx, 0..=8) {
                s.push(noprop::sample_choice(ctx, &['a', 'b', ' ', '\t']));
            }
            s
        }
        1 => {
            // Unterminated regular string.
            let mut s = String::from("\"");
            for _ in 0..noprop::sample_usize_in(ctx, 0..=8) {
                s.push(noprop::sample_choice(ctx, &['a', 'b', ' ']));
            }
            s
        }
        2 => {
            // Unterminated sigil string with paren delimiter.
            let mut s = String::from("~(");
            for _ in 0..noprop::sample_usize_in(ctx, 0..=8) {
                s.push(noprop::sample_choice(ctx, &['a', 'b']));
            }
            s
        }
        3 => {
            // Invalid escape after a quote or `$`.
            noprop::sample_choice(
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
            .to_string()
        }
        4 => {
            // Adjacent string literals.
            String::from(r#""foo""bar""#)
        }
        5 => {
            // Malformed number.
            noprop::sample_choice(ctx, &["12_", "12__3", "1.", "10#", "37#0"]).to_string()
        }
        6 => {
            // A lone non-ASCII character that cannot start any token.
            let cases: [&str; 4] = ["\u{FFFC}", "\u{2603}", "\u{1F600}", "中"];
            let base = noprop::sample_choice(ctx, &cases);
            let mut s = String::from(base);
            // Optionally follow with valid text so recovery must land
            // cleanly on the boundary.
            if noprop::sample_bool(ctx) {
                s.push_str(" ok");
            }
            s.to_string()
        }
        _ => {
            // Random UTF-8 text (as in sample_text) — sometimes valid,
            // sometimes not.
            sample_text(ctx)
        }
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
        let mut steps = 0usize;
        let step_ceiling = src.chars().count() + 8;

        loop {
            let before = pos.offset();
            match scan_token(&src, pos) {
                Ok(None) => break,
                Ok(Some(token)) => {
                    ok_hits.hit();
                    assert!(
                        token.end().offset() > before,
                        "success did not advance offset"
                    );
                    pos = token.end();
                }
                Err(err) => {
                    err_hits.hit();
                    let resume = err.resume_position();
                    assert!(
                        resume.offset() > before,
                        "resume did not advance offset for {src:?}"
                    );
                    assert!(
                        resume.offset() <= src.len(),
                        "resume beyond source end for {src:?}"
                    );
                    assert!(
                        src.is_char_boundary(resume.offset()),
                        "resume off a UTF-8 boundary for {src:?}"
                    );
                    // Classify the error character for coverage.
                    let c = src[before..]
                        .chars()
                        .next()
                        .expect("non-EOF at error position");
                    if c.len_utf8() == 1 {
                        ascii_err_hits.hit();
                    } else {
                        multibyte_err_hits.hit();
                    }
                    // Diagnostic position may point inside the token
                    // (not at scan-start) even though resume advances
                    // exactly one scalar from scan-start.
                    if err.position().offset() > before {
                        inside_token_err_hits.hit();
                    }
                    pos = resume;
                }
            }
            steps += 1;
            assert!(steps <= step_ceiling, "step ceiling exceeded for {src:?}");
        }
        assert_eq!(pos.offset(), src.len(), "did not reach EOF");
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
