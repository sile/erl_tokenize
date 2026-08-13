//! Shared generators and helpers for the `pbt_*` property-based tests.
//!
//! Each property test binary reuses only a subset of these helpers, so
//! dead-code warnings are silenced for the whole module.

#![allow(dead_code)]

use std::cell::Cell;
use std::collections::BTreeSet;

use erl_tokenize::Keyword;

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
pub const SYMBOLS: [&str; 45] = [
    "[", "]", "(", ")", "{", "}", "#", "/", ".", "..", "...", ",", ":", "::", ";", "=", ":=", "|",
    "||", "?", "??", "?=", "!", "-", "--", "+", "++", "*", "->", "<-", "=>", "<=", ">>", "<<",
    "==", "=:=", "/=", "=/=", ">", ">=", "<", "=<", "&&", "<:-", "<:=",
];

/// Whitespace characters recognised by the tokenizer.
pub const WS_CHARS: [char; 5] = [' ', '\t', '\r', '\n', '\u{a0}'];

// ============================================================
// Generators
// ============================================================

/// Sample a text length with 0 / 1 / MAX_LEN boundaries biased.
pub fn sample_len(ctx: &mut noprop::TestCaseContext) -> usize {
    noprop::sample_with_boundaries(
        ctx,
        &[0usize, 1, MAX_LEN],
        noprop::Ratio::one_nth(5),
        |ctx| noprop::sample_usize_in(ctx, 0..=MAX_LEN),
    )
}

