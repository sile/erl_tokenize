//! Source reconstruction and monotonic progress.
//!
//! For a valid-by-construction multi-token source, the concatenation of
//! every scanned `Token::text(source)` must equal the source, offsets
//! must strictly advance, and the scan must terminate at `source.len()`
//! within a step ceiling derived from the source length.

use erl_tokenize::{Position, TokenKind, scan_token};

mod pbt_harness;
use pbt_harness::{
    CASES, Counter, LabelSet, SEED_ENV, insert_separator, sample_bare_atom, sample_char_literal,
    sample_comment, sample_decimal_float, sample_decimal_integer, sample_keyword,
    sample_quoted_atom, sample_radix_integer, sample_regular_string, sample_sigil_string,
    sample_symbol, sample_variable,
};

#[test]
fn source_reconstruction_and_monotonic_progress() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);
    let kinds = LabelSet::default();
    let multi_token_cases = Counter::default();
    let empty_cases = Counter::default();

    runner.run(CASES, |ctx| {
        // Number of tokens: 0 / 1 / up to 8, with the boundaries biased.
        let n = noprop::sample_with_boundaries(
            ctx,
            &[0usize, 1, 8],
            noprop::Ratio::one_nth(5),
            |ctx| noprop::sample_usize_in(ctx, 0..=8),
        );

        let mut pieces: Vec<String> = Vec::with_capacity(n);
        for _ in 0..n {
            let kind = noprop::sample_usize_in(ctx, 0..11);
            let text = match kind {
                0 => sample_bare_atom(ctx),
                1 => sample_quoted_atom(ctx).0,
                2 => sample_char_literal(ctx).0,
                3 => sample_comment(ctx),
                4 => sample_decimal_integer(ctx).0,
                5 => sample_radix_integer(ctx).0,
                6 => sample_decimal_float(ctx).0,
                7 => sample_keyword(ctx).0.to_owned(),
                8 => sample_symbol(ctx).to_owned(),
                9 => sample_regular_string(ctx).0,
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

        // Join with mandatory separators (whitespace or LF for comments).
        let mut src = String::new();
        let mut prev: Option<&str> = None;
        for text in &pieces {
            if let Some(prev_text) = prev {
                src.push(insert_separator(ctx, prev_text));
            }
            src.push_str(text);
            prev = Some(text);
        }

        if src.is_empty() {
            empty_cases.hit();
            assert_eq!(scan_token(&src, Position::new())?, None);
            return Ok(());
        }

        let mut concat = String::with_capacity(src.len());
        let mut pos = Position::new();
        let mut prev_offset = 0usize;
        let mut token_count = 0usize;
        let step_ceiling = src.chars().count() * 2 + 8;
        while let Some(token) = scan_token(&src, pos)? {
            let text = token.text(&src);
            assert!(!text.is_empty(), "empty token text");
            assert!(token.end().offset() > prev_offset, "offset did not advance");
            concat.push_str(text);
            prev_offset = token.end().offset();
            pos = token.end();
            token_count += 1;
            assert!(
                token_count <= step_ceiling,
                "scan exceeded step ceiling for {src:?}"
            );
            kinds.hit(match token.kind() {
                TokenKind::Atom => "atom",
                TokenKind::Char => "char",
                TokenKind::Comment => "comment",
                TokenKind::Float => "float",
                TokenKind::Integer => "integer",
                TokenKind::Keyword(_) => "keyword",
                TokenKind::SigilString => "sigil_string",
                TokenKind::String => "string",
                TokenKind::Symbol(_) => "symbol",
                TokenKind::Variable => "variable",
                TokenKind::Whitespace => "whitespace",
            });
        }
        assert_eq!(concat, src, "concat mismatch");
        assert_eq!(pos.offset(), src.len(), "did not reach EOF");
        if token_count >= 2 {
            multi_token_cases.hit();
        }
        Ok(())
    })?;

    // Coverage gates.
    assert_eq!(
        runner.stats().rejected_cases,
        0,
        "generators must be valid-by-construction\n{runner}"
    );
    assert!(
        empty_cases.get() > 0,
        "no empty-source case exercised\n{runner}"
    );
    assert!(
        multi_token_cases.get() > 0,
        "no multi-token case exercised\n{runner}"
    );
    for expected in [
        "atom",
        "char",
        "comment",
        "float",
        "integer",
        "keyword",
        "sigil_string",
        "string",
        "symbol",
        "variable",
        "whitespace",
    ] {
        assert!(
            kinds.contains(expected),
            "no {expected} token was scanned in any case\n{runner}"
        );
    }
    Ok(())
}
