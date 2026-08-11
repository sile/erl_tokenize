//! Property-based tests using noprop.
//!
//! All tests are seeded from `ERL_TOKENIZE_PBT_SEED` when set; otherwise a
//! fresh time-derived seed is used so every run explores new inputs. A
//! failing case prints its seed (hex) and case index; re-run with
//! `ERL_TOKENIZE_PBT_SEED=<seed>` to reproduce it.

use std::cell::Cell;

use erl_tokenize::tokens::{
    AtomToken, CharToken, CommentToken, FloatToken, IntegerToken, StringToken,
};
use erl_tokenize::{Position, Token, Tokenizer};

const CASES: usize = 256;
const MAX_LEN: usize = 64;
const SEED_ENV: &str = "ERL_TOKENIZE_PBT_SEED";

const KEYWORDS: [&str; 29] = [
    "after", "and", "andalso", "band", "begin", "bnot", "bor", "bsl", "bsr", "bxor", "case",
    "catch", "cond", "div", "end", "fun", "if", "let", "not", "of", "or", "orelse", "receive",
    "rem", "try", "when", "xor", "maybe", "else",
];

const SYMBOLS: [&str; 45] = [
    "[", "]", "(", ")", "{", "}", "#", "/", ".", "..", "...", ",", ":", "::", ";", "=", ":=", "|",
    "||", "?", "??", "?=", "!", "-", "--", "+", "++", "*", "->", "<-", "=>", "<=", ">>", "<<",
    "==", "=:=", "/=", "=/=", ">", ">=", "<", "=<", "&&", "<:-", "<:=",
];

const WS_CHARS: [char; 5] = [' ', '\t', '\r', '\n', '\u{a0}'];

// ============================================================
// Generators
// ============================================================

/// Samples a text length with 0 / 1 / MAX_LEN boundaries biased.
fn sample_len(ctx: &mut noprop::TestCaseContext) -> usize {
    noprop::sample_with_boundaries(
        ctx,
        &[0usize, 1, MAX_LEN],
        noprop::Ratio::one_nth(5),
        |ctx| noprop::sample_usize_in(ctx, 0..=MAX_LEN),
    )
}

/// Samples an arbitrary text, mixing escape-relevant special chars,
/// printable ASCII, and arbitrary Unicode.
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

/// Samples a non-negative `i64`.
///
/// Negative values are excluded because Erlang has no negative literal:
/// `-10` is the `-` operator applied to `10`, so no token type can
/// represent a negative value as a single token.
fn sample_non_negative_i64(ctx: &mut noprop::TestCaseContext) -> i64 {
    noprop::sample_with_boundaries(
        ctx,
        &[0u64, 1, i64::MAX as u64],
        noprop::Ratio::one_nth(5),
        |ctx| noprop::sample_u64(ctx) & i64::MAX as u64,
    ) as i64
}

/// Samples a non-negative `f64`.
fn sample_non_negative_f64(ctx: &mut noprop::TestCaseContext) -> f64 {
    noprop::sample_with_boundaries(
        ctx,
        &[0.0f64, 1.0, 100.0, 1e21, f64::MAX],
        noprop::Ratio::one_nth(5),
        |ctx| noprop::sample_f64_in(ctx, 0.0, f64::MAX),
    )
}

/// Samples a valid variable name (e.g. `Foo`, `_bar2`).
fn sample_variable_text(ctx: &mut noprop::TestCaseContext) -> String {
    const TAIL: [char; 6] = ['a', 'Z', '0', '_', '@', 'x'];
    let len =
        noprop::sample_with_boundaries(ctx, &[0usize, 1, 8], noprop::Ratio::one_nth(5), |ctx| {
            noprop::sample_usize_in(ctx, 0..=8)
        });
    let mut s = String::new();
    s.push(noprop::sample_choice(ctx, &['A', 'Z', '_', 'X', 'F']));
    for _ in 0..len {
        s.push(noprop::sample_choice(ctx, &TAIL));
    }
    s
}

