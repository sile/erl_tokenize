//! Shared generators and helpers for the `pbt_*` property-based tests.
//!
//! Cargo compiles every `tests/*.rs` as its own integration-test binary,
//! so each binary only uses a subset of these helpers. Silence `dead_code`
//! on the `mod pbt_harness;` declaration in each consuming file, not here:
//! a crate-level `expect` is unfulfilled when this file is itself a
//! test target (its `pub` items are the crate's public API).

use std::cell::Cell;
use std::collections::BTreeSet;

use erl_tokenize::{Keyword, Symbol};

/// Environment variable used by `noprop::seed_from_env_or_time` to
/// reproduce a failing case.
pub const SEED_ENV: &str = "ERL_TOKENIZE_PBT_SEED";

/// Number of cases to run per property.
pub const CASES: usize = 256;

/// Upper bound on any single generated text length.
pub const MAX_LEN: usize = 64;

/// All Erlang reserved words in canonical text form.
pub const KEYWORDS: [(&str, Keyword); 29] = [
    ("after", Keyword::After),
    ("and", Keyword::And),
    ("andalso", Keyword::Andalso),
    ("band", Keyword::Band),
    ("begin", Keyword::Begin),
    ("bnot", Keyword::Bnot),
    ("bor", Keyword::Bor),
    ("bsl", Keyword::Bsl),
    ("bsr", Keyword::Bsr),
    ("bxor", Keyword::Bxor),
    ("case", Keyword::Case),
    ("catch", Keyword::Catch),
    ("cond", Keyword::Cond),
    ("div", Keyword::Div),
    ("end", Keyword::End),
    ("fun", Keyword::Fun),
    ("if", Keyword::If),
    ("let", Keyword::Let),
    ("not", Keyword::Not),
    ("of", Keyword::Of),
    ("or", Keyword::Or),
    ("orelse", Keyword::Orelse),
    ("receive", Keyword::Receive),
    ("rem", Keyword::Rem),
    ("try", Keyword::Try),
    ("when", Keyword::When),
    ("xor", Keyword::Xor),
    ("maybe", Keyword::Maybe),
    ("else", Keyword::Else),
];

/// All Erlang punctuation symbols in canonical text form.
pub const SYMBOLS: [(&str, Symbol); 45] = [
    ("[", Symbol::OpenSquare),
    ("]", Symbol::CloseSquare),
    ("(", Symbol::OpenParen),
    (")", Symbol::CloseParen),
    ("{", Symbol::OpenBrace),
    ("}", Symbol::CloseBrace),
    ("#", Symbol::Sharp),
    ("#_", Symbol::WildcardRecord),
    ("/", Symbol::Slash),
    (".", Symbol::Dot),
    ("..", Symbol::DoubleDot),
    ("...", Symbol::TripleDot),
    (",", Symbol::Comma),
    (":", Symbol::Colon),
    ("::", Symbol::DoubleColon),
    (";", Symbol::Semicolon),
    ("=", Symbol::Match),
    (":=", Symbol::MapMatch),
    ("|", Symbol::VerticalBar),
    ("||", Symbol::DoubleVerticalBar),
    ("?", Symbol::Question),
    ("?=", Symbol::MaybeMatch),
    ("!", Symbol::Bang),
    ("-", Symbol::Hyphen),
    ("--", Symbol::MinusMinus),
    ("+", Symbol::Plus),
    ("++", Symbol::PlusPlus),
    ("*", Symbol::Multiply),
    ("->", Symbol::RightArrow),
    ("<-", Symbol::LeftArrow),
    ("=>", Symbol::DoubleRightArrow),
    ("<=", Symbol::DoubleLeftArrow),
    (">>", Symbol::DoubleRightAngle),
    ("<<", Symbol::DoubleLeftAngle),
    ("==", Symbol::Eq),
    ("=:=", Symbol::ExactEq),
    ("/=", Symbol::NotEq),
    ("=/=", Symbol::ExactNotEq),
    (">", Symbol::Greater),
    (">=", Symbol::GreaterEq),
    ("<", Symbol::Less),
    ("=<", Symbol::LessEq),
    ("&&", Symbol::DoubleAmpersand),
    ("<:-", Symbol::StrictLeftArrow),
    ("<:=", Symbol::StrictDoubleLeftArrow),
];

