//! Integration tests covering the public `scan_token` API.
//!
//! Tests are organised by token kind, then contract areas (position
//! tracking, error recovery, iteration, `Token::value` semantics). Every
//! test drives the tokenizer through the free function `scan_token` so
//! any regression in the public surface surfaces here.

use std::borrow::Cow;

use erl_tokenize::{
    ErrorKind, Keyword, Position, Symbol, Token, TokenKind, TokenValue, scan_token,
};

// ============================================================
// Helpers
// ============================================================

fn pos() -> Position {
    Position::new()
}

/// Scan `src` fully and return the emitted tokens.
fn scan_tokens(src: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut p = Position::new();
    while let Some(t) = scan_token(src, p).unwrap() {
        p = t.end();
        out.push(t);
    }
    out
}

/// Scan `src` fully and return the sequence of token texts.
fn texts(src: &str) -> Vec<&str> {
    scan_tokens(src).into_iter().map(|t| t.text(src)).collect()
}

/// Scan the first token of `src` (panics if there is none).
fn first(src: &str) -> Token {
    scan_token(src, pos()).unwrap().expect("at least one token")
}

/// Scan the first token and return its decoded value.
fn first_value(src: &str) -> TokenValue<'_> {
    first(src).value(src)
}

// ============================================================
// Atom
// ============================================================

#[test]
fn atom_bare_and_unicode() {
    for src in [
        "foo",
        "hello_world",
        "foo@bar",
        "a123",
        "comté",
        "äfunc",
        "ärlig",
    ] {
        let t = first(src);
        assert_eq!(t.kind(), TokenKind::Atom);
        assert_eq!(t.text(src), src);
        match t.value(src) {
            TokenValue::Atom(Cow::Borrowed(v)) => assert_eq!(v, src),
            other => panic!("expected bare atom Borrowed for {src:?}, got {other:?}"),
        }
    }
}

#[test]
fn atom_quoted_borrowed_when_no_escape() {
    for (src, expected) in [
        ("'foo'", "foo"),
        ("'Foo'", "Foo"),
        ("'hello world'", "hello world"),
        ("''", ""),
    ] {
        assert_eq!(first(src).kind(), TokenKind::Atom);
        match first_value(src) {
            TokenValue::Atom(Cow::Borrowed(v)) => assert_eq!(v, expected),
            other => panic!("expected quoted atom Borrowed for {src:?}, got {other:?}"),
        }
    }
}

#[test]
fn atom_quoted_owned_on_escape() {
    for (src, expected) in [
        (r"'f\x6Fo'", "foo"),
        (r"'a\nb'", "a\nb"),
        (r"'a\\b'", "a\\b"),
        (r"'a\'b'", "a'b"),
    ] {
        match first_value(src) {
            TokenValue::Atom(Cow::Owned(v)) => assert_eq!(v, expected),
            other => panic!("expected quoted atom Owned for {src:?}, got {other:?}"),
        }
    }
}

#[test]
fn atom_vs_keyword_dispatch() {
    assert!(matches!(
        first("case").kind(),
        TokenKind::Keyword(Keyword::Case)
    ));
    assert_eq!(first("case_x").kind(), TokenKind::Atom);
    assert_eq!(first("foo").kind(), TokenKind::Atom);
}

#[test]
fn atom_errors_or_dispatch_elsewhere() {
    // Leading whitespace: the first token is a whitespace token, not an
    // atom error.
    assert_eq!(first("  foo").kind(), TokenKind::Whitespace);
    // Digits open an integer, not an atom.
    assert_eq!(first("123").kind(), TokenKind::Integer);
    // Empty input has no token.
    assert_eq!(scan_token("", pos()).unwrap(), None);
    // Non-lowercase alphabetic that is not a variable head (a Chinese
    // ideograph) fails as an atom.
    assert!(scan_token("中", pos()).is_err());
    // Non Latin-1 alphabetic characters are not accepted in atom head or
    // body, matching erl_scan's `illegal character`.
    //
    // A non-Latin-1 head fails on the first token. A non-Latin-1 letter in
    // the body terminates the atom at the ASCII prefix, so the offending
    // character opens the next token, which then fails.
    assert_eq!(
        scan_token("μfoo", pos()).unwrap_err().kind(),
        ErrorKind::InvalidAtomToken
    );
    for src in ["fooαbar", "foo中bar"] {
        // The ASCII prefix scans as a bare atom...
        assert_eq!(first(src).kind(), TokenKind::Atom);
        assert_eq!(first(src).text(src), "foo");
        // ...and the next token (opened by the non-Latin-1 head) fails.
        let p = first(src).end();
        assert_eq!(
            scan_token(src, p).unwrap_err().kind(),
            ErrorKind::InvalidAtomToken,
            "for {src:?}"
        );
    }
    // Latin-1 letters are still accepted in atom head and body.
    assert_eq!(first("äfunc").kind(), TokenKind::Atom);
    assert_eq!(first("comté").kind(), TokenKind::Atom);
    assert_eq!(first("ärlig").kind(), TokenKind::Atom);
}