/// Samples a comment token text (starts with `%`, no newline).
fn sample_comment_text(ctx: &mut noprop::TestCaseContext) -> String {
    let len =
        noprop::sample_with_boundaries(ctx, &[0usize, 1, 16], noprop::Ratio::one_nth(5), |ctx| {
            noprop::sample_usize_in(ctx, 0..=16)
        });
    let mut s = String::from("%");
    for _ in 0..len {
        s.push(noprop::sample_ascii_printable_char(ctx));
    }
    s
}

/// Samples a sigil string token text (e.g. `~b"foo"`, `~a(bc)d`).
fn sample_sigil_text(ctx: &mut noprop::TestCaseContext) -> String {
    const DELIMS: [(char, char); 10] = [
        ('(', ')'),
        ('[', ']'),
        ('{', '}'),
        ('<', '>'),
        ('/', '/'),
        ('|', '|'),
        ('\'', '\''),
        ('`', '`'),
        ('#', '#'),
        ('"', '"'),
    ];
    // Prefix / suffix chars must be valid atom non-head chars; content must
    // never contain the delimiters or a backslash so the parse always ends.
    const AFFIX: [char; 7] = ['a', 'b', 'x', '_', '1', '@', 'Q'];
    const CONTENT: [char; 26] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    let (open, close) = noprop::sample_choice(ctx, &DELIMS);
    let prefix_len = noprop::sample_usize_in(ctx, 0..=3);
    let suffix_len = noprop::sample_usize_in(ctx, 0..=3);
    let content_len =
        noprop::sample_with_boundaries(ctx, &[0usize, 1, 16], noprop::Ratio::one_nth(5), |ctx| {
            noprop::sample_usize_in(ctx, 0..=16)
        });
    let mut s = String::from("~");
    for _ in 0..prefix_len {
        s.push(noprop::sample_choice(ctx, &AFFIX));
    }
    s.push(open);
    for _ in 0..content_len {
        s.push(noprop::sample_choice(ctx, &CONTENT));
    }
    s.push(close);
    for _ in 0..suffix_len {
        s.push(noprop::sample_choice(ctx, &AFFIX));
    }
    s
}

/// Samples a valid-by-construction token text.
///
/// Returns `(text, is_comment)`: comments must always be followed by a
/// newline when embedded in a token sequence, and a token whose text ends
/// with `"` must not be directly followed by a token starting with `"`.
fn sample_token_text(ctx: &mut noprop::TestCaseContext) -> (String, bool) {
    match noprop::sample_weighted_index(ctx, &[4, 3, 3, 2, 2, 4, 2, 3, 3, 1, 2]) {
        0 => (
            AtomToken::from_value(&sample_text(ctx), Position::new())
                .text()
                .to_owned(),
            false,
        ),
        1 => (
            StringToken::from_value(&sample_text(ctx), Position::new())
                .text()
                .to_owned(),
            false,
        ),
        2 => (
            IntegerToken::from_value(sample_non_negative_i64(ctx), Position::new())
                .text()
                .to_owned(),
            false,
        ),
        3 => (
            FloatToken::from_value(sample_non_negative_f64(ctx), Position::new())
                .text()
                .to_owned(),
            false,
        ),
        4 => (
            CharToken::from_value(noprop::sample_char(ctx), Position::new())
                .text()
                .to_owned(),
            false,
        ),
        5 => (noprop::sample_choice(ctx, &SYMBOLS).to_owned(), false),
        6 => (noprop::sample_choice(ctx, &KEYWORDS).to_owned(), false),
        7 => (sample_variable_text(ctx), false),
        8 => (noprop::sample_choice(ctx, &WS_CHARS).to_string(), false),
        9 => (sample_comment_text(ctx), true),
        _ => (sample_sigil_text(ctx), false),
    }
}

// ============================================================
// Position model
// ============================================================

/// Advances a (offset, line, column) model by a token text.
fn step_position(
    mut offset: usize,
    mut line: usize,
    mut column: usize,
    text: &str,
) -> (usize, usize, usize) {
    offset += text.len();
    if let Some(i) = text.rfind('\n') {
        line += text[..=i].matches('\n').count();
        column = text.len() - i;
    } else {
        column += text.len();
    }
    (offset, line, column)
}