/// Whitespace characters recognised by the tokenizer.
pub const WS_CHARS: [char; 5] = [' ', '\t', '\r', '\n', '\u{a0}'];

/// Non-LF whitespace characters (the aggregatable half of a token).
pub const HORIZONTAL_WS: [char; 4] = [' ', '\t', '\r', '\u{a0}'];

// ============================================================
// Generators
// ============================================================

/// Sample a text length with 0 / 1 / MAX_LEN boundaries biased.
pub fn sample_len(ctx: &mut noprop::TestCaseContext) -> usize {
    noprop::sample_with_boundaries(
        ctx,
        &[0usize, 1, MAX_LEN],
        noprop::Ratio::one_nth(5),
        |ctx| noprop::sample_usize_in(ctx, 1..MAX_LEN),
    )
}

/// Sample a token-sequence length with empty / singleton / max biased.
pub fn sample_token_count(ctx: &mut noprop::TestCaseContext) -> usize {
    noprop::sample_with_boundaries(ctx, &[0usize, 1, 8], noprop::Ratio::one_nth(5), |ctx| {
        noprop::sample_usize_in(ctx, 0..=8)
    })
}

/// Sample an arbitrary text mixing escape-relevant specials, printable
/// ASCII, and arbitrary Unicode. This intentionally reaches invalid
/// inputs; property tests that need valid-by-construction sources use
/// [`sample_valid_token_text`].
pub fn sample_text(ctx: &mut noprop::TestCaseContext) -> String {
    const SPECIALS: [char; 12] = [
        '"', '\'', '\\', '\n', '\t', '\r', '\0', '\u{1}', '\u{7f}', '\u{80}', '\u{a0}', '\u{2028}',
    ];
    let len = sample_len(ctx);
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        match noprop::sample_weighted_index(ctx, &[1, 1, 1]) {
            0 => s.push(noprop::sample_choice(ctx, &SPECIALS)),
            1 => s.push(noprop::sample_ascii_printable_char(ctx)),
            _ => s.push(noprop::sample_char(ctx)),
        }
    }
    s
}

/// Sample a bare atom (`foo`, `hello_world`). Never contains escapes.
pub fn sample_bare_atom(ctx: &mut noprop::TestCaseContext) -> String {
    const HEAD: [char; 6] = ['a', 'f', 'z', 'x', 'p', 'q'];
    const TAIL: [char; 8] = ['a', 'z', '0', '9', '_', '@', 'X', 'A'];
    let len = noprop::sample_usize_in(ctx, 0..=6);
    let mut s = String::new();
    s.push(noprop::sample_choice(ctx, &HEAD));
    for _ in 0..len {
        s.push(noprop::sample_choice(ctx, &TAIL));
    }
    // Reject keywords by simple suffixing so the atom doesn't accidentally
    // become a reserved word.
    if KEYWORDS.iter().any(|(k, _)| *k == s) {
        s.push('_');
    }
    s
}

/// Sample a bare atom whose head is a non-ASCII lowercase letter.
pub fn sample_unicode_atom(ctx: &mut noprop::TestCaseContext) -> String {
    const HEAD: [char; 4] = ['é', 'ä', 'ñ', 'ω'];
    const TAIL: [char; 4] = ['a', 'z', '0', '_'];
    let len = noprop::sample_usize_in(ctx, 0..=4);
    let mut s = String::new();
    s.push(noprop::sample_choice(ctx, &HEAD));
    for _ in 0..len {
        s.push(noprop::sample_choice(ctx, &TAIL));
    }
    s
}

/// Sample a quoted atom (`'foo bar'`, `'a\nb'`). May include escapes.
///
/// Returns `(text, decoded_value)` — `text` is the full quoted literal
/// and `decoded_value` is what `TokenValue::Atom` should carry.
pub fn sample_quoted_atom(ctx: &mut noprop::TestCaseContext) -> (String, String) {
    let (inner, decoded) = sample_quoted_body(ctx, '\'');
    (format!("'{inner}'"), decoded)
}

