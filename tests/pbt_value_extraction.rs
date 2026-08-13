//! Lazy value extraction oracle.
//!
//! Generate `(expected value, valid token text)` pairs for every
//! `TokenValue` variant. Scan the text and assert
//! `Token::value(source)` matches the expected value. Coverage gates
//! ensure that `Cow::Borrowed` / `Cow::Owned` splits, integer
//! `Some` / `None` boundary, and every variant are actually exercised.

use std::borrow::Cow;

use erl_tokenize::{Keyword, Position, Symbol, TokenKind, TokenValue, scan_token};

#[expect(dead_code, reason = "shared helpers; this binary uses only a subset")]
mod pbt_harness;
use pbt_harness::{
    CASES, Counter, LabelSet, SEED_ENV, sample_bare_atom, sample_char_literal, sample_comment,
    sample_decimal_float, sample_decimal_integer, sample_keyword, sample_one_whitespace_token,
    sample_overflow_integer, sample_quoted_atom, sample_radix_float, sample_radix_integer,
    sample_regular_string, sample_sigil_string, sample_symbol, sample_triple_quoted_string,
    sample_unicode_atom, sample_variable,
};

/// Expected value for one case. Kept flat so a mismatch reports clearly.
enum Expected {
    Atom(String),
    Char(char),
    Comment(String),
    Float(f64),
    Integer(Option<u64>),
    Keyword(Keyword),
    SigilString {
        prefix: String,
        content: String,
        suffix: String,
    },
    String(String),
    Symbol(Symbol),
    Variable(String),
    Whitespace(String),
}

fn sample_case(ctx: &mut noprop::TestCaseContext) -> (String, Expected) {
    match noprop::sample_weighted_index(
        ctx,
        &[
            2, // bare atom
            1, // unicode atom
            2, // quoted atom
            2, // char
            2, // comment
            2, // decimal integer
            2, // radix integer
            2, // overflow integer
            2, // decimal float
            2, // radix float
            2, // keyword
            2, // symbol
            2, // regular string
            2, // triple-quoted string
            2, // variable
            2, // sigil
            2, // whitespace
        ],
    ) {
        0 => {
            let s = sample_bare_atom(ctx);
            (s.clone(), Expected::Atom(s))
        }
        1 => {
            let s = sample_unicode_atom(ctx);
            (s.clone(), Expected::Atom(s))
        }
        2 => {
            let (text, decoded) = sample_quoted_atom(ctx);
            (text, Expected::Atom(decoded))
        }
        3 => {
            let (text, c) = sample_char_literal(ctx);
            (text, Expected::Char(c))
        }
        4 => {
            let text = sample_comment(ctx);
            let value = text[1..].to_owned();
            (text, Expected::Comment(value))
        }
        5 => {
            let (text, v) = sample_decimal_integer(ctx);
            (text, Expected::Integer(Some(v)))
        }
        6 => {
            let (text, v) = sample_radix_integer(ctx);
            (text, Expected::Integer(Some(v)))
        }
        7 => {
            let text = sample_overflow_integer(ctx);
            (text, Expected::Integer(None))
        }
        8 => {
            let (text, v) = sample_decimal_float(ctx);
            (text, Expected::Float(v))
        }
        9 => {
            let (text, v) = sample_radix_float(ctx);
            (text, Expected::Float(v))
        }
        10 => {
            let (text, k) = sample_keyword(ctx);
            (text.to_owned(), Expected::Keyword(k))
        }
        11 => {
            let (text, s) = sample_symbol(ctx);
            (text.to_owned(), Expected::Symbol(s))
        }
        12 => {
            let (text, decoded) = sample_regular_string(ctx);
            (text, Expected::String(decoded))
        }
        13 => {
            let (text, decoded) = sample_triple_quoted_string(ctx);
            (text, Expected::String(decoded))
        }
        14 => {
            let s = sample_variable(ctx);
            (s.clone(), Expected::Variable(s))
        }
        15 => {
            let (text, prefix, content, suffix) = sample_sigil_string(ctx);
            (
                text,
                Expected::SigilString {
                    prefix,
                    content,
                    suffix,
                },
            )
        }
        _ => {
            let s = sample_one_whitespace_token(ctx);
            (s.clone(), Expected::Whitespace(s))
        }
    }
}

