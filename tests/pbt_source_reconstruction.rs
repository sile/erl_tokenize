//! Source reconstruction and monotonic progress.
//!
//! For a valid-by-construction multi-token source, the concatenation of
//! every scanned `erl_tokenize::Token::text(source)` must equal the source, offsets
//! must strictly advance, and the scan must terminate at `source.len()`
//! within a step ceiling derived from the source length.

#[expect(dead_code, reason = "shared helpers; this binary uses only a subset")]
mod pbt_harness;
use pbt_harness::{
    CASES, Counter, LabelSet, SEED_ENV, join_tokens, sample_token_count, sample_valid_token_text,
};

#[test]
fn source_reconstruction_and_monotonic_progress() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);
    let kinds = LabelSet::default();
    let multi_token_cases = Counter::default();
    let empty_cases = Counter::default();
    let saw_triple = Counter::default();
    let saw_unicode_atom = Counter::default();

    runner.run(CASES, |ctx| {
        let n = sample_token_count(ctx);
        let mut pieces: Vec<String> = Vec::with_capacity(n);
        for _ in 0..n {
            pieces.push(sample_valid_token_text(ctx));
        }
        let src = join_tokens(ctx, &pieces);

        if src.is_empty() {
            empty_cases.hit();
            assert_eq!(
                erl_tokenize::scan_token(&src, erl_tokenize::Position::new())?,
                None,
                "empty source produced a token"
            );
            return Ok(());
        }

        let mut concat = String::with_capacity(src.len());
        let mut pos = erl_tokenize::Position::new();
        let mut prev_offset = 0usize;
        let mut token_count = 0usize;
        let step_ceiling = src.chars().count() * 2 + 8;
        while let Some(token) = erl_tokenize::scan_token(&src, pos)? {
            let text = token.text(&src);
            assert!(!text.is_empty(), "empty token text in {src:?}");
            assert!(
                token.end().offset() > prev_offset,
                "offset did not advance in {src:?}"
            );
            concat.push_str(text);
            prev_offset = token.end().offset();
            pos = token.end();
            token_count += 1;
            assert!(
                token_count <= step_ceiling,
                "scan exceeded step ceiling for {src:?}"
            );
            kinds.hit(match token.kind() {
                erl_tokenize::TokenKind::Atom => "atom",
                erl_tokenize::TokenKind::Char => "char",
                erl_tokenize::TokenKind::Comment => "comment",
                erl_tokenize::TokenKind::Float => "float",
                erl_tokenize::TokenKind::Integer => "integer",
                erl_tokenize::TokenKind::Keyword(_) => "keyword",
                erl_tokenize::TokenKind::SigilString => "sigil_string",
                erl_tokenize::TokenKind::String => "string",
                erl_tokenize::TokenKind::Symbol(_) => "symbol",
                erl_tokenize::TokenKind::Variable => "variable",
                erl_tokenize::TokenKind::Whitespace => "whitespace",
            });
            if token.kind() == erl_tokenize::TokenKind::String && text.starts_with("\"\"\"") {
                saw_triple.hit();
            }
            if token.kind() == erl_tokenize::TokenKind::Atom
                && text.chars().any(|c| c.len_utf8() > 1)
            {
                saw_unicode_atom.hit();
            }
        }
        assert_eq!(concat, src, "concat mismatch for {src:?}");
        assert_eq!(pos.offset(), src.len(), "did not reach EOF for {src:?}");
        if token_count >= 2 {
            multi_token_cases.hit();
        }
        Ok(())
    })?;

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
    assert!(
        saw_triple.get() > 0,
        "no triple-quoted string was scanned\n{runner}"
    );
    assert!(
        saw_unicode_atom.get() > 0,
        "no Unicode bare atom was scanned\n{runner}"
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