/// Sample a body that would appear inside `'...'` or `"..."`.
/// Returns `(raw_text_between_quotes, decoded_content)`.
pub fn sample_quoted_body(ctx: &mut noprop::TestCaseContext, terminator: char) -> (String, String) {
    const PLAIN: [char; 10] = ['a', 'b', 'c', 'd', 'e', ' ', 'X', '0', '_', '/'];
    let len = noprop::sample_usize_in(ctx, 0..=8);
    let mut raw = String::new();
    let mut decoded = String::new();
    for _ in 0..len {
        match noprop::sample_weighted_index(ctx, &[4, 1]) {
            0 => {
                let c = noprop::sample_choice(ctx, &PLAIN);
                if c == terminator || c == '\\' {
                    raw.push('\\');
                    raw.push(c);
                    decoded.push(c);
                } else {
                    raw.push(c);
                    decoded.push(c);
                }
            }
            _ => {
                let (escape_src, escape_val) = sample_escape(ctx, terminator);
                raw.push_str(&escape_src);
                decoded.push(escape_val);
            }
        }
    }
    (raw, decoded)
}

/// Sample a valid escape sequence. Returns `(source_text, decoded_char)`.
pub fn sample_escape(ctx: &mut noprop::TestCaseContext, terminator: char) -> (String, char) {
    const NAMED: [(char, char); 9] = [
        ('b', 8 as char),
        ('d', 127 as char),
        ('e', 27 as char),
        ('f', 12 as char),
        ('n', '\n'),
        ('r', '\r'),
        ('s', ' '),
        ('t', '\t'),
        ('v', 11 as char),
    ];
    match noprop::sample_weighted_index(ctx, &[3, 2, 2, 2, 2, 3]) {
        0 => {
            let (letter, value) = noprop::sample_choice(ctx, &NAMED);
            (format!("\\{letter}"), value)
        }
        1 => {
            let hi = noprop::sample_usize_in(ctx, 0..16);
            let lo = noprop::sample_usize_in(ctx, 0..16);
            let code = (hi * 16 + lo) as u32;
            let src = format!("\\x{hi:X}{lo:X}");
            (src, char::from_u32(code).expect("byte in Unicode range"))
        }
        2 => {
            let c = noprop::sample_char(ctx);
            (format!("\\x{{{:X}}}", c as u32), c)
        }
        3 => {
            let n = noprop::sample_usize_in(ctx, 0..=0o377);
            (
                format!("\\{n:03o}"),
                char::from_u32(n as u32).expect("octal 0..=255 is a valid scalar"),
            )
        }
        4 => sample_caret_escape(ctx),
        _ => {
            let term_weight = u32::from(matches!(terminator, '\'' | '"'));
            match noprop::sample_weighted_index(ctx, &[term_weight, 2, 1]) {
                0 => (format!("\\{terminator}"), terminator),
                1 => ("\\\\".to_owned(), '\\'),
                _ => {
                    let c = noprop::sample_choice(ctx, &['$', '#', '%', ' ']);
                    (format!("\\{c}"), c)
                }
            }
        }
    }
}

fn sample_caret_escape(ctx: &mut noprop::TestCaseContext) -> (String, char) {
    const CARET_SPECIAL: [char; 7] = ['@', '[', '\\', ']', '^', '_', '?'];
    let c = if noprop::sample_bool(ctx) {
        noprop::sample_choice(ctx, &CARET_SPECIAL)
    } else {
        let off = noprop::sample_usize_in(ctx, 0..26);
        if noprop::sample_bool(ctx) {
            char::from(b'a' + off as u8)
        } else {
            char::from(b'A' + off as u8)
        }
    };
    let value = if c == '?' {
        127 as char
    } else {
        (c as u32 % 32) as u8 as char
    };
    (format!("\\^{c}"), value)
}

/// Sample a character literal like `$a` or `$\n`. Returns
/// `(text, decoded_char)`.
pub fn sample_char_literal(ctx: &mut noprop::TestCaseContext) -> (String, char) {
    match noprop::sample_weighted_index(ctx, &[2, 2, 2]) {
        0 => {
            let (src, c) = sample_escape(ctx, '\0');
            (format!("${src}"), c)
        }
        1 => {
            let plain = noprop::sample_choice(ctx, &['a', 'Z', '0', ' ', '#', '?', '@', '$']);
            (format!("${plain}"), plain)
        }
        _ => {
            let (src, c) = sample_caret_escape(ctx);
            (format!("${src}"), c)
        }
    }
}