/// Sample an arbitrary text mixing escape-relevant specials, printable
/// ASCII, and arbitrary Unicode. This intentionally reaches invalid
/// inputs; property tests that need valid-by-construction sources use
/// the per-kind generators below.
pub fn sample_text(ctx: &mut noprop::TestCaseContext) -> String {
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

/// Sample a quoted atom (`'foo bar'`, `'a\nb'`). May include escapes.
///
/// Returns `(text, decoded_value)` — `text` is the full quoted literal
/// and `decoded_value` is what `TokenValue::Atom` should carry.
pub fn sample_quoted_atom(ctx: &mut noprop::TestCaseContext) -> (String, String) {
    let (inner, decoded) = sample_quoted_body(ctx, '\'');
    (format!("'{inner}'"), decoded)
}

/// Sample a body that would appear inside `'...'` or `"..."`, avoiding
/// the given terminator character. Returns (raw_text_between_quotes,
/// decoded_content).
pub fn sample_quoted_body(ctx: &mut noprop::TestCaseContext, terminator: char) -> (String, String) {
    // Small alphabet plus optional escape emissions.
    const PLAIN: [char; 10] = ['a', 'b', 'c', 'd', 'e', ' ', 'X', '0', '_', '/'];
    let len = noprop::sample_usize_in(ctx, 0..=8);
    let mut raw = String::new();
    let mut decoded = String::new();
    for _ in 0..len {
        // 4-in-5 plain char, 1-in-5 escape.
        match noprop::sample_usize_in(ctx, 0..5) {
            0 => {
                let (escape_src, escape_val) = sample_escape(ctx, terminator);
                raw.push_str(&escape_src);
                decoded.push(escape_val);
            }
            _ => {
                let c = noprop::sample_choice(ctx, &PLAIN);
                if c == terminator || c == '\\' {
                    // Escape it to keep the body valid-by-construction.
                    raw.push('\\');
                    raw.push(c);
                    decoded.push(c);
                } else {
                    raw.push(c);
                    decoded.push(c);
                }
            }
        }
    }
    (raw, decoded)
}

/// Sample a valid escape sequence. Returns (source_text, decoded_char).
pub fn sample_escape(ctx: &mut noprop::TestCaseContext, _terminator: char) -> (String, char) {
    // Named escapes carry no ambiguity across terminators.
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
    match noprop::sample_usize_in(ctx, 0..4) {
        0 => {
            let (letter, value) = noprop::sample_choice(ctx, &NAMED);
            (format!("\\{letter}"), value)
        }
        1 => {
            // Fixed-width hex escape: \xNN.
            let hi = noprop::sample_usize_in(ctx, 0..16);
            let lo = noprop::sample_usize_in(ctx, 0..16);
            let code = (hi * 16 + lo) as u32;
            let src = format!("\\x{hi:X}{lo:X}");
            (src, char::from_u32(code).expect("byte in Unicode range"))
        }
        2 => {
            // Braced hex escape: \x{...}.
            let code = loop {
                let candidate = noprop::sample_u32(ctx) & 0x10_FFFF;
                if let Some(c) = char::from_u32(candidate) {
                    break c;
                }
            };
            (format!("\\x{{{:X}}}", code as u32), code)
        }
        _ => {
            // Octal escape: always three digits (zero-padded) so that a
            // following plain digit cannot be re-absorbed by the
            // scanner's up-to-3-digits octal rule.
            let n = noprop::sample_usize_in(ctx, 0..=0o377);
            (format!("\\{n:03o}"), char::from_u32(n as u32).unwrap())
        }
    }
}

/// Sample a character literal like `$a` or `$\n`. Returns
/// (text, decoded_char).
pub fn sample_char_literal(ctx: &mut noprop::TestCaseContext) -> (String, char) {
    if noprop::sample_bool(ctx) {
        let (src, c) = sample_escape(ctx, '\0');
        (format!("${src}"), c)
    } else {
        // Any single non-`\` character.
        let plain = noprop::sample_choice(ctx, &['a', 'Z', '0', ' ', '#', '?', '@']);
        (format!("${plain}"), plain)
    }
}

/// Sample a comment token text like `%foo`. Never contains LF.
pub fn sample_comment(ctx: &mut noprop::TestCaseContext) -> String {
    let len = noprop::sample_usize_in(ctx, 0..=16);
    let mut s = String::from("%");
    for _ in 0..len {
        s.push(noprop::sample_ascii_printable_char(ctx));
    }
    s
}

/// Sample a decimal integer literal in the i64 range. Returns
/// `(text, value)`.
pub fn sample_decimal_integer(ctx: &mut noprop::TestCaseContext) -> (String, i64) {
    let v = noprop::sample_with_boundaries(
        ctx,
        &[0_u64, 1, i64::MAX as u64],
        noprop::Ratio::one_nth(5),
        |ctx| noprop::sample_u64(ctx) & i64::MAX as u64,
    ) as i64;
    let text = if noprop::sample_bool(ctx) {
        insert_underscores(&v.to_string(), ctx)
    } else {
        v.to_string()
    };
    (text, v)
}

/// Sample a radix integer literal in the i64 range. Returns
/// `(text, value)`.
pub fn sample_radix_integer(ctx: &mut noprop::TestCaseContext) -> (String, i64) {
    let radix: u32 = noprop::sample_usize_in(ctx, 2..=36) as u32;
    let v = (noprop::sample_u64(ctx) & i64::MAX as u64) as i64;
    let digits = to_radix_digits(v as u64, radix);
    (format!("{radix}#{digits}"), v)
}

/// Sample a decimal float in `[0.0, 1e6]`. Returns `(text, value)`.
///
/// The text is intentionally the Rust `Display` form so the generator
/// does not exercise the same fractional decoding as the tokenizer; the
/// tokenizer's decoded value must simply equal the Rust `Display`
/// round-trip value.
pub fn sample_decimal_float(ctx: &mut noprop::TestCaseContext) -> (String, f64) {
    // Choose a small integer numerator/denominator to keep the printed
    // form short and exactly representable across a round-trip.
    const FIXED: [f64; 8] = [0.0, 0.5, 1.25, 1.0, 100.5, 2.5, 3.75, 1234.5];
    let v = noprop::sample_choice(ctx, &FIXED);
    let mut text = format!("{v}");
    if !text.contains('.') {
        text.push_str(".0");
    }
    (text, v)
}

/// Sample a keyword text.
pub fn sample_keyword(ctx: &mut noprop::TestCaseContext) -> (&'static str, Keyword) {
    noprop::sample_choice(ctx, &KEYWORDS)
}

/// Sample a symbol text.
pub fn sample_symbol(ctx: &mut noprop::TestCaseContext) -> &'static str {
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

/// Sample a sigil string literal. Returns
/// `(text, prefix, decoded_content, suffix)`.
pub fn sample_sigil_string(ctx: &mut noprop::TestCaseContext) -> (String, String, String, String) {
    // Delimiters and their content-safe alphabets. For content we avoid
    // both delimiter characters and backslash entirely so no escaping
    // logic is exercised in the value oracle (escape coverage is done
    // separately by the atom / string generators).
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
    const AFFIX: [char; 6] = ['a', 'b', 'x', '_', '1', 'Q'];
    const CONTENT: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'x', 'y', 'z'];

    let (open, close) = noprop::sample_choice(ctx, &DELIMS);
    let prefix_len = noprop::sample_usize_in(ctx, 0..=3);
    let suffix_len = noprop::sample_usize_in(ctx, 0..=3);
    let content_len = noprop::sample_usize_in(ctx, 0..=8);

    let mut prefix = String::new();
    for _ in 0..prefix_len {
        prefix.push(noprop::sample_choice(ctx, &AFFIX));
    }
    let mut content = String::new();
    for _ in 0..content_len {
        content.push(noprop::sample_choice(ctx, &CONTENT));
    }
    let mut suffix = String::new();
    for _ in 0..suffix_len {
        suffix.push(noprop::sample_choice(ctx, &AFFIX));
    }

    let text = format!("~{prefix}{open}{content}{close}{suffix}");
    (text, prefix, content, suffix)
}

/// Sample a whitespace sequence using a weighted mix of ASCII/NBSP
/// whitespace characters.
pub fn sample_whitespace_sequence(ctx: &mut noprop::TestCaseContext) -> String {
    let len =
        noprop::sample_with_boundaries(ctx, &[0usize, 1, 16], noprop::Ratio::one_nth(5), |ctx| {
            noprop::sample_usize_in(ctx, 0..=16)
        });
    let mut s = String::new();
    for _ in 0..len {
        // Weighted: LF is rarer than horizontal whitespace so a token
        // typically pulls in trailing non-LF whitespace.
        match noprop::sample_usize_in(ctx, 0..10) {
            0..=3 => s.push(' '),
            4..=6 => s.push('\t'),
            7 => s.push('\r'),
            8 => s.push('\n'),
            _ => s.push('\u{a0}'),
        }
    }
    s
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
/// their terminating newline). Otherwise any of `WS_CHARS` is safe: the
/// oracle only needs the concatenation of scanned token texts to equal
/// the source, and whitespace runs simply aggregate.
pub fn insert_separator(ctx: &mut noprop::TestCaseContext, prev: &str) -> char {
    if prev.starts_with('%') {
        return '\n';
    }
    noprop::sample_choice(ctx, &WS_CHARS)
}