// ============================================================
// Character
// ============================================================

#[test]
fn char_simple() {
    for (src, expected) in [("$a", 'a'), ("$Z", 'Z'), ("$0", '0'), ("$ ", ' ')] {
        let t = first(src);
        assert_eq!(t.kind(), TokenKind::Char);
        assert_eq!(t.text(src), src);
        assert_eq!(t.value(src), TokenValue::Char(expected));
    }
}

#[test]
fn char_named_escapes() {
    for (src, expected) in [
        (r"$\b", 8_u32),
        (r"$\d", 127),
        (r"$\e", 27),
        (r"$\f", 12),
        (r"$\n", '\n' as u32),
        (r"$\r", '\r' as u32),
        (r"$\s", ' ' as u32),
        (r"$\t", '\t' as u32),
        (r"$\v", 11),
    ] {
        match first_value(src) {
            TokenValue::Char(c) => assert_eq!(c as u32, expected, "for {src}"),
            other => panic!("expected Char for {src}, got {other:?}"),
        }
    }
}

#[test]
fn char_caret_notation_full_alphabet() {
    // Symbols explicitly accepted after `\^`.
    for (c, expected) in [
        ('@', 0x00_u32),
        ('[', 0x1b),
        ('\\', 0x1c),
        (']', 0x1d),
        ('^', 0x1e),
        ('_', 0x1f),
        ('?', 0x7f),
    ] {
        let src = format!(r"$\^{c}");
        assert_eq!(
            first_value(&src),
            TokenValue::Char(char::from_u32(expected).unwrap())
        );
    }
    // Every letter follows the `c % 32` rule.
    for c in ('a'..='z').chain('A'..='Z') {
        let src = format!(r"$\^{c}");
        let expected = char::from_u32((c as u32) % 32).unwrap();
        assert_eq!(first_value(&src), TokenValue::Char(expected));
    }
}

#[test]
fn char_caret_notation_invalid() {
    for src in [r"$\^!", r"$\^0", r"$\^あ", r"$\^>", r"$\^`", r"$\^{"] {
        assert!(scan_token(src, pos()).is_err(), "expected error for {src}");
    }
}

#[test]
fn char_hex_forms() {
    assert_eq!(first_value(r"$\x6F"), TokenValue::Char('o'));
    assert_eq!(first_value(r"$\x41"), TokenValue::Char('A'));
    assert_eq!(first_value(r"$\x{06F}"), TokenValue::Char('o'));
    assert_eq!(first_value(r"$\x{10FFFF}"), TokenValue::Char('\u{10FFFF}'));
}

#[test]
fn char_hex_invalid() {
    for src in [
        r"$\x{ab",      // unterminated brace
        r"$\x{}",       // empty braces
        r"$\x{",        // EOF after `{`
        r"$\x",         // EOF after `\x`
        r"$\x6",        // fixed-width needs two digits
        r"$\x{zz}",     // non-hex digits
        r"$\x{110000}", // beyond U+10FFFF
        r"$\x{D800}",   // surrogate
    ] {
        assert!(scan_token(src, pos()).is_err(), "expected error for {src}");
    }
}

#[test]
fn char_octal() {
    assert_eq!(first_value(r"$\123"), TokenValue::Char('S'));
    assert_eq!(first_value(r"$\17"), TokenValue::Char('\u{f}'));
    assert_eq!(first_value(r"$\01"), TokenValue::Char('\u{1}'));
    assert_eq!(first_value(r"$\0"), TokenValue::Char('\0'));
    assert_eq!(first_value(r"$\7"), TokenValue::Char('\u{7}'));
    assert_eq!(first_value(r"$\377"), TokenValue::Char('\u{ff}'));
    // Octal stops at three digits.
    assert_eq!(texts(r"$\1234"), [r"$\123", "4"]);
}

#[test]
fn char_errors() {
    for src in ["$", r"$\"] {
        assert!(scan_token(src, pos()).is_err(), "expected error for {src}");
    }
}

// ============================================================
// Comment
// ============================================================

#[test]
fn comment_basic_and_value() {
    let t = first("%");
    assert_eq!(t.kind(), TokenKind::Comment);
    assert_eq!(t.text("%"), "%");
    assert_eq!(t.value("%"), TokenValue::Comment(""));

    let src = "%% foo ";
    let t = first(src);
    assert_eq!(t.text(src), src);
    assert_eq!(t.value(src), TokenValue::Comment("% foo "));
}

#[test]
fn comment_stops_at_lf_and_hands_off_to_whitespace() {
    let src = "% comment\n  foo";
    // The trailing LF is not part of the comment; it heads the next
    // whitespace token.
    assert_eq!(texts(src), ["% comment", "\n  ", "foo"]);
}

// ============================================================
// Integer
// ============================================================

#[test]
fn integer_basic_and_underscores() {
    for (src, expected) in [
        ("0", 0_u64),
        ("42", 42),
        ("123456789", 123456789),
        ("123_456", 123456),
        ("123_456_789", 123456789),
    ] {
        assert_eq!(first(src).kind(), TokenKind::Integer);
        assert_eq!(first_value(src), TokenValue::Integer(Some(expected)));
    }
}