/// Sample a comment token text like `%foo`. Never contains LF.
pub fn sample_comment(ctx: &mut noprop::TestCaseContext) -> String {
    match noprop::sample_weighted_index(ctx, &[2, 1, 2]) {
        0 => "%".to_owned(),
        1 => {
            let len = noprop::sample_usize_in(ctx, 0..=8);
            let mut s = String::from("%%");
            for _ in 0..len {
                s.push(noprop::sample_ascii_printable_char(ctx));
            }
            s
        }
        _ => {
            let len = noprop::sample_usize_in(ctx, 0..=16);
            let mut s = String::from("%");
            for _ in 0..len {
                s.push(noprop::sample_ascii_printable_char(ctx));
            }
            s
        }
    }
}

/// Sample a decimal integer literal in the u64 range. Returns
/// `(text, value)`.
pub fn sample_decimal_integer(ctx: &mut noprop::TestCaseContext) -> (String, u64) {
    let v = noprop::sample_with_boundaries(
        ctx,
        &[0_u64, 1, i64::MAX as u64, u64::MAX],
        noprop::Ratio::one_nth(5),
        noprop::sample_u64,
    );
    let text = if noprop::sample_bool(ctx) {
        insert_underscores(&v.to_string(), ctx)
    } else {
        v.to_string()
    };
    (text, v)
}

/// Sample a radix integer literal in the u64 range. Returns
/// `(text, value)`.
pub fn sample_radix_integer(ctx: &mut noprop::TestCaseContext) -> (String, u64) {
    let radix: u32 = noprop::sample_usize_in(ctx, 2..=36) as u32;
    let v = noprop::sample_u64(ctx);
    let digits = to_radix_digits(v, radix);
    (format!("{radix}#{digits}"), v)
}

/// Sample an integer literal whose value exceeds `u64::MAX`.
pub fn sample_overflow_integer(ctx: &mut noprop::TestCaseContext) -> String {
    let extra = noprop::sample_usize_in(ctx, 0..=999) as u128;
    let n = u64::MAX as u128 + 1 + extra;
    match noprop::sample_weighted_index(ctx, &[2, 1]) {
        0 => n.to_string(),
        _ => format!("16#{n:x}"),
    }
}

/// Sample a decimal float. Returns `(text, value)`.
///
/// The pairs are written independently of the tokenizer's decoder: the
/// expected `f64` is a Rust literal, not `text.parse()`.
pub fn sample_decimal_float(ctx: &mut noprop::TestCaseContext) -> (String, f64) {
    const CASES: [(&str, f64); 16] = [
        ("0.0", 0.0),
        ("0.5", 0.5),
        ("1.0", 1.0),
        ("1.25", 1.25),
        ("2.5", 2.5),
        ("3.75", 3.75),
        ("100.5", 100.5),
        ("1234.5", 1234.5),
        ("1.0e2", 100.0),
        ("1.25e2", 125.0),
        ("1.0e+2", 100.0),
        ("2.5e-1", 0.25),
        ("1.0E3", 1000.0),
        ("5.0e-1", 0.5),
        ("8.0e0", 8.0),
        ("4.0e+1", 40.0),
    ];
    let (text, value) = noprop::sample_choice(ctx, &CASES);
    (text.to_owned(), value)
}

/// Sample a radix-prefixed float. Returns `(text, value)`.
pub fn sample_radix_float(ctx: &mut noprop::TestCaseContext) -> (String, f64) {
    const POSITIVE: [(&str, f64); 12] = [
        ("2#1.0", 1.0),
        ("2#1.1", 1.5),
        ("2#0.1", 0.5),
        ("2#0.01", 0.25),
        ("16#1.0", 1.0),
        ("16#1.8", 1.5),
        ("2#1.0#e1", 2.0),
        ("2#1.0#e2", 4.0),
        // Uppercase `E` and a `+` sign are accepted in the exponent too.
        ("2#1.0#E1", 2.0),
        ("2#0.01#E2", 1.0),
        ("2#1.0#e+1", 2.0),
        ("16#1.0#E+1", 16.0),
    ];
    // Negative exponents are a separate decoder branch (`#e-N`). Weight
    // the two sub-sets evenly (rather than proportional to template
    // count) so `pbt_value_extraction`'s `saw_neg_radix_exp` coverage
    // gate fires reliably within `CASES` samples.
    const NEGATIVE: [(&str, f64); 4] = [
        ("2#1.0#e-1", 0.5),
        ("2#1.0#e-2", 0.25),
        ("2#1.1#e-1", 0.75),
        ("16#1.0#e-1", 0.0625),
    ];
    let (text, value) = match noprop::sample_weighted_index(ctx, &[1, 1]) {
        0 => noprop::sample_choice(ctx, &POSITIVE),
        _ => noprop::sample_choice(ctx, &NEGATIVE),
    };
    (text.to_owned(), value)
}