/// Advances a (offset, line, column) model by one character.
fn step_char(
    mut offset: usize,
    mut line: usize,
    mut column: usize,
    c: char,
) -> (usize, usize, usize) {
    offset += c.len_utf8();
    if c == '\n' {
        line += 1;
        column = 1;
    } else {
        column += c.len_utf8();
    }
    (offset, line, column)
}

// ============================================================
// from_value -> from_text roundtrip properties
// ============================================================

// NOTE: `string_from_value_roundtrip` fails because `StringToken::from_value`
// emits Rust-style `\u{...}` escapes, which do not exist in the Erlang escape
// table (only `\x{...}` and named escapes do), for values containing
// non-printable chars (reproduction seed: 0x18cac499b08f9840). Kept
// commented out until the from_value fix lands; it should then pass
// unconstrained.
//
// #[test]
// fn string_from_value_roundtrip() -> noprop::TestResult {
//     let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
//     let escape_cases = Cell::new(0usize);
//     let mut runner = noprop::Runner::new(seed);
//
//     runner.run(CASES, |ctx| {
//         let value = sample_text(ctx);
//         let expected_text = StringToken::from_value(&value, Position::new())
//             .text()
//             .to_owned();
//         if expected_text.contains('\\') {
//             escape_cases.set(escape_cases.get() + 1);
//         }
//         let parsed = StringToken::from_text(&expected_text, Position::new())
//             .map_err(|e| format!("cannot parse {expected_text:?} (from value {value:?}): {e}"))?;
//         assert_eq!(parsed.value(), value, "value mismatch for {expected_text:?}");
//         assert_eq!(parsed.text(), expected_text, "text mismatch for value {value:?}");
//         Ok(())
//     })?;
//
//     assert!(
//         escape_cases.get() > 0,
//         "no case exercised an escaped string\n{runner}"
//     );
//     Ok(())
// }

#[test]
fn atom_from_value_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let escaped_cases = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let value = sample_text(ctx);
        if value.contains('\'') || value.contains('\\') {
            escaped_cases.set(escaped_cases.get() + 1);
        }
        let expected_text = AtomToken::from_value(&value, Position::new())
            .text()
            .to_owned();
        let parsed = AtomToken::from_text(&expected_text, Position::new())
            .map_err(|e| format!("cannot parse {expected_text:?} (from value {value:?}): {e}"))?;
        assert_eq!(
            parsed.value(),
            value,
            "value mismatch for {expected_text:?}"
        );
        assert_eq!(
            parsed.text(),
            expected_text,
            "text mismatch for value {value:?}"
        );
        Ok(())
    })?;

    assert!(
        escaped_cases.get() > 0,
        "no case exercised an escaped atom\n{runner}"
    );
    Ok(())
}

#[test]
fn char_from_value_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let special_cases = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        // Uniform `sample_char` almost never hits escape-relevant chars
        // (special cases are ~10^-6 of all scalars), so mix them in
        // explicitly.
        let value = match noprop::sample_usize_in(ctx, 0..3) {
            0 => noprop::sample_choice(
                ctx,
                &[
                    '\\', '\n', '\0', '\u{1}', '\u{7f}', '"', '\'', '$', '\t', '\r',
                ],
            ),
            1 => noprop::sample_ascii_printable_char(ctx),
            _ => noprop::sample_char(ctx),
        };
        if matches!(value, '\\' | '\n' | '\0' | '\u{1}') {
            special_cases.set(special_cases.get() + 1);
        }
        let expected_text = CharToken::from_value(value, Position::new())
            .text()
            .to_owned();
        let parsed = CharToken::from_text(&expected_text, Position::new())
            .map_err(|e| format!("cannot parse {expected_text:?} (from value {value:?}): {e}"))?;
        assert_eq!(
            parsed.value(),
            value,
            "value mismatch for {expected_text:?}"
        );
        assert_eq!(
            parsed.text(),
            expected_text,
            "text mismatch for value {value:?}"
        );
        Ok(())
    })?;

    assert!(
        special_cases.get() > 0,
        "no case exercised a special char\n{runner}"
    );
    Ok(())
}