#[test]
fn token_value_matches_generated_oracle() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);

    let variants = LabelSet::default();
    let borrowed = Counter::default();
    let owned = Counter::default();
    let integer_some = Counter::default();
    let integer_none = Counter::default();
    let saw_triple = Counter::default();
    let saw_radix_float = Counter::default();
    let saw_neg_radix_exp = Counter::default();
    let saw_exponent_float = Counter::default();
    let saw_empty_comment = Counter::default();
    let saw_caret = Counter::default();

    runner.run(CASES, |ctx| {
        let (text, expected) = sample_case(ctx);
        let token = scan_token(&text, Position::new())?.expect("generator produced empty source");
        let value = token.value(&text);
        if text.contains("\\^") {
            saw_caret.hit();
        }
        match (&expected, &value) {
            (Expected::Atom(exp), TokenValue::Atom(v)) => {
                assert_eq!(v.as_ref(), exp, "atom value for {text:?}");
                variants.hit("atom");
                cow_hit(&borrowed, &owned, matches!(v, Cow::Borrowed(_)));
            }
            (Expected::Char(exp), TokenValue::Char(c)) => {
                assert_eq!(c, exp, "char value for {text:?}");
                variants.hit("char");
            }
            (Expected::Comment(exp), TokenValue::Comment(s)) => {
                assert_eq!(s, exp, "comment value for {text:?}");
                variants.hit("comment");
                if exp.is_empty() {
                    saw_empty_comment.hit();
                }
            }
            (Expected::Float(exp), TokenValue::Float(v)) => {
                assert_eq!(v, exp, "float value for {text:?}");
                variants.hit("float");
                if text.contains('#') {
                    saw_radix_float.hit();
                }
                if text.contains("#e-") {
                    saw_neg_radix_exp.hit();
                }
                if text.contains('e') || text.contains('E') {
                    saw_exponent_float.hit();
                }
            }
            (Expected::Integer(exp), TokenValue::Integer(v)) => {
                assert_eq!(v, exp, "integer value for {text:?}");
                variants.hit("integer");
                match exp {
                    Some(_) => integer_some.hit(),
                    None => integer_none.hit(),
                }
            }
            (Expected::Keyword(exp), TokenValue::Keyword(v)) => {
                assert_eq!(v, exp, "keyword value for {text:?}");
                assert_eq!(token.text(&text), v.as_str(), "keyword as_str for {text:?}");
                variants.hit("keyword");
            }
            (
                Expected::SigilString {
                    prefix: ep,
                    content: ec,
                    suffix: es,
                },
                TokenValue::SigilString {
                    prefix,
                    content,
                    suffix,
                },
            ) => {
                assert_eq!(prefix, ep, "sigil prefix for {text:?}");
                assert_eq!(content.as_ref(), ec, "sigil content for {text:?}");
                assert_eq!(suffix, es, "sigil suffix for {text:?}");
                variants.hit("sigil_string");
                cow_hit(&borrowed, &owned, matches!(content, Cow::Borrowed(_)));
            }
            (Expected::String(exp), TokenValue::String(v)) => {
                assert_eq!(v.as_ref(), exp, "string value for {text:?}");
                variants.hit("string");
                cow_hit(&borrowed, &owned, matches!(v, Cow::Borrowed(_)));
                if text.starts_with("\"\"\"") {
                    saw_triple.hit();
                }
            }
            (Expected::Symbol(exp), TokenValue::Symbol(v)) => {
                assert_eq!(v, exp, "symbol value for {text:?}");
                assert_eq!(token.text(&text), v.as_str(), "symbol as_str for {text:?}");
                variants.hit("symbol");
            }
            (Expected::Variable(exp), TokenValue::Variable(v)) => {
                assert_eq!(v, exp, "variable value for {text:?}");
                variants.hit("variable");
            }
            (Expected::Whitespace(exp), TokenValue::Whitespace(v)) => {
                assert_eq!(v, exp, "whitespace value for {text:?}");
                assert_eq!(token.kind(), TokenKind::Whitespace);
                variants.hit("whitespace");
            }
            _ => panic!("kind mismatch: {value:?} for {text:?}"),
        }
        Ok(())
    })?;

    assert_eq!(runner.stats().rejected_cases, 0, "no rejects\n{runner}");
    for name in [
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
        assert!(variants.contains(name), "no {name} case\n{runner}");
    }
    assert!(borrowed.get() > 0, "no borrowed Cow case\n{runner}");
    assert!(owned.get() > 0, "no owned Cow case\n{runner}");
    assert!(integer_some.get() > 0, "no Integer(Some) case\n{runner}");
    assert!(integer_none.get() > 0, "no Integer(None) case\n{runner}");
    assert!(saw_triple.get() > 0, "no triple-quoted string\n{runner}");
    assert!(saw_radix_float.get() > 0, "no radix float\n{runner}");
    assert!(
        saw_neg_radix_exp.get() > 0,
        "no negative-exponent radix float\n{runner}"
    );
    assert!(saw_exponent_float.get() > 0, "no exponent float\n{runner}");
    assert!(saw_empty_comment.get() > 0, "no empty comment\n{runner}");
    assert!(saw_caret.get() > 0, "no caret escape\n{runner}");
    Ok(())
}

fn cow_hit(borrowed: &Counter, owned: &Counter, is_borrowed: bool) {
    if is_borrowed {
        borrowed.hit();
    } else {
        owned.hit();
    }
}