/// Sample a keyword text.
pub fn sample_keyword(ctx: &mut noprop::TestCaseContext) -> (&'static str, Keyword) {
    noprop::sample_choice(ctx, &KEYWORDS)
}

/// Sample a symbol text and its enum value.
pub fn sample_symbol(ctx: &mut noprop::TestCaseContext) -> (&'static str, Symbol) {
    noprop::sample_choice(ctx, &SYMBOLS)
}

/// Sample a variable name.
pub fn sample_variable(ctx: &mut noprop::TestCaseContext) -> String {
    const TAIL: [char; 6] = ['a', 'Z', '0', '_', '@', 'x'];
    let len = noprop::sample_usize_in(ctx, 0..=6);
    let mut s = String::new();
    s.push(noprop::sample_choice(ctx, &['A', 'Z', '_', 'X', 'F']));
    for _ in 0..len {
        s.push(noprop::sample_choice(ctx, &TAIL));
    }
    s
}

/// Sample a regular string literal (may include escapes). Returns
/// `(text, decoded_value)`.
pub fn sample_regular_string(ctx: &mut noprop::TestCaseContext) -> (String, String) {
    let (inner, decoded) = sample_quoted_body(ctx, '"');
    (format!("\"{inner}\""), decoded)
}

/// Sample a triple-quoted string. Returns `(text, decoded_value)`.
///
/// Content avoids `"` so the closer cannot appear in a body line.
/// Indent 0 borrows; indent > 0 requires stripping and is owned.
pub fn sample_triple_quoted_string(ctx: &mut noprop::TestCaseContext) -> (String, String) {
    const CONTENT: [char; 6] = ['a', 'b', 'c', 'x', 'y', ' '];
    let indent =
        noprop::sample_with_boundaries(ctx, &[0usize, 1, 4], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 0..=4)
        });
    let n_lines =
        noprop::sample_with_boundaries(ctx, &[0usize, 1, 4], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 0..=4)
        });
    let pad = " ".repeat(indent);
    let mut decoded_lines = Vec::with_capacity(n_lines);
    let mut text = String::from("\"\"\"\n");
    for _ in 0..n_lines {
        // Indented form rejects a body line that is shorter than `indent`
        // (it has no content character at column `indent`).
        let min_len = usize::from(indent > 0);
        let len = noprop::sample_usize_in(ctx, min_len..=6);
        let mut line = String::new();
        for _ in 0..len {
            line.push(noprop::sample_choice(ctx, &CONTENT));
        }
        text.push_str(&pad);
        text.push_str(&line);
        text.push('\n');
        decoded_lines.push(line);
    }
    text.push_str(&pad);
    text.push_str("\"\"\"");
    (text, decoded_lines.join("\n"))
}