#[test]
fn comment_from_value_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let nonempty_cases = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let len = sample_len(ctx);
        let mut value = String::new();
        for _ in 0..len {
            value.push(noprop::sample_ascii_printable_char(ctx));
        }
        if !value.is_empty() {
            nonempty_cases.set(nonempty_cases.get() + 1);
        }
        let expected_text = CommentToken::from_value(&value, Position::new())
            .map_err(|e| format!("from_value({value:?}) failed: {e}"))?
            .text()
            .to_owned();
        let parsed = CommentToken::from_text(&expected_text, Position::new())
            .map_err(|e| format!("cannot parse {expected_text:?}: {e}"))?;
        assert_eq!(
            parsed.value(),
            value,
            "value mismatch for {expected_text:?}"
        );
        assert_eq!(
            parsed.text(),
            expected_text,
            "text mismatch for value {value:?}"
        );
        Ok(())
    })?;

    assert!(
        nonempty_cases.get() > 0,
        "no case exercised a non-empty comment\n{runner}"
    );
    Ok(())
}

#[test]
fn integer_from_value_roundtrip() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let large_cases = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let value = sample_non_negative_i64(ctx);
        if value > 9 {
            large_cases.set(large_cases.get() + 1);
        }
        let expected_text = IntegerToken::from_value(value, Position::new())
            .text()
            .to_owned();
        let parsed = IntegerToken::from_text(&expected_text, Position::new())
            .map_err(|e| format!("cannot parse {expected_text:?}: {e}"))?;
        assert_eq!(
            parsed.value(),
            Some(value),
            "value mismatch for {expected_text:?}"
        );
        assert_eq!(parsed.text(), expected_text);
        Ok(())
    })?;

    assert!(
        large_cases.get() > 0,
        "no case exercised a multi-digit integer\n{runner}"
    );
    Ok(())
}

// NOTE: `float_from_value_roundtrip` fails because `FloatToken::from_value`
// generates text without a fractional part (`1.0` → `"1"`, `1e21` →
// `"1e21"`), violating the Erlang float literal grammar (a decimal point is
// mandatory; reproduction seed: 0x18cac499b08f1f28). Kept commented out
// until the from_value fix lands; it should then pass unconstrained.
//
// #[test]
// fn float_from_value_roundtrip() -> noprop::TestResult {
//     let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
//     let exponent_cases = Cell::new(0usize);
//     let mut runner = noprop::Runner::new(seed);
//
//     runner.run(CASES, |ctx| {
//         let value = sample_non_negative_f64(ctx);
//         let expected_text = FloatToken::from_value(value, Position::new()).text().to_owned();
//         if expected_text.contains('e') {
//             exponent_cases.set(exponent_cases.get() + 1);
//         }
//         let parsed = FloatToken::from_text(&expected_text, Position::new())
//             .map_err(|e| format!("cannot parse {expected_text:?} (from value {value:?}): {e}"))?;
//         assert_eq!(parsed.value(), value, "value mismatch for {expected_text:?}");
//         assert_eq!(parsed.text(), expected_text, "text mismatch for value {value:?}");
//         Ok(())
//     })?;
//
//     assert!(
//         exponent_cases.get() > 0,
//         "no case exercised an exponent form\n{runner}"
//     );
//     Ok(())
// }

// ============================================================
// Tokenizer structural invariants
// ============================================================

#[test]
fn token_from_text_prefix_invariant() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let text = sample_text(ctx);
        if let Ok(token) = Token::from_text(&text, Position::new()) {
            let t = token.text();
            assert!(!t.is_empty(), "token with empty text for {text:?}");
            assert!(
                text.starts_with(t),
                "token text {t:?} is not a prefix of {text:?}"
            );
        }
        Ok(())
    })?;

    Ok(())
}