#[test]
fn integer_radix() {
    for (src, expected) in [
        ("2#101", 0b101_u64),
        ("8#777", 0o777),
        ("16#ab0e", 0xab0e),
        ("1_6#a_b_0e", 0xab0e),
    ] {
        assert_eq!(first_value(src), TokenValue::Integer(Some(expected)));
    }
}

#[test]
fn integer_overflow_is_none() {
    // i64::MAX still fits in u64.
    assert_eq!(
        first_value("9223372036854775807"),
        TokenValue::Integer(Some(i64::MAX as u64))
    );
    // i64::MAX + 1 fits in u64.
    assert_eq!(
        first_value("9223372036854775808"),
        TokenValue::Integer(Some(1u64 << 63))
    );
    // u64::MAX is the last representable value.
    assert_eq!(
        first_value("18446744073709551615"),
        TokenValue::Integer(Some(u64::MAX))
    );
    // Just past u64::MAX overflows to None.
    assert_eq!(
        first_value("18446744073709551616"),
        TokenValue::Integer(None)
    );
    assert_eq!(
        first_value("16#ffffffffffffffffff"),
        TokenValue::Integer(None)
    );
}

#[test]
fn integer_errors() {
    // Leading `-` is a symbol.
    assert_eq!(first("-10").kind(), TokenKind::Symbol(Symbol::Hyphen));
    for src in ["123_456_", "123__456", "1#0", "37#0"] {
        assert!(scan_token(src, pos()).is_err(), "expected error for {src}");
    }
}

// ============================================================
// Float
// ============================================================

#[test]
fn float_decimal() {
    for (src, expected) in [
        ("0.1", 0.1_f64),
        ("1.5", 1.5),
        ("12.3e-1", 1.23),
        ("12.3E-1", 1.23),
        ("12.3e+1", 123.0),
    ] {
        assert_eq!(first_value(src), TokenValue::Float(expected), "for {src}");
    }
}

#[test]
fn float_underscores() {
    assert_eq!(first_value("1_2.3_4"), TokenValue::Float(12.34));
    assert_eq!(
        first_value("1_2.3_4e-1_0"),
        TokenValue::Float(0.000000001234)
    );
}

#[test]
fn float_radix() {
    for (src, expected) in [
        ("2#0.111", 0.875_f64),
        ("16#f_f.F_F", 255.99609375),
        ("2#0.10101#e8", 168.0),
        ("1_6#fefe.fefe#e1_6", 1.2041849337671418e24),
        ("16#a.b#E10", 11751030521856.0),
        ("2#0.0#e+5", 0.0),
        ("2#1.0#E-3", 0.125),
    ] {
        assert_eq!(first_value(src), TokenValue::Float(expected), "for {src}");
    }
}

#[test]
fn float_errors() {
    // These shapes reach `scan_float` (because the dispatcher recognises
    // them as float-like on lookahead) and then fail.
    for src in ["12_.3", "12.3_", "1__2.3", "12.3__4", "12.34e-1__0"] {
        assert!(scan_token(src, pos()).is_err(), "expected error for {src}");
    }
    // These shapes do not look float-like to the dispatcher, so they
    // tokenize as their component tokens rather than erroring:
    // `1.` becomes Integer + Dot; `.123` becomes Dot + Integer; `123`
    // alone is an integer.
    assert_eq!(texts("1."), ["1", "."], "`1.` splits into integer and dot");
    assert_eq!(
        texts(".123"),
        [".", "123"],
        "`.123` splits into dot and integer"
    );
    assert_eq!(first("123").kind(), TokenKind::Integer);
}

#[test]
fn float_overflow_is_error() {
    // Matches `erl_scan`: a decoded value outside the finite f64 range
    // is rejected with the same InvalidFloatToken as syntactic errors.
    for src in ["1.8e308", "1.0e400", "2#1.0#e10000"] {
        assert!(
            scan_token(src, pos()).is_err(),
            "expected overflow error for {src}"
        );
    }
    // An exponent that overflows `i32` must not panic; the resulting
    // non-finite magnitude is rejected like any other float overflow.
    assert!(
        scan_token("2#0.0#e10000000000", pos()).is_err(),
        "expected error (not panic) for exponent beyond i32"
    );
    // Boundary exponents at i32::MAX and just past it must not panic and
    // must return either `Ok` or `Err` (both saturate to non-finite).
    for src in ["2#1.0#e2147483647", "2#1.0#e2147483648"] {
        let _ = scan_token(src, pos());
    }
    // The last representable magnitude scans successfully.
    assert_eq!(
        first_value("1.7e308"),
        TokenValue::Float(1.7e308),
        "1.7e308 should still be scannable"
    );
}

#[test]
fn float_underflow_is_zero() {
    // Matches `erl_scan`: an exponent that underflows collapses to 0.0.
    assert_eq!(first_value("1.0e-400"), TokenValue::Float(0.0));
    assert_eq!(first_value("1.0e-500"), TokenValue::Float(0.0));
}