/// Sample a sigil string literal. Returns
/// `(text, prefix, decoded_content, suffix)`.
///
/// `erl_scan` treats the empty prefix, `b`, and `s` as non-verbatim
/// (escape sequences are decoded); every other prefix — including `~B`,
/// `~S`, and multi-letter forms — is verbatim, so the content is emitted
/// literally and `\` is not an escape introducer. The generator picks a
/// verbatim / non-verbatim branch first and then samples a prefix and
/// content consistent with that choice, so `decoded_content` always
/// matches what the tokenizer will produce.
pub fn sample_sigil_string(ctx: &mut noprop::TestCaseContext) -> (String, String, String, String) {
    const DELIMS: [(char, char); 9] = [
        ('(', ')'),
        ('[', ']'),
        ('{', '}'),
        ('<', '>'),
        ('/', '/'),
        ('|', '|'),
        ('\'', '\''),
        ('`', '`'),
        ('#', '#'),
    ];
    const AFFIX: [char; 6] = ['a', 'b', 'x', '_', '1', 'Q'];
    const NON_VERBATIM_PREFIXES: [&str; 3] = ["", "b", "s"];

    let verbatim = noprop::sample_weighted_index(ctx, &[1, 1]) == 0;
    let prefix = if verbatim {
        let mut p = String::new();
        // Ensure the resulting prefix is neither empty nor `b`/`s` so it
        // is truly verbatim; if the sample happens to land on one of
        // those, upgrade the head char to `Q` (a stable member of AFFIX
        // that is neither `b` nor `s`).
        let len = noprop::sample_usize_in(ctx, 1..=3);
        for _ in 0..len {
            p.push(noprop::sample_choice(ctx, &AFFIX));
        }
        if matches!(p.as_str(), "" | "b" | "s") {
            p.insert(0, 'Q');
        }
        p
    } else {
        noprop::sample_choice(ctx, &NON_VERBATIM_PREFIXES).to_owned()
    };
    let suffix_len = noprop::sample_usize_in(ctx, 0..=3);
    let mut suffix = String::new();
    for _ in 0..suffix_len {
        suffix.push(noprop::sample_choice(ctx, &AFFIX));
    }

    match noprop::sample_weighted_index(ctx, &[3, 2, 1]) {
        0 => {
            let (open, close) = noprop::sample_choice(ctx, &DELIMS);
            let (inner, decoded) = if verbatim {
                let raw = sample_verbatim_body(ctx, close);
                (raw.clone(), raw)
            } else {
                sample_quoted_body(ctx, close)
            };
            let text = format!("~{prefix}{open}{inner}{close}{suffix}");
            (text, prefix, decoded, suffix)
        }
        1 => {
            let (inner, decoded) = if verbatim {
                let raw = sample_verbatim_body(ctx, '"');
                (raw.clone(), raw)
            } else {
                sample_quoted_body(ctx, '"')
            };
            let text = format!("~{prefix}\"{inner}\"{suffix}");
            (text, prefix, decoded, suffix)
        }
        _ => {
            let (triple, decoded) = sample_triple_quoted_string(ctx);
            let text = format!("~{prefix}{triple}{suffix}");
            (text, prefix, decoded, suffix)
        }
    }
}

/// Sample the inner body of a verbatim quoted region: no `\` characters
/// (which would be preserved literally and desync the samplers' decoded
/// oracle), and no occurrence of the closing `terminator`.
pub fn sample_verbatim_body(ctx: &mut noprop::TestCaseContext, terminator: char) -> String {
    const PLAIN: [char; 9] = ['a', 'b', 'c', 'd', 'e', ' ', 'X', '0', '_'];
    let len = noprop::sample_usize_in(ctx, 0..=8);
    let mut raw = String::new();
    for _ in 0..len {
        let c = noprop::sample_choice(ctx, &PLAIN);
        if c == terminator {
            continue;
        }
        raw.push(c);
    }
    raw
}

/// Sample a whitespace-only source, including empty / CRLF / consecutive
/// LF / NBSP as explicit branches so coverage gates are not luck-based.
pub fn sample_whitespace_sequence(ctx: &mut noprop::TestCaseContext) -> String {
    sample_whitespace_sequence_ex(ctx, true)
}

/// Sample a non-empty whitespace-only source.
pub fn sample_nonempty_whitespace_sequence(ctx: &mut noprop::TestCaseContext) -> String {
    sample_whitespace_sequence_ex(ctx, false)
}