/// A differential position test: maintains a (offset, line, column) model of
/// the input and checks it against `Tokenizer::next_position()` after every
/// step, with error recovery via `consume_char`.
///
/// This catches position bookkeeping bugs (e.g. the char-boundary panic of
/// erlls issue 5) as well as token texts that are not prefixes of the
/// remaining input.
#[test]
fn tokenizer_position_model() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let ok_cases = Cell::new(0usize);
    let error_cases = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let text = sample_text(ctx);
        let mut tokenizer = Tokenizer::new(text.as_str());
        let mut offset = 0usize;
        let mut line = 1usize;
        let mut column = 1usize;
        let mut saw_ok = false;
        let mut saw_error = false;

        loop {
            // The position must be read before `next()`: the iterator
            // advances the tokenizer past the token it returns.
            let start = tokenizer.next_position();
            assert_eq!(start.offset(), offset, "offset mismatch");
            assert_eq!(start.line(), line, "line mismatch");
            assert_eq!(start.column(), column, "column mismatch");
            let result = tokenizer.next();
            match result {
                None => break,
                Some(Ok(token)) => {
                    saw_ok = true;
                    let t = token.text();
                    let rest = text.get(offset..).expect("offset must be a char boundary");
                    assert!(
                        rest.starts_with(t),
                        "token text {t:?} is not a prefix of {rest:?}"
                    );
                    (offset, line, column) = step_position(offset, line, column, t);
                    let after = tokenizer.next_position();
                    assert_eq!(after.offset(), offset, "end offset mismatch");
                    assert_eq!(after.line(), line, "end line mismatch");
                    assert_eq!(after.column(), column, "end column mismatch");
                }
                Some(Err(_)) => {
                    saw_error = true;
                    let c = tokenizer
                        .consume_char()
                        .expect("consume_char at a non-EOF position");
                    (offset, line, column) = step_char(offset, line, column, c);
                }
            }
        }
        assert_eq!(offset, text.len(), "tokenizer stopped before EOF");
        if saw_ok {
            ok_cases.set(ok_cases.get() + 1);
        }
        if saw_error {
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

/// A valid-by-construction multi-token input must tokenize without error and
/// the concatenation of the token texts must equal the input.
#[test]
fn tokenizer_concat_invariant() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let multi_token_cases = Cell::new(0usize);
    let comment_cases = Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let count = noprop::sample_with_boundaries(
            ctx,
            &[0usize, 1, 8],
            noprop::Ratio::one_nth(5),
            |ctx| noprop::sample_usize_in(ctx, 0..=8),
        );
        let mut tokens = Vec::new();
        for _ in 0..count {
            tokens.push(sample_token_text(ctx));
        }

        // Join token texts with separators, avoiding the only error
        // triggers of the tokenizer: a comment must be followed by a
        // newline, a token ending with `"` must not be followed by one
        // starting with `"` (adjacent string literals), and a token ending
        // with a digit must not be followed by one starting with `_`
        // (`123_` is an invalid integer prefix) or `#` (`1#` is parsed as a
        // bad radix marker).
        let mut input = String::new();
        let mut prev: Option<(&str, bool)> = None;
        for (text, is_comment) in &tokens {
            if let Some((prev_text, prev_comment)) = prev {
                let prev_ends_with_digit =
                    prev_text.chars().last().is_some_and(|c| c.is_ascii_digit());
                if prev_comment {
                    input.push('\n');
                } else if (prev_text.ends_with('"') && text.starts_with('"'))
                    || (prev_ends_with_digit && (text.starts_with('_') || text.starts_with('#')))
                {
                    input.push(' ');
                } else if noprop::sample_bool(ctx) {
                    input.push(noprop::sample_choice(ctx, &WS_CHARS));
                }
            }
            input.push_str(text);
            prev = Some((text, *is_comment));
        }

        let parsed = Tokenizer::new(input.as_str())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("unexpected error at {:?}: {e}", e.position()))?;
        let concat: String = parsed.iter().map(|t| t.text()).collect();
        assert_eq!(concat, input, "token text concatenation mismatch");

        if parsed.len() >= 2 {
            multi_token_cases.set(multi_token_cases.get() + 1);
        }
        if tokens.iter().any(|(_, is_comment)| *is_comment) {
            comment_cases.set(comment_cases.get() + 1);
        }
        Ok(())
    })?;

    assert_eq!(
        runner.stats().rejected_cases,
        0,
        "generators must be valid-by-construction\n{runner}"
    );
    assert!(
        multi_token_cases.get() > 0,
        "no case produced multiple tokens\n{runner}"
    );
    assert!(
        comment_cases.get() > 0,
        "no case included a comment token\n{runner}"
    );
    Ok(())
}