// ============================================================
// Keyword
// ============================================================

#[test]
fn keyword_all_reserved_words() {
    for (src, expected) in [
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
    ] {
        let t = first(src);
        assert_eq!(t.kind(), TokenKind::Keyword(expected), "for {src}");
        assert_eq!(t.value(src), TokenValue::Keyword(expected));
        assert_eq!(t.text(src), src);
    }
}

// ============================================================
// Sigil string
// ============================================================

#[test]
fn sigil_string_all_delimiters_borrow_content() {
    for (src, expected_content, expected_suffix) in [
        (r#"~"foo""#, "foo", ""),
        ("~(foo)", "foo", ""),
        ("~[bar]qq", "bar", "qq"),
        ("~a{baz}", "baz", ""),
        ("~<qux>", "qux", ""),
        ("~/quux/", "quux", ""),
        ("~|corge|", "corge", ""),
        ("~'grault'", "grault", ""),
        ("~`garply`", "garply", ""),
        ("~#waldo#", "waldo", ""),
    ] {
        let t = first(src);
        assert_eq!(t.kind(), TokenKind::SigilString);
        assert_eq!(t.text(src), src);
        match t.value(src) {
            TokenValue::SigilString {
                prefix: _,
                content,
                suffix,
            } => {
                assert_eq!(content.as_ref(), expected_content, "content for {src}");
                assert_eq!(suffix, expected_suffix, "suffix for {src}");
                assert!(
                    matches!(content, Cow::Borrowed(_)),
                    "borrowed content for {src}"
                );
            }
            other => panic!("expected SigilString for {src}, got {other:?}"),
        }
    }
}

#[test]
fn sigil_string_prefix_suffix() {
    let src = r#"~b"foo"qq"#;
    match first_value(src) {
        TokenValue::SigilString {
            prefix,
            content,
            suffix,
        } => {
            assert_eq!(prefix, "b");
            assert_eq!(content.as_ref(), "foo");
            assert_eq!(suffix, "qq");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn sigil_string_escape_produces_owned_content() {
    match first_value(r#"~(a\nb)"#) {
        TokenValue::SigilString {
            content: Cow::Owned(s),
            ..
        } => assert_eq!(s, "a\nb"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn sigil_string_errors() {
    for src in ["~", "~?foo?", "~(foo"] {
        assert!(scan_token(src, pos()).is_err(), "expected error for {src}");
    }
    // Non-sigil string.
    assert_eq!(first(r#""foo""#).kind(), TokenKind::String);
}

#[test]
fn sigil_string_verbatim_prefix_preserves_backslash() {
    // Uppercase prefix (`~B`) is a verbatim binary sigil: `\` is a
    // literal character, not an escape introducer, so `~B"\"` is a valid
    // one-character content ending at the second `"`.
    let src = r#"~B"\""#;
    match first_value(src) {
        TokenValue::SigilString {
            prefix,
            content: Cow::Borrowed(s),
            suffix,
        } => {
            assert_eq!(prefix, "B");
            assert_eq!(s, "\\");
            assert_eq!(suffix, "");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn sigil_string_verbatim_non_string_delim() {
    // Verbatim rule applies to non-string delimiters too: `~R(\)` scans
    // as content `\`, not an ill-formed escape.
    match first_value(r#"~R(\)"#) {
        TokenValue::SigilString {
            prefix,
            content: Cow::Borrowed(s),
            suffix,
        } => {
            assert_eq!(prefix, "R");
            assert_eq!(s, "\\");
            assert_eq!(suffix, "");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn sigil_string_lowercase_prefix_processes_escapes() {
    // `~s` and `~b` (and empty prefix) still process escapes.
    match first_value(r#"~s"a\nb""#) {
        TokenValue::SigilString {
            prefix,
            content: Cow::Owned(s),
            ..
        } => {
            assert_eq!(prefix, "s");
            assert_eq!(s, "a\nb");
        }
        other => panic!("{other:?}"),
    }
    match first_value(r#"~b"a\nb""#) {
        TokenValue::SigilString {
            prefix,
            content: Cow::Owned(s),
            ..
        } => {
            assert_eq!(prefix, "b");
            assert_eq!(s, "a\nb");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn sigil_string_empty_content_followed_by_semicolon() {
    // `~"";` in the middle of a case clause used to be rejected as an
    // adjacent string literal; the sigil suffix scanner should treat the
    // `;` as a separate token, not a repeated string opener.
    let tokens = scan_tokens("~\"\";");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind(), TokenKind::SigilString);
    assert_eq!(tokens[1].kind(), TokenKind::Symbol(Symbol::Semicolon));
}

#[test]
fn sigil_string_empty_content_followed_by_string_rejects() {
    // A sigil with empty suffix followed immediately by `"` is still
    // adjacent-string, matching `scan_string_concat` in `erl_scan`.
    assert!(scan_token(r#"~""""foo""#, pos()).is_err());
}

#[test]
fn sigil_string_triple_quoted_non_verbatim_decodes_escapes() {
    // Non-verbatim prefixes (`~s`, `~b`) decode escapes inside
    // triple-quoted content, matching erl_scan. The body line is the escape
    // `\n` (backslash-n) which decodes to a single LF.
    for (src, expected) in [
        (
            r#"~s"""
\n
""""#
                .to_string(),
            "\n",
        ),
        (
            r#"~b"""
\n
""""#
                .to_string(),
            "\n",
        ),
        (
            r#"~s"""
\x{41}
""""#
                .to_string(),
            "A",
        ),
        (
            r#"~s"""
\tfoo
""""#
                .to_string(),
            "\tfoo",
        ),
    ] {
        match first_value(&src) {
            TokenValue::SigilString {
                content: Cow::Owned(s),
                ..
            } => assert_eq!(s, expected, "for {src:?}"),
            other => panic!("{other:?} for {src:?}"),
        }
    }
}

#[test]
fn sigil_string_triple_quoted_empty_prefix_is_verbatim() {
    // The empty prefix `~"""..."""` is verbatim in erl_scan (only `s`/`b`
    // are non-verbatim in the triple-quoted form), so `\n` stays literal.
    let src = r#"~"""
\n
""""#;
    match first_value(src) {
        TokenValue::SigilString {
            content: Cow::Borrowed(s),
            ..
        } => assert_eq!(s, "\\n"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn sigil_string_triple_quoted_verbatim_preserves_backslash() {
    // Verbatim prefixes (`~S`, `~B`) and the plain triple-quoted string
    // keep `\` literal inside triple-quoted content.
    let src = r#"~S"""
\n
""""#;
    match first_value(src) {
        TokenValue::SigilString {
            content: Cow::Borrowed(s),
            ..
        } => assert_eq!(s, "\\n"),
        other => panic!("{other:?}"),
    }
    let src = "\"\"\"\n\\n\n\"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Borrowed(s)) => assert_eq!(s, "\\n"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn sigil_string_triple_quoted_non_verbatim_malformed_escape_rejects() {
    // erl_scan rejects a malformed escape in non-verbatim triple-quoted
    // sigil content; we must too (the decode step would otherwise panic on
    // its `scanner already validated escape` expect). `\x` with no hex
    // digits, an empty `\x{}`, and an unclosed `\x{` are all illegal.
    for src in [
        "~s\"\"\"\n\\x\n\"\"\"",
        "~s\"\"\"\n\\x{}\n\"\"\"",
        "~s\"\"\"\n\\x{41\n\"\"\"",
        "~b\"\"\"\n\\x\n\"\"\"",
    ] {
        assert!(
            scan_token(src, pos()).is_err(),
            "expected error for {src:?}"
        );
    }
}

#[test]
fn sigil_string_triple_quoted_non_verbatim_line_continuation() {
    // erl_scan treats a `\` immediately before the raw LF that ends a
    // content line as a line-continuation marker: the backslash is
    // consumed and the LF still acts as the content-line separator, so
    // `foo\<LF>bar` decodes to `"foo\nbar"` (not an escape error).
    // Verify each observable rule:
    //   * lone `\<LF>` at the end of a line just drops the `\`;
    //   * `\\` (an escaped backslash) still decodes to a literal `\`;
    //   * a trailing odd count of `\` drops one continuation `\`
    //     after the preceding `\\` escapes decode.
    for (src, expected) in [
        ("~s\"\"\"\nfoo\\\nbar\n\"\"\"", "foo\nbar"),
        ("~s\"\"\"\nabc\\\n\"\"\"", "abc"),
        ("~s\"\"\"\n\\\n\\\n\"\"\"", "\n"),
        ("~s\"\"\"\na\\\nb\\\nc\n\"\"\"", "a\nb\nc"),
        ("~s\"\"\"\n\\\\\n\"\"\"", "\\"),
        ("~s\"\"\"\n\\\\\\\n\"\"\"", "\\"),
        ("~s\"\"\"\n\\\\\\\\\n\"\"\"", "\\\\"),
        ("~s\"\"\"\n  foo\\\n  bar\n  \"\"\"", "foo\nbar"),
        ("~s\"\"\"\n  \\\n  \"\"\"", ""),
    ] {
        match first_value(src) {
            TokenValue::SigilString {
                content: Cow::Owned(s),
                ..
            } => assert_eq!(s, expected, "for {src:?}"),
            other => panic!("{other:?} for {src:?}"),
        }
    }
}

#[test]
fn sigil_string_triple_quoted_verbatim_preserves_trailing_backslash() {
    // Verbatim triple-quoted content keeps `\` literal, including a
    // trailing `\` before the LF that ends a content line (no line
    // continuation).
    let src = "~S\"\"\"\nfoo\\\nbar\n\"\"\"";
    match first_value(src) {
        TokenValue::SigilString {
            content: Cow::Borrowed(s),
            ..
        } => assert_eq!(s, "foo\\\nbar"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn sigil_string_suffix_separates_from_next_string() {
    // A non-empty sigil suffix separates the tokens, so a `"..."` may
    // follow without triggering adjacent-string.
    let src = r#"~"foo"s"bar""#;
    let tokens = scan_tokens(src);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind(), TokenKind::SigilString);
    assert_eq!(tokens[1].kind(), TokenKind::String);
}

// ============================================================
// String
// ============================================================

#[test]
fn string_regular_borrow_and_own() {
    match first_value(r#""hello""#) {
        TokenValue::String(Cow::Borrowed(s)) => assert_eq!(s, "hello"),
        other => panic!("{other:?}"),
    }
    match first_value(r#""a\nb""#) {
        TokenValue::String(Cow::Owned(s)) => assert_eq!(s, "a\nb"),
        other => panic!("{other:?}"),
    }
    match first_value(r#""f\x6Fo""#) {
        TokenValue::String(Cow::Owned(s)) => assert_eq!(s, "foo"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_no_indent_borrowed() {
    let src = "\"\"\"\nfoo\n\"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Borrowed(s)) => assert_eq!(s, "foo"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_multiline_borrowed() {
    let src = "\"\"\"\nline1\nline2\nline3\n\"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Borrowed(s)) => assert_eq!(s, "line1\nline2\nline3"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_indented_owned() {
    let src = "\"\"\"\n  hello\n  world\n  \"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Owned(s)) => assert_eq!(s, "hello\nworld"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_empty() {
    let src = "\"\"\"\n\"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Borrowed(s)) => assert_eq!(s, ""),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_empty_indented() {
    let src = "\"\"\"\n  \"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Owned(s)) => assert_eq!(s, ""),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_with_blank_lines_borrowed() {
    let src = "\"\"\"\nfoo\n\nbar\n\"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Borrowed(s)) => assert_eq!(s, "foo\n\nbar"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_with_blank_line_indented() {
    let src = "\"\"\"\n  foo\n\n  bar\n  \"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Owned(s)) => assert_eq!(s, "foo\n\nbar"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_crlf_cases() {
    for (src, expected) in [
        ("\"\"\"\n hello\r\n world\r\n \"\"\"", "hello\r\nworld"),
        ("\"\"\"\n  a\r\n  b\r\n  c\r\n  \"\"\"", "a\r\nb\r\nc"),
        ("\"\"\"\n  a\n  \r\n  \"\"\"", "a\n"),
        ("\"\"\"\n \r\n \"\"\"", ""),
        ("\"\"\"\nabc\r\r\n\"\"\"", "abc\r"),
    ] {
        assert_eq!(
            first_value(src),
            TokenValue::String(Cow::Borrowed(expected)),
            "for {src:?}"
        );
    }
}

#[test]
fn string_adjacent_literals_reject() {
    assert!(scan_token(r#""foo""bar""#, pos()).is_err());
}

#[test]
fn string_triple_quoted_four_quote_closer() {
    // An opener of N quotes must be matched by exactly N quotes on the
    // closer line. `""""` opens a 4-quote string; a 3-quote body line is
    // ordinary content.
    let src = "\"\"\"\"\n  \"\"\"\n  body\n  \"\"\"\"";
    match first_value(src) {
        TokenValue::String(Cow::Owned(s)) => assert_eq!(s, "\"\"\"\nbody"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn string_triple_quoted_closer_requires_contiguous_quotes() {
    // For a 4-quote opener `""""`, a body line of `""" ""` (3 quotes,
    // space, 2 quotes) is not a valid closer: the closer must be a
    // contiguous run of exactly N `"` on a line with only leading
    // whitespace. The real closer is the `""""` on the last line.
    // This is the shape used by OTP's `sigils_SUITE.erl` line 249.
    let src = "\"\"\"\"\n  \"\"\" \"\"\n  \"\"\"\"";
    let t = first(src);
    assert_eq!(t.kind(), TokenKind::String);
    assert_eq!(t.text(src), src);
}

#[test]
fn string_errors() {
    for src in ["\"", r#""unterminated"#] {
        assert!(
            scan_token(src, pos()).is_err(),
            "expected error for {src:?}"
        );
    }
}

#[test]
fn string_escape_error_position_tracks_line_breaks() {
    // An ill-formed escape reports the position of the backslash that
    // opens it, tracking line/column across embedded LF so multi-line
    // sources report the real location (erl_scan behaviour).
    let src = "\"line1\n\\^0\"";
    let err = scan_token(src, pos()).unwrap_err();
    assert_eq!(
        (err.position().line().get(), err.position().column().get()),
        (2, 1),
        "for {src:?}"
    );

    let src = "\"'\n\n\\^0'\"";
    let err = scan_token(src, pos()).unwrap_err();
    assert_eq!(
        (err.position().line().get(), err.position().column().get()),
        (3, 1),
        "for {src:?}"
    );

    // Single-line escapes are unchanged.
    let src = r#""ab\^0""#;
    let err = scan_token(src, pos()).unwrap_err();
    assert_eq!(
        (err.position().line().get(), err.position().column().get()),
        (1, 3),
        "for {src:?}"
    );
}

// ============================================================
// Symbol
// ============================================================

#[test]
fn symbol_all() {
    for (src, expected) in [
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
    ] {
        let t = first(src);
        assert_eq!(t.kind(), TokenKind::Symbol(expected), "for {src}");
        assert_eq!(t.text(src), src);
    }
}

#[test]
fn symbol_wildcard_record_single_token() {
    // `#_` is a single wildcard-record symbol, not `#` followed by `_`.
    let tokens = scan_tokens("Node#_{anno=[]}");
    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind()).collect();
    assert_eq!(
        kinds,
        [
            TokenKind::Variable,
            TokenKind::Symbol(Symbol::WildcardRecord),
            TokenKind::Symbol(Symbol::OpenBrace),
            TokenKind::Atom,
            TokenKind::Symbol(Symbol::Match),
            TokenKind::Symbol(Symbol::OpenSquare),
            TokenKind::Symbol(Symbol::CloseSquare),
            TokenKind::Symbol(Symbol::CloseBrace),
        ]
    );
    assert_eq!(tokens[1].text("Node#_{anno=[]}"), "#_");
    assert_eq!(
        tokens[1].value("Node#_{anno=[]}"),
        TokenValue::Symbol(Symbol::WildcardRecord)
    );
}

#[test]
fn double_question_splits_into_two_question_tokens() {
    // `??` is not a single symbol; erl_scan emits it as two `?` tokens
    // (the preprocessor combines them, e.g. as the stringify-arg operator).
    assert_eq!(texts("??"), ["?", "?"]);
    assert_eq!(texts("??foo"), ["?", "?", "foo"]);
    // A lone `?` is unchanged.
    assert_eq!(texts("?"), ["?"]);
    assert_eq!(first("?").kind(), TokenKind::Symbol(Symbol::Question));
}

// ============================================================
// Variable
// ============================================================

#[test]
fn variable_basic_and_borrow() {
    for src in ["Foo", "_", "_foo", "Foo_1", "Foo_1@bar"] {
        let t = first(src);
        assert_eq!(t.kind(), TokenKind::Variable);
        assert_eq!(t.text(src), src);
        assert_eq!(t.value(src), TokenValue::Variable(src));
    }
}

#[test]
fn variable_latin1_uppercase_head() {
    // Latin-1 uppercase letters (`À..Þ` minus `×`) are valid variable
    // heads in `erl_scan`. The `compile_SUITE_data/small.erl` fixture in
    // OTP uses `Överskott` as a parameter name.
    for src in ["Överskott", "Ärger", "Ünique"] {
        let t = first(src);
        assert_eq!(t.kind(), TokenKind::Variable, "for {src}");
        assert_eq!(t.text(src), src);
    }
}

// ============================================================
// Whitespace (erl_scan return_white_spaces rules)
// ============================================================

#[test]
fn whitespace_aggregation_table() {
    // Boundary cases: non-LF runs aggregate; LF starts a new token; each
    // token holds at most one LF and only at the start; CR is not a line
    // break.
    for (src, expected) in [
        ("   \t", ["   \t"].as_slice()),
        (" \t\n", &[" \t", "\n"]),
        ("\n \t", &["\n \t"]),
        ("\n\n", &["\n", "\n"]),
        ("\n \n\t", &["\n ", "\n\t"]),
        ("\r\n ", &["\r", "\n "]),
        ("\u{A0} \t", &["\u{A0} \t"]),
    ] {
        assert_eq!(texts(src), *expected, "aggregation for {src:?}");
    }
}

#[test]
fn whitespace_at_most_one_lf_at_start() {
    let src = "a  \n  b\n\nc";
    for t in scan_tokens(src) {
        if t.kind() != TokenKind::Whitespace {
            continue;
        }
        let text = t.text(src);
        let lfs = text.matches('\n').count();
        assert!(lfs <= 1, "unexpected LF count in {text:?}");
        if lfs == 1 {
            assert!(text.starts_with('\n'), "LF not at start: {text:?}");
        }
    }
}

#[test]
fn whitespace_boundaries_around_other_tokens() {
    assert_eq!(texts("foo\t\t bar"), ["foo", "\t\t ", "bar"]);
    assert_eq!(texts("1 . 2"), ["1", " ", ".", " ", "2"]);
}

#[test]
fn whitespace_value_borrows_aggregated_text() {
    let src = " \t \n\t";
    let t = first(src);
    match t.value(src) {
        TokenValue::Whitespace(s) => {
            assert_eq!(s, " \t ");
            assert_eq!(s.as_ptr(), src.as_ptr());
        }
        other => panic!("{other:?}"),
    }
}

// ============================================================
// Position tracking
// ============================================================

#[test]
fn position_after_first_token_advances_column() {
    let src = "foo bar";
    let mut p = Position::new();
    assert_eq!(p.offset(), 0);
    let t = scan_token(src, p).unwrap().unwrap();
    p = t.end();
    assert_eq!(p.offset(), 3);
    assert_eq!(p.column().get(), 4);
}

#[test]
fn position_advances_across_lf_whitespace() {
    let src = "  \n \tX";
    let mut p = Position::new();
    let t1 = scan_token(src, p).unwrap().unwrap();
    p = t1.end();
    assert_eq!((p.line().get(), p.column().get()), (1, 3));
    let t2 = scan_token(src, p).unwrap().unwrap();
    p = t2.end();
    assert_eq!((p.line().get(), p.column().get()), (2, 3));
    let t3 = scan_token(src, p).unwrap().unwrap();
    p = t3.end();
    assert_eq!((p.line().get(), p.column().get()), (2, 4));
    assert_eq!(t3.text(src), "X");
}

// ============================================================
// scan_token / Token contract
// ============================================================

#[test]
fn scan_token_returns_none_at_eof() {
    assert_eq!(scan_token("", Position::new()).unwrap(), None);
    let src = "foo";
    let t = scan_token(src, Position::new()).unwrap().unwrap();
    assert_eq!(scan_token(src, t.end()).unwrap(), None);
}

#[test]
fn scan_token_walks_source() {
    let src = "io:format(\"Hello\").";
    assert_eq!(
        texts(src),
        ["io", ":", "format", "(", "\"Hello\"", ")", "."]
    );
}

#[test]
fn scan_token_texts_reconstruct_source() {
    let src = "a  \n\tb\n\n c% comment\n";
    let concat: String = texts(src).concat();
    assert_eq!(concat, src);
}

#[test]
fn hidden_filter_matches_lexical_only() {
    let src = "foo  \n  bar % tail\n baz";
    let lex: Vec<_> = scan_tokens(src)
        .into_iter()
        .filter(|t| t.kind().is_lexical())
        .map(|t| t.text(src))
        .collect();
    assert_eq!(lex, ["foo", "bar", "baz"]);
}

// ============================================================
// Token::value semantics
// ============================================================

#[test]
fn token_value_is_not_cached() {
    let src = r"'f\x6Fo'";
    let t = first(src);
    let a = t.value(src);
    let b = t.value(src);
    assert_eq!(a, b);
    match (a, b) {
        (TokenValue::Atom(Cow::Owned(a)), TokenValue::Atom(Cow::Owned(b))) => {
            assert_eq!(a, b);
            assert_ne!(a.as_ptr(), b.as_ptr());
        }
        _ => panic!("expected two independent Owned atoms"),
    }
}

// ============================================================
// Error / resume_position
// ============================================================

#[test]
fn error_is_copy_and_exposes_positions() {
    fn take_copy<T: Copy>(_: T) {}
    let err = scan_token("\u{2603}", Position::new()).unwrap_err();
    take_copy(err);
    let _ = err.position();
    let _ = err.resume_position();
}

#[test]
fn resume_position_advances_one_unicode_scalar() {
    let src = "\u{1F600} rest";
    let err = scan_token(src, Position::new()).unwrap_err();
    let r = err.resume_position();
    assert_eq!(r.offset(), '\u{1F600}'.len_utf8());
    assert!(src.is_char_boundary(r.offset()));
    let next = scan_token(src, r).unwrap().unwrap();
    assert_eq!(next.text(src), " ");
}

#[test]
fn resume_position_monotonic_on_repeated_errors() {
    let src = "\u{FFFC}\u{FFFC}";
    let mut p = Position::new();
    for _ in 0..2 {
        let e = scan_token(src, p).unwrap_err();
        assert!(e.resume_position().offset() > p.offset());
        p = e.resume_position();
    }
    assert_eq!(p.offset(), src.len());
    assert_eq!(scan_token(src, p).unwrap(), None);
}

// ============================================================
// Compile-time Copy / Hash sanity
// ============================================================

#[test]
fn token_position_and_kind_are_copy_and_hashable() {
    fn take_copy<T: Copy>(_: T) {}
    fn take_hash<T: std::hash::Hash>(_: T) {}
    let t = first("foo");
    take_copy(t);
    take_hash(t);
    take_copy(t.kind());
    take_copy(t.start());
    let copy = t;
    assert_eq!(t.kind(), copy.kind());
}

// ============================================================
// Real-world Erlang patterns (smoke tests)
// ============================================================

#[test]
fn tokenize_gen_server_callback() {
    let src = "handle_call(Request, From, State) ->\n    {reply, ok, State}.";
    let all = texts(src);
    assert_eq!(all[0], "handle_call");
    assert!(all.contains(&"->"));
    assert!(all.contains(&"{"));
    assert!(all.contains(&"}"));
}

#[test]
fn tokenize_otp_style_module() {
    let src = r#"-module(my_server).
-behaviour(gen_server).

-export([start_link/0, init/1, handle_call/3, handle_cast/2]).

-record(state, {count = 0 :: non_neg_integer()}).

start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

init([]) ->
    {ok, #state{}}.

handle_call(get_count, _From, #state{count = Count} = State) ->
    {reply, Count, State};
handle_call(_Request, _From, State) ->
    {reply, ok, State}.

handle_cast(increment, #state{count = Count} = State) ->
    {noreply, State#state{count = Count + 1}}.
"#;
    assert!(scan_tokens(src).len() > 100);
}