/// Sample a single aggregated whitespace token (the whole string is one
/// token, so the value oracle is the string itself).
pub fn sample_one_whitespace_token(ctx: &mut noprop::TestCaseContext) -> String {
    match noprop::sample_weighted_index(ctx, &[3, 2, 1]) {
        0 => {
            let len = noprop::sample_with_boundaries(
                ctx,
                &[1usize, 16],
                noprop::Ratio::one_nth(5),
                |ctx| noprop::sample_usize_in(ctx, 1..=16),
            );
            let mut s = String::new();
            for _ in 0..len {
                s.push(noprop::sample_choice(ctx, &HORIZONTAL_WS));
            }
            s
        }
        1 => {
            let len = noprop::sample_usize_in(ctx, 0..=8);
            let mut s = String::from("\n");
            for _ in 0..len {
                s.push(noprop::sample_choice(ctx, &HORIZONTAL_WS));
            }
            s
        }
        _ => noprop::sample_choice(ctx, &['\u{a0}', '\r', '\t']).to_string(),
    }
}

fn sample_whitespace_sequence_ex(ctx: &mut noprop::TestCaseContext, allow_empty: bool) -> String {
    let empty_w = u32::from(allow_empty);
    match noprop::sample_weighted_index(ctx, &[empty_w, 1, 2, 2, 2, 2, 1, 4]) {
        0 => String::new(),
        1 => noprop::sample_choice(ctx, &WS_CHARS).to_string(),
        2 => {
            let mut s = String::new();
            if noprop::sample_bool(ctx) {
                s.push(noprop::sample_choice(ctx, &HORIZONTAL_WS));
            }
            s.push_str("\r\n");
            if noprop::sample_bool(ctx) {
                s.push(noprop::sample_choice(ctx, &HORIZONTAL_WS));
            }
            s
        }
        3 => {
            let mut s = String::from("\n\n");
            if noprop::sample_bool(ctx) {
                s.push(noprop::sample_choice(ctx, &HORIZONTAL_WS));
            }
            s
        }
        4 => {
            let mut s = String::from("\u{a0}");
            let extra = noprop::sample_usize_in(ctx, 0..=4);
            for _ in 0..extra {
                s.push(noprop::sample_choice(ctx, &HORIZONTAL_WS));
            }
            s
        }
        5 => {
            let mut s = String::from("\n");
            let extra = noprop::sample_usize_in(ctx, 0..=4);
            for _ in 0..extra {
                s.push(noprop::sample_choice(ctx, &HORIZONTAL_WS));
            }
            s
        }
        6 => {
            let mut s = String::new();
            for _ in 0..16 {
                s.push(noprop::sample_choice(ctx, &WS_CHARS));
            }
            s
        }
        _ => {
            let len = if allow_empty {
                noprop::sample_with_boundaries(
                    ctx,
                    &[0usize, 1, 16],
                    noprop::Ratio::one_nth(5),
                    |ctx| noprop::sample_usize_in(ctx, 0..=16),
                )
            } else {
                noprop::sample_with_boundaries(
                    ctx,
                    &[1usize, 16],
                    noprop::Ratio::one_nth(5),
                    |ctx| noprop::sample_usize_in(ctx, 1..=16),
                )
            };
            let mut s = String::new();
            for _ in 0..len {
                match noprop::sample_weighted_index(ctx, &[4, 3, 1, 1, 1]) {
                    0 => s.push(' '),
                    1 => s.push('\t'),
                    2 => s.push('\r'),
                    3 => s.push('\n'),
                    _ => s.push('\u{a0}'),
                }
            }
            s
        }
    }
}

/// Sample one valid-by-construction token's source text.
pub fn sample_valid_token_text(ctx: &mut noprop::TestCaseContext) -> String {
    match noprop::sample_weighted_index(
        ctx,
        &[
            3, // bare atom
            1, // unicode atom
            2, // quoted atom
            2, // char
            2, // comment
            2, // decimal integer
            2, // radix integer
            2, // decimal float
            1, // radix float
            2, // keyword
            2, // symbol
            2, // regular string
            1, // triple-quoted string
            2, // variable
            2, // sigil
        ],
    ) {
        0 => sample_bare_atom(ctx),
        1 => sample_unicode_atom(ctx),
        2 => sample_quoted_atom(ctx).0,
        3 => sample_char_literal(ctx).0,
        4 => sample_comment(ctx),
        5 => sample_decimal_integer(ctx).0,
        6 => sample_radix_integer(ctx).0,
        7 => sample_decimal_float(ctx).0,
        8 => sample_radix_float(ctx).0,
        9 => sample_keyword(ctx).0.to_owned(),
        10 => sample_symbol(ctx).0.to_owned(),
        11 => sample_regular_string(ctx).0,
        12 => sample_triple_quoted_string(ctx).0,
        13 => sample_variable(ctx),
        _ => sample_sigil_string(ctx).0,
    }
}

