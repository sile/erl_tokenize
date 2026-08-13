//! Lazy value extraction oracle.
//!
//! Generate `(expected value, valid token text)` pairs for every
//! `TokenValue` variant. Scan the text and assert
//! `Token::value(source)` matches the expected value. Coverage gates
//! ensure that `Cow::Borrowed` / `Cow::Owned` splits, integer
//! `Some` / `None` boundary, and every variant are actually exercised.

use std::borrow::Cow;

use erl_tokenize::{Position, TokenKind, TokenValue, scan_token};

mod pbt_harness;
use pbt_harness::{
    CASES, Counter, LabelSet, SEED_ENV, sample_bare_atom, sample_char_literal, sample_comment,
    sample_decimal_float, sample_decimal_integer, sample_keyword, sample_quoted_atom,
    sample_radix_integer, sample_regular_string, sample_sigil_string, sample_symbol,
    sample_variable, sample_whitespace_sequence,
};

/// Expected value for one case. Kept flat so a mismatch reports clearly.
enum Expected {
    Atom(String),
    Char(char),
    Comment(String),
    Float(f64),
    Integer(Option<i64>),
    Keyword(erl_tokenize::Keyword),
    SigilString {
        prefix: String,
        content: String,
        suffix: String,
    },
    String(String),
    Symbol(erl_tokenize::Symbol),
    Variable(String),
    Whitespace(String),
}

fn sample_case(ctx: &mut noprop::TestCaseContext) -> (String, Expected) {
    match noprop::sample_usize_in(ctx, 0..12) {
        0 => {
            let s = sample_bare_atom(ctx);
            (s.clone(), Expected::Atom(s))
        }
        1 => {
            let (text, decoded) = sample_quoted_atom(ctx);
            (text, Expected::Atom(decoded))
        }
        2 => {
            let (text, c) = sample_char_literal(ctx);
            (text, Expected::Char(c))
        }
        3 => {
            let text = sample_comment(ctx);
            let value = text[1..].to_owned();
            (text, Expected::Comment(value))
        }
        4 => {
            let (text, v) = sample_decimal_integer(ctx);
            (text, Expected::Integer(Some(v)))
        }
        5 => {
            let (text, v) = sample_radix_integer(ctx);
            (text, Expected::Integer(Some(v)))
        }
        6 => {
            // Integer overflow: append digits past i64::MAX.
            let text = "9223372036854775808".to_owned();
            (text, Expected::Integer(None))
        }
        7 => {
            let (text, v) = sample_decimal_float(ctx);
            (text, Expected::Float(v))
        }
        8 => {
            let (text, k) = sample_keyword(ctx);
            (text.to_owned(), Expected::Keyword(k))
        }
        9 => {
            let text = sample_symbol(ctx);
            let s = match text {
                "[" => erl_tokenize::Symbol::OpenSquare,
                "]" => erl_tokenize::Symbol::CloseSquare,
                "(" => erl_tokenize::Symbol::OpenParen,
                ")" => erl_tokenize::Symbol::CloseParen,
                "{" => erl_tokenize::Symbol::OpenBrace,
                "}" => erl_tokenize::Symbol::CloseBrace,
                "#" => erl_tokenize::Symbol::Sharp,
                "/" => erl_tokenize::Symbol::Slash,
                "." => erl_tokenize::Symbol::Dot,
                ".." => erl_tokenize::Symbol::DoubleDot,
                "..." => erl_tokenize::Symbol::TripleDot,
                "," => erl_tokenize::Symbol::Comma,
                ":" => erl_tokenize::Symbol::Colon,
                "::" => erl_tokenize::Symbol::DoubleColon,
                ";" => erl_tokenize::Symbol::Semicolon,
                "=" => erl_tokenize::Symbol::Match,
                ":=" => erl_tokenize::Symbol::MapMatch,
                "|" => erl_tokenize::Symbol::VerticalBar,
                "||" => erl_tokenize::Symbol::DoubleVerticalBar,
                "?" => erl_tokenize::Symbol::Question,
                "??" => erl_tokenize::Symbol::DoubleQuestion,
                "?=" => erl_tokenize::Symbol::MaybeMatch,
                "!" => erl_tokenize::Symbol::Bang,
                "-" => erl_tokenize::Symbol::Hyphen,
                "--" => erl_tokenize::Symbol::MinusMinus,
                "+" => erl_tokenize::Symbol::Plus,
                "++" => erl_tokenize::Symbol::PlusPlus,
                "*" => erl_tokenize::Symbol::Multiply,
                "->" => erl_tokenize::Symbol::RightArrow,
                "<-" => erl_tokenize::Symbol::LeftArrow,
                "=>" => erl_tokenize::Symbol::DoubleRightArrow,
                "<=" => erl_tokenize::Symbol::DoubleLeftArrow,
                ">>" => erl_tokenize::Symbol::DoubleRightAngle,
                "<<" => erl_tokenize::Symbol::DoubleLeftAngle,
                "==" => erl_tokenize::Symbol::Eq,
                "=:=" => erl_tokenize::Symbol::ExactEq,
                "/=" => erl_tokenize::Symbol::NotEq,
                "=/=" => erl_tokenize::Symbol::ExactNotEq,
                ">" => erl_tokenize::Symbol::Greater,
                ">=" => erl_tokenize::Symbol::GreaterEq,
                "<" => erl_tokenize::Symbol::Less,
                "=<" => erl_tokenize::Symbol::LessEq,
                "&&" => erl_tokenize::Symbol::DoubleAmpersand,
                "<:-" => erl_tokenize::Symbol::StrictLeftArrow,
                "<:=" => erl_tokenize::Symbol::StrictDoubleLeftArrow,
                _ => unreachable!(),
            };
            (text.to_owned(), Expected::Symbol(s))
        }
        10 => {
            let (text, decoded) = sample_regular_string(ctx);
            (text, Expected::String(decoded))
        }
        _ => {
            if noprop::sample_bool(ctx) {
                let s = sample_variable(ctx);
                (s.clone(), Expected::Variable(s))
            } else if noprop::sample_bool(ctx) {
                let (text, prefix, content, suffix) = sample_sigil_string(ctx);
                (
                    text,
                    Expected::SigilString {
                        prefix,
                        content,
                        suffix,
                    },
                )
            } else {
                let s = sample_whitespace_sequence(ctx);
                if s.is_empty() {
                    // Whitespace must be non-empty; fall back to a single space.
                    (" ".to_owned(), Expected::Whitespace(" ".to_owned()))
                } else {
                    // Only the first aggregated whitespace token is
                    // tested; truncate the expected value accordingly.
                    let head_len = first_ws_token_len(&s);
                    let head = s[..head_len].to_owned();
                    (s, Expected::Whitespace(head))
                }
            }
        }
    }
}

/// Compute the length of the first aggregated whitespace token per the
/// `erl_scan return_white_spaces` rules.
fn first_ws_token_len(s: &str) -> usize {
    let mut chars = s.char_indices();
    let (_, head) = chars.next().expect("non-empty whitespace");
    let mut end = head.len_utf8();
    for (_, c) in chars {
        if matches!(c, ' ' | '\t' | '\r' | '\u{a0}') {
            end += c.len_utf8();
        } else {
            break;
        }
    }
    end
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

    runner.run(CASES, |ctx| {
        let (text, expected) = sample_case(ctx);
        let token = scan_token(&text, Position::new())?.expect("generator produced empty source");
        let value = token.value(&text);
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
            }
            (Expected::Float(exp), TokenValue::Float(v)) => {
                assert_eq!(v, exp, "float value for {text:?}");
                variants.hit("float");
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
            }
            (Expected::Symbol(exp), TokenValue::Symbol(v)) => {
                assert_eq!(v, exp, "symbol value for {text:?}");
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

    // Coverage: every variant reached; both Cow forms exercised; integer
    // Some and None both seen.
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
    Ok(())
}

fn cow_hit(borrowed: &Counter, owned: &Counter, is_borrowed: bool) {
    if is_borrowed {
        borrowed.hit();
    } else {
        owned.hit();
    }
}