/// Join token texts with valid-by-construction separators.
pub fn join_tokens(ctx: &mut noprop::TestCaseContext, pieces: &[String]) -> String {
    let mut src = String::new();
    let mut prev: Option<&str> = None;
    for text in pieces {
        if let Some(p) = prev {
            src.push_str(&insert_separator(ctx, p));
        }
        src.push_str(text);
        prev = Some(text);
    }
    src
}

// ============================================================
// Utilities
// ============================================================

/// Insert underscores into a digit string at random-but-legal positions
/// (never at the head, tail, or adjacent to another underscore).
fn insert_underscores(digits: &str, ctx: &mut noprop::TestCaseContext) -> String {
    if digits.len() < 2 {
        return digits.to_owned();
    }
    let mut out = String::with_capacity(digits.len());
    for (i, c) in digits.char_indices() {
        out.push(c);
        let can_insert = i + 1 < digits.len()
            && !out.ends_with('_')
            && digits
                .as_bytes()
                .get(i + 1)
                .is_some_and(|b| b.is_ascii_digit());
        if can_insert && noprop::sample_bool(ctx) {
            out.push('_');
        }
    }
    out
}

/// Format an unsigned value in the given radix using Erlang's lowercase
/// alphanumeric alphabet.
fn to_radix_digits(mut v: u64, radix: u32) -> String {
    if v == 0 {
        return "0".to_owned();
    }
    let mut buf = Vec::new();
    while v > 0 {
        let d = (v % u64::from(radix)) as u32;
        v /= u64::from(radix);
        buf.push(char::from_digit(d, radix).expect("digit in range"));
    }
    buf.iter().rev().collect()
}

// ============================================================
// Position model (independent of production's Position helpers)
// ============================================================

/// Advance the model `(offset, line, column)` by walking `text`.
pub fn step_position(
    (mut offset, mut line, mut column): (usize, usize, usize),
    text: &str,
) -> (usize, usize, usize) {
    for c in text.chars() {
        (offset, line, column) = step_char((offset, line, column), c);
    }
    (offset, line, column)
}

/// Advance the model by one Unicode scalar value.
pub fn step_char(
    (mut offset, mut line, mut column): (usize, usize, usize),
    c: char,
) -> (usize, usize, usize) {
    let n = c.len_utf8();
    offset += n;
    if c == '\n' {
        line += 1;
        column = 1;
    } else {
        column += n;
    }
    (offset, line, column)
}

// ============================================================
// Coverage counters
// ============================================================

/// Small newtype over `Cell<usize>` for coverage counting inside a
/// noprop closure (which requires interior mutability).
#[derive(Default)]
pub struct Counter(Cell<usize>);

impl Counter {
    pub fn hit(&self) {
        self.0.set(self.0.get() + 1);
    }
    pub fn get(&self) -> usize {
        self.0.get()
    }
}

/// Collect the set of labels seen. Useful when the property wants to
/// prove that every equivalence class was exercised (e.g. every kind).
#[derive(Default)]
pub struct LabelSet(std::cell::RefCell<BTreeSet<String>>);

impl LabelSet {
    pub fn hit(&self, label: impl Into<String>) {
        self.0.borrow_mut().insert(label.into());
    }
    pub fn contains(&self, label: &str) -> bool {
        self.0.borrow().contains(label)
    }
    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}

// ============================================================
// Multi-token source construction
// ============================================================

/// Insert a whitespace separator between two token texts.
///
/// A previous comment must be followed by LF (comments do not include
/// their terminating newline). Otherwise a non-empty whitespace run is
/// inserted so aggregated whitespace between tokens is in the support.
pub fn insert_separator(ctx: &mut noprop::TestCaseContext, prev: &str) -> String {
    if prev.starts_with('%') {
        let mut s = String::from("\n");
        if noprop::sample_ratio(ctx, noprop::Ratio::one_nth(3)) {
            s.push_str(&sample_nonempty_whitespace_sequence(ctx));
        }
        s
    } else {
        sample_nonempty_whitespace_sequence(ctx)
    }
}
