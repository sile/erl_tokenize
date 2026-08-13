use erl_tokenize::tokens::{
    AtomToken, CharToken, CommentToken, FloatToken, IntegerToken, KeywordToken, SigilStringToken,
    StringToken, SymbolToken, VariableToken, WhitespaceToken,
};
use erl_tokenize::values::{Keyword, Symbol, Whitespace};
use erl_tokenize::{Position, PositionRange, TokenKind, Tokenizer, scan_token};

macro_rules! tokenize {
    ($text:expr) => {{
        let src: &str = $text;
        Tokenizer::new(src)
            .map(|t| t.unwrap().text(src).to_string())
            .collect::<Vec<_>>()
    }};
}

fn pos() -> Position {
    Position::new()
}

// ============================================================
// Atom tests
// ============================================================

#[test]
fn atom_bare() {
    assert_eq!(tokenize!("foo"), ["foo"]);
    assert_eq!(tokenize!("hello_world"), ["hello_world"]);
    assert_eq!(tokenize!("foo@bar"), ["foo@bar"]);
    assert_eq!(tokenize!("a123"), ["a123"]);
}

#[test]
fn atom_bare_unicode() {
    assert_eq!(tokenize!("comté"), ["comté"]);
    assert_eq!(tokenize!("äfunc"), ["äfunc"]);
    assert_eq!(tokenize!("ärlig"), ["ärlig"]);
}

#[test]
fn atom_quoted_basic() {
    assert_eq!(tokenize!("'foo'"), ["'foo'"]);
    assert_eq!(tokenize!("'Foo'"), ["'Foo'"]);
    assert_eq!(tokenize!("'hello world'"), ["'hello world'"]);
    assert_eq!(tokenize!("''"), ["''"]);
}

#[test]
fn atom_quoted_escapes() {
    let t = AtomToken::from_text(r"'f\x6Fo'", pos()).unwrap();
    assert_eq!(t.value(), "foo");
    assert_eq!(t.text(), r"'f\x6Fo'");

    let t = AtomToken::from_text(r"'a\nb'", pos()).unwrap();
    assert_eq!(t.value(), "a\nb");

    let t = AtomToken::from_text(r"'a\\b'", pos()).unwrap();
    assert_eq!(t.value(), "a\\b");

    let t = AtomToken::from_text(r"'a\'b'", pos()).unwrap();
    assert_eq!(t.value(), "a'b");
}

#[test]
fn atom_from_value() {
    let t = AtomToken::from_value("foo", pos());
    assert_eq!(t.text(), "'foo'");

    let t = AtomToken::from_value("foo's", pos());
    assert_eq!(t.text(), r"'foo\'s'");

    let t = AtomToken::from_value("a\\b", pos());
    assert_eq!(t.text(), r"'a\\b'");

    let t = AtomToken::from_value("a\x001b", pos());
    assert_eq!(t.text(), r"'a\x{0}1b'");
}

#[test]
fn atom_vs_keyword() {
    let token = scan_token("case", pos()).unwrap().unwrap();
    assert!(matches!(token.kind(), TokenKind::Keyword(Keyword::Case)));

    let token = scan_token("case_x", pos()).unwrap().unwrap();
    assert_eq!(token.kind(), TokenKind::Atom);

    let token = scan_token("foo", pos()).unwrap().unwrap();
    assert_eq!(token.kind(), TokenKind::Atom);
}

#[test]
fn atom_errors() {
    assert!(AtomToken::from_text("  foo", pos()).is_err());
    assert!(AtomToken::from_text("123", pos()).is_err());
    assert!(AtomToken::from_text("", pos()).is_err());
}

// ============================================================
// Character token tests
// ============================================================

#[test]
fn char_simple() {
    let t = CharToken::from_text("$a", pos()).unwrap();
    assert_eq!(t.value(), 'a');
    assert_eq!(t.text(), "$a");

    let t = CharToken::from_text("$Z", pos()).unwrap();
    assert_eq!(t.value(), 'Z');

    let t = CharToken::from_text("$0", pos()).unwrap();
    assert_eq!(t.value(), '0');

    let t = CharToken::from_text("$ ", pos()).unwrap();
    assert_eq!(t.value(), ' ');
}

#[test]
fn char_named_escapes() {
    let cases = [
        (r"$\b", 8u32), // backspace
        (r"$\d", 127),  // delete
        (r"$\e", 27),   // escape
        (r"$\f", 12),   // form feed
        (r"$\n", '\n' as u32),
        (r"$\r", '\r' as u32),
        (r"$\s", ' ' as u32),
        (r"$\t", '\t' as u32),
        (r"$\v", 11), // vertical tab
    ];
    for (src, expected) in cases {
        let t = CharToken::from_text(src, pos()).unwrap();
        assert_eq!(
            t.value() as u32,
            expected,
            "failed for {src}: got {:?}",
            t.value()
        );
    }
}

#[test]
fn char_caret_notation() {
    // All 59 characters accepted after `\^` (OTP 26). The expected values
    // are hardcoded so the test does not duplicate the implementation's
    // `% 32` arithmetic.
    let cases: &[(char, u32)] = &[
        ('@', 0x00),
        ('[', 0x1b),
        ('\\', 0x1c),
        (']', 0x1d),
        ('^', 0x1e),
        ('_', 0x1f),
        ('A', 0x01),
        ('B', 0x02),
        ('C', 0x03),
        ('D', 0x04),
        ('E', 0x05),
        ('F', 0x06),
        ('G', 0x07),
        ('H', 0x08),
        ('I', 0x09),
        ('J', 0x0a),
        ('K', 0x0b),
        ('L', 0x0c),
        ('M', 0x0d),
        ('N', 0x0e),
        ('O', 0x0f),
        ('P', 0x10),
        ('Q', 0x11),
        ('R', 0x12),
        ('S', 0x13),
        ('T', 0x14),
        ('U', 0x15),
        ('V', 0x16),
        ('W', 0x17),
        ('X', 0x18),
        ('Y', 0x19),
        ('Z', 0x1a),
        ('a', 0x01),
        ('b', 0x02),
        ('c', 0x03),
        ('d', 0x04),
        ('e', 0x05),
        ('f', 0x06),
        ('g', 0x07),
        ('h', 0x08),
        ('i', 0x09),
        ('j', 0x0a),
        ('k', 0x0b),
        ('l', 0x0c),
        ('m', 0x0d),
        ('n', 0x0e),
        ('o', 0x0f),
        ('p', 0x10),
        ('q', 0x11),
        ('r', 0x12),
        ('s', 0x13),
        ('t', 0x14),
        ('u', 0x15),
        ('v', 0x16),
        ('w', 0x17),
        ('x', 0x18),
        ('y', 0x19),
        ('z', 0x1a),
        ('?', 0x7f), // Delete
    ];
    for (c, expected) in cases {
        let src = format!("$\\^{c}");
        let t = CharToken::from_text(&src, pos()).unwrap();
        assert_eq!(t.value() as u32, *expected, "failed for {src:?}");
        assert_eq!(t.text(), src);
    }
}

#[test]
fn char_caret_notation_invalid() {
    // As of OTP 26, only the documented characters are allowed after `\^`.
    assert!(CharToken::from_text(r"$\^!", pos()).is_err());
    assert!(CharToken::from_text(r"$\^0", pos()).is_err());
    assert!(CharToken::from_text(r"$\^あ", pos()).is_err());
    // Boundary characters around the allowed set.
    assert!(CharToken::from_text(r"$\^>", pos()).is_err()); // 0x3E: just after `?`
    assert!(CharToken::from_text(r"$\^`", pos()).is_err()); // 0x60: between `_` and `a`
    assert!(CharToken::from_text(r"$\^{", pos()).is_err()); // 0x7B: just after `z`
}

#[test]
fn char_hex_invalid() {
    assert!(CharToken::from_text(r"$\x{ab", pos()).is_err()); // unterminated braces
    assert!(CharToken::from_text(r"$\x{}", pos()).is_err()); // empty braces
    assert!(CharToken::from_text(r"$\x{", pos()).is_err()); // EOF right after `{`
    assert!(CharToken::from_text(r"$\x", pos()).is_err()); // EOF right after `\x`
    assert!(CharToken::from_text(r"$\x6", pos()).is_err()); // fixed-width form needs two digits
    assert!(CharToken::from_text(r"$\x{zz}", pos()).is_err()); // non-hex digits
    assert!(CharToken::from_text(r"$\x{110000}", pos()).is_err()); // out of Unicode range
    assert!(CharToken::from_text(r"$\x{D800}", pos()).is_err()); // surrogate
}

#[test]
fn char_octal() {
    // The octal loop must accumulate each peeked digit; a naive impl
    // that reuses the initial digit yields the wrong code point for
    // multi-digit escapes (e.g. \123 would return 'I' instead of 'S').
    let t = CharToken::from_text(r"$\123", pos()).unwrap();
    assert_eq!(t.value(), 'S'); // 0o123 = 83 = 'S'

    let t = CharToken::from_text(r"$\17", pos()).unwrap();
    assert_eq!(t.value() as u32, 15);

    let t = CharToken::from_text(r"$\01", pos()).unwrap();
    assert_eq!(t.value() as u32, 1);

    let t = CharToken::from_text(r"$\0", pos()).unwrap();
    assert_eq!(t.value(), '\0');

    let t = CharToken::from_text(r"$\7", pos()).unwrap();
    assert_eq!(t.value() as u32, 7);

    // 3-digit maximum (0o377 = 255).
    let t = CharToken::from_text(r"$\377", pos()).unwrap();
    assert_eq!(t.value() as u32, 255);

    // The octal escape must stop after 3 digits: `$\1234` tokenizes as
    // the char `$\123` followed by the integer `4`.
    assert_eq!(tokenize!(r"$\1234"), [r"$\123", "4"]);
}

#[test]
fn char_hex() {
    let t = CharToken::from_text(r"$\x6F", pos()).unwrap();
    assert_eq!(t.value(), 'o');

    let t = CharToken::from_text(r"$\x41", pos()).unwrap();
    assert_eq!(t.value(), 'A');
}

#[test]
fn char_hex_braces() {
    let t = CharToken::from_text(r"$\x{06F}", pos()).unwrap();
    assert_eq!(t.value(), 'o');

    let t = CharToken::from_text(r"$\x{41}", pos()).unwrap();
    assert_eq!(t.value(), 'A');

    let t = CharToken::from_text(r"$\x{aaa}", pos()).unwrap();
    assert_eq!(t.value(), '\u{aaa}');

    let t = CharToken::from_text(r"$\x{10FFFF}", pos()).unwrap();
    assert_eq!(t.value(), '\u{10FFFF}');
}

#[test]
fn char_from_value() {
    let t = CharToken::from_value('a', pos());
    assert_eq!(t.text(), "$a");
    assert_eq!(t.value(), 'a');

    let t = CharToken::from_value('\\', pos());
    assert_eq!(t.text(), r"$\\");

    let t = CharToken::from_value('\0', pos());
    assert_eq!(t.text(), r"$\x{0}");

    let t = CharToken::from_value('\u{1}', pos());
    assert_eq!(t.text(), r"$\x{1}");
}

#[test]
fn char_errors() {
    assert!(CharToken::from_text("  $a", pos()).is_err());
    assert!(CharToken::from_text(r"$\", pos()).is_err());
    assert!(CharToken::from_text("a", pos()).is_err());
    assert!(CharToken::from_text("$", pos()).is_err());
}

// ============================================================
// Comment tests
// ============================================================

#[test]
fn comment_basic() {
    assert_eq!(tokenize!("% foo"), ["% foo"]);
    assert_eq!(tokenize!("%"), ["%"]);
    assert_eq!(tokenize!("%% foo "), ["%% foo "]);
    assert_eq!(tokenize!("%%% module doc"), ["%%% module doc"]);
}

#[test]
fn comment_value() {
    let t = CommentToken::from_text("%", pos()).unwrap();
    assert_eq!(t.value(), "");

    let t = CommentToken::from_text("%% foo ", pos()).unwrap();
    assert_eq!(t.value(), "% foo ");
}

#[test]
fn comment_multiline() {
    let src = "% line1\n% line2";
    assert_eq!(tokenize!(src), ["% line1", "\n", "% line2"]);
}

#[test]
fn comment_from_value() {
    let t = CommentToken::from_value("foo", pos()).unwrap();
    assert_eq!(t.text(), "%foo");

    assert!(CommentToken::from_value("foo\nbar", pos()).is_err());
}

#[test]
fn comment_errors() {
    assert!(CommentToken::from_text("  % foo", pos()).is_err());
    assert!(CommentToken::from_text("foo", pos()).is_err());
}

// ============================================================
// Integer tests
// ============================================================

#[test]
fn integer_basic() {
    let t = IntegerToken::from_text("0", pos()).unwrap();
    assert_eq!(t.value(), Some(0));

    let t = IntegerToken::from_text("42", pos()).unwrap();
    assert_eq!(t.value(), Some(42));

    let t = IntegerToken::from_text("123456789", pos()).unwrap();
    assert_eq!(t.value(), Some(123456789));
}

#[test]
fn integer_underscores() {
    let t = IntegerToken::from_text("123_456", pos()).unwrap();
    assert_eq!(t.value(), Some(123456));
    assert_eq!(t.text(), "123_456");

    let t = IntegerToken::from_text("123_456_789", pos()).unwrap();
    assert_eq!(t.value(), Some(123456789));

    let t = IntegerToken::from_text("1_2", pos()).unwrap();
    assert_eq!(t.value(), Some(12));
}

#[test]
fn integer_based() {
    let t = IntegerToken::from_text("2#101", pos()).unwrap();
    assert_eq!(t.value(), Some(5));

    let t = IntegerToken::from_text("8#777", pos()).unwrap();
    assert_eq!(t.value(), Some(0o777));

    let t = IntegerToken::from_text("16#FF", pos()).unwrap();
    assert_eq!(t.value(), Some(0xFF));

    let t = IntegerToken::from_text("16#ab0e", pos()).unwrap();
    assert_eq!(t.value(), Some(0xab0e));

    let t = IntegerToken::from_text("36#ZZ", pos()).unwrap();
    assert_eq!(t.value(), Some(35 * 36 + 35));
}

#[test]
fn integer_based_with_underscores() {
    let t = IntegerToken::from_text("1_6#10", pos()).unwrap();
    assert_eq!(t.value(), Some(16));
    assert_eq!(t.text(), "1_6#10");

    let t = IntegerToken::from_text("1_6#a_b_0e", pos()).unwrap();
    assert_eq!(t.value(), Some(0xab0e));

    let t = IntegerToken::from_text("2#0011_0101_0011", pos()).unwrap();
    assert_eq!(t.value(), Some(0b001101010011));
}

#[test]
fn integer_out_of_range() {
    let t = IntegerToken::from_text("9223372036854775808", pos()).unwrap();
    assert_eq!(t.value(), None);
    assert_eq!(t.text(), "9223372036854775808");
}

#[test]
fn integer_from_value() {
    let t = IntegerToken::from_value(123, pos());
    assert_eq!(t.text(), "123");
    assert_eq!(t.value(), Some(123));
}

#[test]
fn integer_errors() {
    assert!(IntegerToken::from_text("-10", pos()).is_err());
    assert!(IntegerToken::from_text("123_456_", pos()).is_err());
    assert!(IntegerToken::from_text("123__456", pos()).is_err());
    assert!(IntegerToken::from_text("123_", pos()).is_err());
    assert!(IntegerToken::from_text("_123", pos()).is_err());
}

#[test]
fn integer_invalid_base() {
    assert!(IntegerToken::from_text("1#000", pos()).is_err());
    assert!(IntegerToken::from_text("37#000", pos()).is_err());
}

// ============================================================
// Float tests
// ============================================================

#[test]
fn float_basic() {
    let t = FloatToken::from_text("0.1", pos()).unwrap();
    assert_eq!(t.value(), 0.1);

    // The literal is intentional: the test verifies that parsing "3.14"
    // yields exactly the value 3.14 (not PI).
    #[expect(
        clippy::approx_constant,
        reason = "intentional: parse \"3.14\" must yield 3.14"
    )]
    {
        let t = FloatToken::from_text("3.14", pos()).unwrap();
        assert_eq!(t.value(), 3.14);
    }

    let t = FloatToken::from_text("1.0", pos()).unwrap();
    assert_eq!(t.value(), 1.0);
}

#[test]
fn float_scientific() {
    let t = FloatToken::from_text("12.3e-1", pos()).unwrap();
    assert_eq!(t.value(), 1.23);

    let t = FloatToken::from_text("1.0e10", pos()).unwrap();
    assert_eq!(t.value(), 1.0e10);

    let t = FloatToken::from_text("1.0E10", pos()).unwrap();
    assert_eq!(t.value(), 1.0e10);

    let t = FloatToken::from_text("1.0e+3", pos()).unwrap();
    assert_eq!(t.value(), 1.0e3);

    let t = FloatToken::from_text("1.0e-3", pos()).unwrap();
    assert_eq!(t.value(), 1.0e-3);
}

#[test]
fn float_underscores() {
    let t = FloatToken::from_text("1_2.3_4e-1_0", pos()).unwrap();
    assert_eq!(t.value(), 0.000000001234);

    let t = FloatToken::from_text("1_0.0", pos()).unwrap();
    assert_eq!(t.value(), 10.0);

    let t = FloatToken::from_text("1.2_3e+1_0", pos()).unwrap();
    assert_eq!(t.text(), "1.2_3e+1_0");
}

#[test]
fn float_based_binary() {
    let t = FloatToken::from_text("2#0.111", pos()).unwrap();
    assert_eq!(t.value(), 0.875);

    let t = FloatToken::from_text("2#101.0", pos()).unwrap();
    assert_eq!(t.value(), 5.0);

    let t = FloatToken::from_text("2#101.1", pos()).unwrap();
    assert_eq!(t.value(), 5.5);

    let t = FloatToken::from_text("2#101.101", pos()).unwrap();
    assert_eq!(t.value(), 5.625);
}

#[test]
fn float_based_hex() {
    let t = FloatToken::from_text("16#f_f.F_F", pos()).unwrap();
    assert_eq!(t.value(), 255.99609375);

    let t = FloatToken::from_text("16#100.0", pos()).unwrap();
    assert_eq!(t.value(), 256.0);
}

#[test]
fn float_based_with_exponent() {
    let t = FloatToken::from_text("2#0.10101#e8", pos()).unwrap();
    assert_eq!(t.value(), 168.0);

    let t = FloatToken::from_text("2#1.0#e-3", pos()).unwrap();
    assert_eq!(t.value(), 0.125);

    let t = FloatToken::from_text("1_6#fefe.fefe#e1_6", pos()).unwrap();
    assert_eq!(t.value(), 1.2041849337671418e24);

    let t = FloatToken::from_text("10#1.0#e0", pos()).unwrap();
    assert_eq!(t.value(), 1.0);

    let t = FloatToken::from_text("10#1.0#e-3", pos()).unwrap();
    assert_eq!(t.value(), 0.001);
}

#[test]
fn float_from_value() {
    let t = FloatToken::from_value(1.23, pos());
    assert_eq!(t.text(), "1.23");
    assert_eq!(t.value(), 1.23);
}

#[test]
fn float_errors() {
    assert!(FloatToken::from_text("123", pos()).is_err());
    assert!(FloatToken::from_text(".123", pos()).is_err());
    assert!(FloatToken::from_text("1.", pos()).is_err());
    assert!(FloatToken::from_text("12_.3", pos()).is_err());
    assert!(FloatToken::from_text("12._3", pos()).is_err());
    assert!(FloatToken::from_text("12.3_", pos()).is_err());
    assert!(FloatToken::from_text("1__2.3", pos()).is_err());
    assert!(FloatToken::from_text("12.3__4", pos()).is_err());
    assert!(FloatToken::from_text("10_#12.34", pos()).is_err());
    assert!(FloatToken::from_text("12.34e-1__0", pos()).is_err());
    assert!(FloatToken::from_text("10#.123", pos()).is_err());
    assert!(FloatToken::from_text("10#1.", pos()).is_err());
    assert!(FloatToken::from_text("10#12_.3", pos()).is_err());
    assert!(FloatToken::from_text("10#12._3", pos()).is_err());
    assert!(FloatToken::from_text("10#12.3_", pos()).is_err());
    assert!(FloatToken::from_text("10#1__2.3", pos()).is_err());
    assert!(FloatToken::from_text("10#12.3__4", pos()).is_err());
}

// ============================================================
// Keyword tests
// ============================================================

#[test]
fn keyword_all() {
    let keywords = [
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
    for (text, expected) in keywords {
        let t = KeywordToken::from_text(text, pos()).unwrap();
        assert_eq!(t.value(), expected, "failed for keyword: {text}");
        assert_eq!(t.text(), text);
    }
}

#[test]
fn keyword_with_trailing() {
    let t = KeywordToken::from_text("and  ", pos()).unwrap();
    assert_eq!(t.value(), Keyword::And);
    assert_eq!(t.text(), "and");
}

#[test]
fn keyword_not_keyword() {
    assert!(KeywordToken::from_text("foo", pos()).is_err());
    assert!(KeywordToken::from_text("  and", pos()).is_err());
    assert!(KeywordToken::from_text("andfoo", pos()).is_err());
}

#[test]
fn keyword_from_value() {
    let t = KeywordToken::from_value(Keyword::Case, pos());
    assert_eq!(t.text(), "case");
    assert_eq!(t.value(), Keyword::Case);
}

// ============================================================
// String tests
// ============================================================

#[test]
fn string_basic() {
    let t = StringToken::from_text(r#""foo""#, pos()).unwrap();
    assert_eq!(t.value(), "foo");
    assert_eq!(t.text(), r#""foo""#);

    let t = StringToken::from_text(r#""""#, pos()).unwrap();
    assert_eq!(t.value(), "");
}

#[test]
fn string_escapes() {
    let t = StringToken::from_text(r#""a\nb""#, pos()).unwrap();
    assert_eq!(t.value(), "a\nb");

    let t = StringToken::from_text(r#""a\tb""#, pos()).unwrap();
    assert_eq!(t.value(), "a\tb");

    let t = StringToken::from_text(r#""a\\b""#, pos()).unwrap();
    assert_eq!(t.value(), "a\\b");

    let t = StringToken::from_text(r#""a\"b""#, pos()).unwrap();
    assert_eq!(t.value(), "a\"b");

    let t = StringToken::from_text(r#""f\x6Fo""#, pos()).unwrap();
    assert_eq!(t.value(), "foo");

    // Octal escapes are shared with the char path but must also be
    // exercised through `StringToken::from_text` so a regression in the
    // string path alone can be caught.
    let t = StringToken::from_text(r#""\123""#, pos()).unwrap();
    assert_eq!(t.value(), "S"); // 0o123 = 83 = 'S'

    let t = StringToken::from_text(r#""\0a""#, pos()).unwrap();
    assert_eq!(t.value(), "\0a");
}

#[test]
fn string_hex_unicode() {
    let t = StringToken::from_text(r#""\x{aaa}""#, pos()).unwrap();
    assert_eq!(t.value(), "\u{aaa}");

    let t = StringToken::from_text(r#""\x41\x{fff}\x42""#, pos()).unwrap();
    assert_eq!(t.value(), "\x41\u{fff}\x42");
}

#[test]
fn string_with_newlines() {
    let t = StringToken::from_text("\"a\nb\"", pos()).unwrap();
    assert_eq!(t.value(), "a\nb");
}

#[test]
fn string_from_value() {
    let t = StringToken::from_value("foo", pos());
    assert_eq!(t.text(), r#""foo""#);
    assert_eq!(t.value(), "foo");

    // NUL is always rewritten to `\x{0}` so it cannot merge with a
    // following octal digit into a single character.
    let t = StringToken::from_value("\0", pos());
    assert_eq!(t.text(), r#""\x{0}""#);
    // `"\x001"` is `\x00` + literal `1` (Rust has no octal escapes, so
    // `\0` before a digit is written this way to avoid clippy warnings).
    let t = StringToken::from_value("\x001", pos());
    assert_eq!(t.text(), r#""\x{0}1""#);

    // Non-printable Unicode is emitted as `\x{...}` (Erlang has no `\u{...}`).
    let t = StringToken::from_value("a\u{1}b", pos());
    assert_eq!(t.text(), r#""a\x{1}b""#);
    let t = StringToken::from_value("a\u{10ffff}b", pos());
    assert_eq!(t.text(), r#""a\x{10ffff}b""#);

    // Named escapes and printable chars pass through as-is.
    let t = StringToken::from_value("a\nb\tc\\d\"e", pos());
    assert_eq!(t.text(), r#""a\nb\tc\\d\"e""#);
}

#[test]
fn string_errors() {
    assert!(StringToken::from_text(r#"  "foo""#, pos()).is_err());
    assert!(StringToken::from_text("", pos()).is_err());
}

// `StringToken::from_value` depends on the exact shape of
// `char::escape_debug` output (NUL as `\0`, non-printable Unicode as
// `\u{...}`, quotes/backslashes/named-escapes as themselves). If a
// future std change alters any of these forms, from_value's rewrite
// logic could silently produce non-Erlang text. This test pins the
// contract so any drift is caught in CI.
#[test]
fn char_escape_debug_output_lock() {
    assert_eq!('\0'.escape_debug().to_string(), r"\0");
    assert_eq!('\u{1}'.escape_debug().to_string(), r"\u{1}");
    assert_eq!('\u{10ffff}'.escape_debug().to_string(), r"\u{10ffff}");
    assert_eq!('\n'.escape_debug().to_string(), r"\n");
    assert_eq!('\r'.escape_debug().to_string(), r"\r");
    assert_eq!('\t'.escape_debug().to_string(), r"\t");
    assert_eq!('\\'.escape_debug().to_string(), r"\\");
    assert_eq!('"'.escape_debug().to_string(), "\\\"");
    assert_eq!('\''.escape_debug().to_string(), r"\'");
    assert_eq!('a'.escape_debug().to_string(), "a");
}

#[test]
fn string_adjacent_error() {
    let src = r#""a""b""#;
    let mut tokenizer = Tokenizer::new(src);
    let result = tokenizer.next().unwrap();
    assert!(result.is_err());
}

// ============================================================
// Triple-quoted string tests
// ============================================================

#[test]
fn triple_quoted_basic() {
    let src = "\"\"\"\nfoo\n\"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "foo");
}

#[test]
fn triple_quoted_indented() {
    let src = "\"\"\"\n foo\n \"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "foo");
}

#[test]
fn triple_quoted_multiline() {
    let src = "\"\"\"\nline1\nline2\nline3\n\"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "line1\nline2\nline3");
}

#[test]
fn triple_quoted_empty() {
    let src = "\"\"\"\n\"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "");
}

#[test]
fn triple_quoted_blank_line() {
    let src = "\"\"\"\n\n\"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "");
}

#[test]
fn triple_quoted_two_blank_lines() {
    let src = "\"\"\"\n\n\n\"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "\n");
}

#[test]
fn triple_quoted_four_quotes() {
    let src = "\"\"\"\"\nfoo\n\"\"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "foo");
}

#[test]
fn triple_quoted_five_quotes() {
    let src = "\"\"\"\"\"\n  5-quoted\n  \"\"\"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "5-quoted");
}

#[test]
fn triple_quoted_containing_quotes() {
    let src = "\"\"\"\ncontains \"quotes\" here\n\"\"\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), "contains \"quotes\" here");
}

#[test]
fn triple_quoted_errors() {
    let src = r#""""foo""""#;
    assert!(StringToken::from_text(src, pos()).is_err());

    let src = r#""""\nfoo\n """"#;
    assert!(StringToken::from_text(src, pos()).is_err());
}

// ============================================================
// Sigil string tests
// ============================================================

#[test]
fn sigil_empty() {
    let t = SigilStringToken::from_text("~\"\"", pos()).unwrap();
    assert_eq!(t.value(), ("", "", ""));
}

#[test]
fn sigil_all_delimiters() {
    let cases = [
        ("~a(b)c", ("a", "b", "c")),
        ("~a[b]c", ("a", "b", "c")),
        ("~a{b}c", ("a", "b", "c")),
        ("~a<b>c", ("a", "b", "c")),
        ("~a/b/c", ("a", "b", "c")),
        ("~a|b|c", ("a", "b", "c")),
        ("~a'b'c", ("a", "b", "c")),
        ("~a\"b\"c", ("a", "b", "c")),
        ("~a`b`c", ("a", "b", "c")),
        ("~a#b#c", ("a", "b", "c")),
    ];
    for (src, expected) in cases {
        let t = SigilStringToken::from_text(src, pos()).unwrap();
        assert_eq!(t.value(), expected, "failed for {src}");
    }
}

#[test]
fn sigil_no_prefix_suffix() {
    let t = SigilStringToken::from_text("~(content)", pos()).unwrap();
    assert_eq!(t.value(), ("", "content", ""));

    let t = SigilStringToken::from_text("~\"content\"", pos()).unwrap();
    assert_eq!(t.value(), ("", "content", ""));
}

#[test]
fn sigil_prefix_only() {
    let t = SigilStringToken::from_text("~b\"foo\"", pos()).unwrap();
    assert_eq!(t.value(), ("b", "foo", ""));
}

#[test]
fn sigil_triple_quoted() {
    let src = "~a\"\"\"\n    b\n    \"\"\"c";
    let t = SigilStringToken::from_text(src, pos()).unwrap();
    assert_eq!(t.value(), ("a", "b", "c"));
}

#[test]
fn sigil_in_sequence() {
    assert_eq!(tokenize!("~a`b`c 10"), ["~a`b`c", " ", "10"]);
}

#[test]
fn sigil_errors() {
    assert!(SigilStringToken::from_text(r#""foo""#, pos()).is_err());
    assert!(SigilStringToken::from_text("~", pos()).is_err());
}

// ============================================================
// Symbol tests
// ============================================================

#[test]
fn symbol_single_char() {
    let singles = [
        ("[", Symbol::OpenSquare),
        ("]", Symbol::CloseSquare),
        ("(", Symbol::OpenParen),
        (")", Symbol::CloseParen),
        ("{", Symbol::OpenBrace),
        ("}", Symbol::CloseBrace),
        ("#", Symbol::Sharp),
        ("/", Symbol::Slash),
        (".", Symbol::Dot),
        (",", Symbol::Comma),
        (":", Symbol::Colon),
        (";", Symbol::Semicolon),
        ("=", Symbol::Match),
        ("|", Symbol::VerticalBar),
        ("?", Symbol::Question),
        ("!", Symbol::Bang),
        ("-", Symbol::Hyphen),
        ("+", Symbol::Plus),
        ("*", Symbol::Multiply),
        (">", Symbol::Greater),
        ("<", Symbol::Less),
    ];
    for (text, expected) in singles {
        let t = SymbolToken::from_text(text, pos()).unwrap();
        assert_eq!(t.value(), expected, "failed for: {text}");
        assert_eq!(t.text(), text);
    }
}

#[test]
fn symbol_double_char() {
    let doubles = [
        ("::", Symbol::DoubleColon),
        (":=", Symbol::MapMatch),
        ("||", Symbol::DoubleVerticalBar),
        ("--", Symbol::MinusMinus),
        ("++", Symbol::PlusPlus),
        ("->", Symbol::RightArrow),
        ("<-", Symbol::LeftArrow),
        ("=>", Symbol::DoubleRightArrow),
        ("<=", Symbol::DoubleLeftArrow),
        (">>", Symbol::DoubleRightAngle),
        ("<<", Symbol::DoubleLeftAngle),
        ("==", Symbol::Eq),
        ("/=", Symbol::NotEq),
        (">=", Symbol::GreaterEq),
        ("=<", Symbol::LessEq),
        ("??", Symbol::DoubleQuestion),
        ("?=", Symbol::MaybeMatch),
        ("..", Symbol::DoubleDot),
        ("&&", Symbol::DoubleAmpersand),
    ];
    for (text, expected) in doubles {
        let t = SymbolToken::from_text(text, pos()).unwrap();
        assert_eq!(t.value(), expected, "failed for: {text}");
        assert_eq!(t.text(), text);
    }
}

#[test]
fn symbol_triple_char() {
    let triples = [
        ("=:=", Symbol::ExactEq),
        ("=/=", Symbol::ExactNotEq),
        ("...", Symbol::TripleDot),
        ("<:-", Symbol::StrictLeftArrow),
        ("<:=", Symbol::StrictDoubleLeftArrow),
    ];
    for (text, expected) in triples {
        let t = SymbolToken::from_text(text, pos()).unwrap();
        assert_eq!(t.value(), expected, "failed for: {text}");
        assert_eq!(t.text(), text);
    }
}

#[test]
fn symbol_from_value() {
    let t = SymbolToken::from_value(Symbol::Dot, pos());
    assert_eq!(t.text(), ".");
}

#[test]
fn symbol_errors() {
    assert!(SymbolToken::from_text("  .", pos()).is_err());
    assert!(SymbolToken::from_text("foo", pos()).is_err());
    assert!(SymbolToken::from_text("", pos()).is_err());
}

#[test]
fn symbol_disambiguation() {
    assert_eq!(tokenize!("=:="), ["=:="]);
    assert_eq!(tokenize!("=/="), ["=/="]);
    assert_eq!(tokenize!("..."), ["..."]);
    assert_eq!(tokenize!(".."), [".."]);

    assert_eq!(tokenize!("<<>>"), ["<<", ">>"]);
    assert_eq!(tokenize!("<="), ["<="]);
    assert_eq!(tokenize!("=<"), ["=<"]);
}

// ============================================================
// Variable tests
// ============================================================

#[test]
fn variable_basic() {
    let t = VariableToken::from_text("Foo", pos()).unwrap();
    assert_eq!(t.value(), "Foo");

    let t = VariableToken::from_text("BAR", pos()).unwrap();
    assert_eq!(t.value(), "BAR");

    let t = VariableToken::from_text("X", pos()).unwrap();
    assert_eq!(t.value(), "X");
}

#[test]
fn variable_underscore() {
    let t = VariableToken::from_text("_", pos()).unwrap();
    assert_eq!(t.value(), "_");

    let t = VariableToken::from_text("_Baz", pos()).unwrap();
    assert_eq!(t.value(), "_Baz");

    let t = VariableToken::from_text("_foo", pos()).unwrap();
    assert_eq!(t.value(), "_foo");
}

#[test]
fn variable_with_at() {
    let t = VariableToken::from_text("_foo@bar", pos()).unwrap();
    assert_eq!(t.value(), "_foo@bar");

    let t = VariableToken::from_text("Var@123", pos()).unwrap();
    assert_eq!(t.value(), "Var@123");
}

#[test]
fn variable_from_value() {
    let t = VariableToken::from_value("Foo", pos()).unwrap();
    assert_eq!(t.text(), "Foo");

    assert!(VariableToken::from_value("foo", pos()).is_err());
}

#[test]
fn variable_errors() {
    assert!(VariableToken::from_text("foo", pos()).is_err());
    assert!(VariableToken::from_text("  Foo", pos()).is_err());
    assert!(VariableToken::from_text("", pos()).is_err());
}

// ============================================================
// Whitespace tests
// ============================================================

#[test]
fn whitespace_all_types() {
    let cases = [
        (" ", Whitespace::Space),
        ("\t", Whitespace::Tab),
        ("\r", Whitespace::Return),
        ("\n", Whitespace::Newline),
        ("\u{A0}", Whitespace::NoBreakSpace),
    ];
    for (text, expected) in cases {
        let t = WhitespaceToken::from_text(text, pos()).unwrap();
        assert_eq!(t.value(), expected, "failed for whitespace: {expected:?}");
    }
}

#[test]
fn whitespace_as_char() {
    assert_eq!(Whitespace::Space.as_char(), ' ');
    assert_eq!(Whitespace::Tab.as_char(), '\t');
    assert_eq!(Whitespace::Return.as_char(), '\r');
    assert_eq!(Whitespace::Newline.as_char(), '\n');
    assert_eq!(Whitespace::NoBreakSpace.as_char(), '\u{A0}');
}

#[test]
fn whitespace_errors() {
    assert!(WhitespaceToken::from_text("foo", pos()).is_err());
    assert!(WhitespaceToken::from_text("", pos()).is_err());
}

// ============================================================
// Tokenizer integration tests
// ============================================================

#[test]
fn tokenize_module_declaration() {
    let src = "-module(foo).";
    assert_eq!(tokenize!(src), ["-", "module", "(", "foo", ")", "."]);
}

#[test]
fn tokenize_function_definition() {
    let src = "add(X, Y) -> X + Y.";
    assert_eq!(
        tokenize!(src),
        [
            "add", "(", "X", ",", " ", "Y", ")", " ", "->", " ", "X", " ", "+", " ", "Y", "."
        ]
    );
}

#[test]
fn tokenize_export() {
    let src = "-export([foo/0, bar/2]).";
    assert_eq!(
        tokenize!(src),
        [
            "-", "export", "(", "[", "foo", "/", "0", ",", " ", "bar", "/", "2", "]", ")", "."
        ]
    );
}

#[test]
fn tokenize_case_expression() {
    let src = "case X of\n    1 -> one;\n    _ -> other\nend";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "case");
    assert_eq!(tokens[4], "of");
    assert!(tokens.contains(&"->".to_string()));
    assert!(tokens.contains(&";".to_string()));
    assert!(tokens.last().unwrap() == "end");
}

#[test]
fn tokenize_guard() {
    let src = "foo(X) when X > 0, X < 100 -> ok.";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"when".to_string()));
    assert!(tokens.contains(&">".to_string()));
    assert!(tokens.contains(&"<".to_string()));
    assert!(tokens.contains(&",".to_string()));
}

#[test]
fn tokenize_binary_syntax() {
    let src = "<<1:8, 2:16/integer-big>>";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "<<");
    assert!(tokens.last().unwrap() == ">>");
}

#[test]
fn tokenize_record() {
    let src = "#person{name = \"John\", age = 30}";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "#");
    assert_eq!(tokens[1], "person");
    assert_eq!(tokens[2], "{");
}

#[test]
fn tokenize_map() {
    let src = "#{key => value, count := 0}";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"#".to_string()));
    assert!(tokens.contains(&"=>".to_string()));
    assert!(tokens.contains(&":=".to_string()));
}

#[test]
fn tokenize_list_comprehension() {
    let src = "[X || X <- List, X > 0]";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"||".to_string()));
    assert!(tokens.contains(&"<-".to_string()));
}

#[test]
fn tokenize_maybe_expression() {
    let src = "maybe\n    {ok, X} ?= foo(),\n    X\nend";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "maybe");
    assert!(tokens.contains(&"?=".to_string()));
    assert!(tokens.last().unwrap() == "end");
}

#[test]
fn tokenize_try_catch() {
    let src = "try foo() catch error:Reason -> {error, Reason} end";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "try");
    assert!(tokens.contains(&"catch".to_string()));
    assert!(tokens.contains(&":".to_string()));
    assert!(tokens.last().unwrap() == "end");
}

#[test]
fn tokenize_fun_expression() {
    let src = "fun(X) -> X + 1 end";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "fun");
    assert!(tokens.last().unwrap() == "end");
}

#[test]
fn tokenize_receive() {
    let src = "receive\n    {msg, X} -> X\nafter 1000 -> timeout\nend";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "receive");
    assert!(tokens.contains(&"after".to_string()));
    assert!(tokens.last().unwrap() == "end");
}

#[test]
fn tokenize_type_spec() {
    let src = "-spec add(integer(), integer()) -> integer().";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "-");
    assert_eq!(tokens[1], "spec");
    assert!(tokens.contains(&"->".to_string()));
}

#[test]
fn tokenize_macro() {
    let src = "-define(MAX, 100).";
    assert_eq!(
        tokenize!(src),
        ["-", "define", "(", "MAX", ",", " ", "100", ")", "."]
    );
}

#[test]
fn tokenize_macro_usage() {
    let src = "?MODULE:?FUNCTION_NAME";
    assert_eq!(tokenize!(src), ["?", "MODULE", ":", "?", "FUNCTION_NAME"]);
}

#[test]
fn tokenize_double_question_macro() {
    let src = "??X";
    assert_eq!(tokenize!(src), ["??", "X"]);
}

#[test]
fn tokenize_bitwise_ops() {
    let src = "X band Y bor Z bxor W bsl 2 bsr 1 bnot X";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"band".to_string()));
    assert!(tokens.contains(&"bor".to_string()));
    assert!(tokens.contains(&"bxor".to_string()));
    assert!(tokens.contains(&"bsl".to_string()));
    assert!(tokens.contains(&"bsr".to_string()));
    assert!(tokens.contains(&"bnot".to_string()));
}

#[test]
fn tokenize_comparison_ops() {
    let src = "A == B, A /= C, A =:= D, A =/= E, A >= F, A =< G";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"==".to_string()));
    assert!(tokens.contains(&"/=".to_string()));
    assert!(tokens.contains(&"=:=".to_string()));
    assert!(tokens.contains(&"=/=".to_string()));
    assert!(tokens.contains(&">=".to_string()));
    assert!(tokens.contains(&"=<".to_string()));
}

#[test]
fn tokenize_list_ops() {
    let src = "[1, 2] ++ [3] -- [1]";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"++".to_string()));
    assert!(tokens.contains(&"--".to_string()));
}

#[test]
fn tokenize_send_op() {
    let src = "Pid ! {msg, Data}";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"!".to_string()));
}

#[test]
fn tokenize_crlf() {
    let src = "a\r\nb";
    assert_eq!(tokenize!(src), ["a", "\r", "\n", "b"]);
}

#[test]
fn tokenize_multiple_spaces() {
    let src = "a   b";
    assert_eq!(tokenize!(src), ["a", " ", " ", " ", "b"]);
}

#[test]
fn tokenize_tabs() {
    let src = "a\t\tb";
    assert_eq!(tokenize!(src), ["a", "\t", "\t", "b"]);
}

#[test]
fn tokenize_mixed_whitespace() {
    let src = "a \t \n b";
    assert_eq!(tokenize!(src), ["a", " ", "\t", " ", "\n", " ", "b"]);
}

#[test]
fn tokenize_nobreak_space() {
    let src = "a\u{a0}b";
    assert_eq!(tokenize!(src), ["a", "\u{a0}", "b"]);
}

#[test]
fn tokenize_consecutive_numbers() {
    let src = "10 1_2_3 1_6#10 1.02 1.2_3e+1_0 1_0.0";
    assert_eq!(
        tokenize!(src),
        [
            "10",
            " ",
            "1_2_3",
            " ",
            "1_6#10",
            " ",
            "1.02",
            " ",
            "1.2_3e+1_0",
            " ",
            "1_0.0"
        ]
    );
}

#[test]
fn tokenize_symbols_complex() {
    let src = r#"[ 0 || x <:- [] && y <:= <<>> ]"#;
    assert_eq!(
        tokenize!(src),
        [
            "[", " ", "0", " ", "||", " ", "x", " ", "<:-", " ", "[", "]", " ", "&&", " ", "y",
            " ", "<:=", " ", "<<", ">>", " ", "]"
        ]
    );
}

#[test]
fn tokenize_comment_in_code() {
    let src = "foo(X) -> % add one\n    X + 1.";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"% add one".to_string()));
}

#[test]
fn tokenize_empty_string() {
    assert!(tokenize!("").is_empty());
    assert_eq!(scan_token("", pos()).unwrap(), None);
}

// ============================================================
// Token filtering tests (replaces the deleted Lexer type)
// ============================================================

fn lexical_texts(src: &str) -> Vec<&str> {
    Tokenizer::new(src)
        .filter_map(|t| {
            let t = t.unwrap();
            t.is_lexical().then(|| t.text(src))
        })
        .collect()
}

#[test]
fn tokenizer_filter_skips_whitespace() {
    assert_eq!(lexical_texts("foo bar"), ["foo", "bar"]);
}

#[test]
fn tokenizer_filter_skips_comments() {
    assert_eq!(lexical_texts("foo % comment\nbar"), ["foo", "bar"]);
}

#[test]
fn tokenizer_filter_complex() {
    let src = "-module(foo).\n\n-export([bar/0]).\n\nbar() -> ok.";
    assert_eq!(
        lexical_texts(src),
        [
            "-", "module", "(", "foo", ")", ".", "-", "export", "(", "[", "bar", "/", "0", "]",
            ")", ".", "bar", "(", ")", "->", "ok", "."
        ]
    );
}

// ============================================================
// Position tracking tests
// ============================================================

#[test]
fn position_single_line() {
    let src = "foo bar";
    let mut tokenizer = Tokenizer::new(src);
    assert_eq!(tokenizer.next_position().offset(), 0);
    assert_eq!(tokenizer.next_position().line(), 1);
    assert_eq!(tokenizer.next_position().column(), 1);

    tokenizer.next(); // "foo"
    assert_eq!(tokenizer.next_position().offset(), 3);
    assert_eq!(tokenizer.next_position().column(), 4);

    tokenizer.next(); // " "
    assert_eq!(tokenizer.next_position().offset(), 4);

    tokenizer.next(); // "bar"
    assert_eq!(tokenizer.next_position().offset(), 7);
}

#[test]
fn position_multiline() {
    let src = "a\nb\nc";
    let mut tokenizer = Tokenizer::new(src);

    tokenizer.next(); // "a"
    assert_eq!(tokenizer.next_position().line(), 1);

    tokenizer.next(); // "\n"
    assert_eq!(tokenizer.next_position().line(), 2);
    assert_eq!(tokenizer.next_position().column(), 1);

    tokenizer.next(); // "b"
    assert_eq!(tokenizer.next_position().line(), 2);
    assert_eq!(tokenizer.next_position().column(), 2);

    tokenizer.next(); // "\n"
    assert_eq!(tokenizer.next_position().line(), 3);

    tokenizer.next(); // "c"
    assert_eq!(tokenizer.next_position().line(), 3);
    assert_eq!(tokenizer.next_position().column(), 2);
}

#[test]
fn position_set_and_reset() {
    let src = "foo bar baz";
    let mut tokenizer = Tokenizer::new(src);

    tokenizer.next(); // "foo"
    tokenizer.next(); // " "
    let saved = tokenizer.next_position();

    tokenizer.next(); // "bar"
    tokenizer.next(); // " "
    assert_eq!(tokenizer.next_position().offset(), 8);

    tokenizer.set_position(saved);
    assert_eq!(tokenizer.next_position().offset(), 4);

    let t = tokenizer.next().unwrap().unwrap();
    assert_eq!(t.text(src), "bar");
}

#[test]
fn position_range_atom() {
    let src = "hello";
    let t = AtomToken::from_text(src, pos()).unwrap();
    assert_eq!(t.start_position().offset(), 0);
    assert_eq!(t.end_position().offset(), 5);
}

#[test]
fn position_range_string_multiline() {
    let src = "\"a\nb\"";
    let t = StringToken::from_text(src, pos()).unwrap();
    let end = t.end_position();
    assert_eq!(end.line(), 2);
    assert_eq!(end.offset(), 5);
}

// ============================================================
// TokenKind / classification tests
// ============================================================

#[test]
fn token_is_lexical_or_hidden() {
    let atom = scan_token("foo", pos()).unwrap().unwrap();
    assert!(atom.is_lexical());
    assert!(!atom.is_hidden());

    let ws = scan_token(" ", pos()).unwrap().unwrap();
    assert!(!ws.is_lexical());
    assert!(ws.is_hidden());

    let comment = scan_token("% test", pos()).unwrap().unwrap();
    assert!(!comment.is_lexical());
    assert!(comment.is_hidden());
}

#[test]
fn token_kinds() {
    let cases: &[(&str, TokenKind)] = &[
        ("foo", TokenKind::Atom),
        ("$a", TokenKind::Char),
        ("42", TokenKind::Integer),
        ("1.5", TokenKind::Float),
        ("case", TokenKind::Keyword(Keyword::Case)),
        (r#""hello""#, TokenKind::String),
        ("~b\"hello\"", TokenKind::SigilString),
        (".", TokenKind::Symbol(Symbol::Dot)),
        ("Foo", TokenKind::Variable),
        ("% comment", TokenKind::Comment),
        (" ", TokenKind::Whitespace),
    ];
    for (src, expected) in cases {
        let t = scan_token(src, pos()).unwrap().unwrap();
        assert_eq!(t.kind(), *expected, "kind mismatch for {src:?}");
    }
}

// ============================================================
// scan_token contract tests
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
    let mut p = Position::new();
    let mut texts = Vec::new();
    while let Some(token) = scan_token(src, p).unwrap() {
        texts.push(token.text(src).to_owned());
        p = token.end();
    }
    assert_eq!(texts, ["io", ":", "format", "(", "\"Hello\"", ")", "."]);
}

#[test]
fn token_text_matches_source_slice() {
    let cases = [
        "foo",
        "'bar'",
        "$a",
        "% comment",
        "42",
        "1.5",
        "case",
        r#""hello""#,
        ".",
        "Foo",
        " ",
    ];
    for src in cases {
        let t = scan_token(src, pos()).unwrap().unwrap();
        assert_eq!(t.text(src), src, "text mismatch for {src:?}");
    }
}

#[test]
fn token_is_copy_and_hashable() {
    fn take_copy<T: Copy>(_: T) {}
    fn take_hash<T: std::hash::Hash>(_: T) {}
    let t = scan_token("foo", pos()).unwrap().unwrap();
    take_copy(t);
    take_hash(t);
    take_copy(t.kind());
    take_copy(t.start());
    let cloned = t;
    assert_eq!(t.kind(), cloned.kind());
}

// ============================================================
// Error recovery tests
// ============================================================

#[test]
fn tokenizer_auto_recovers_via_resume_position() {
    let text = "-module(repro).\n-moduledoc \"\"\"\n\u{5e94}\u{8be5}\u{62a5}\u{9519}\n\"\".";
    let mut token_count = 0;
    for token in Tokenizer::new(text) {
        if token.is_ok() {
            token_count += 1;
        }
    }
    assert!(token_count > 0);
}

#[test]
fn resume_position_advances_one_unicode_scalar() {
    // Non-ASCII invalid symbol should advance by the character's UTF-8
    // length, not a single byte, so we never land in the middle of a code
    // point.
    let src = "\u{1F600} rest";
    let err = scan_token(src, Position::new()).unwrap_err();
    let resume = err.resume_position();
    assert_eq!(resume.offset(), '\u{1F600}'.len_utf8());
    assert!(src.is_char_boundary(resume.offset()));
    // The next scan yields a well-formed token.
    let next = scan_token(src, resume).unwrap().unwrap();
    assert_eq!(next.text(src), " ");
}

#[test]
fn resume_position_updates_line_on_lf() {
    let src = "\n";
    let mut pos = Position::new();
    // Whitespace scan succeeds for `\n`; ensure the diagnostic path we
    // want to exercise instead uses an invalid symbol.
    let err_src = "\u{2603}"; // snowman: fails as a symbol.
    let err = scan_token(err_src, pos).unwrap_err();
    assert_eq!(err.position().line(), 1);
    assert_eq!(err.resume_position().offset(), '\u{2603}'.len_utf8());

    // LF path
    let ws = scan_token(src, pos).unwrap().unwrap();
    pos = ws.end();
    assert_eq!(pos.line(), 2);
    assert_eq!(pos.column(), 1);
}

#[test]
fn resume_position_makes_repeated_errors_monotonic() {
    // Two invalid characters in a row must each advance by one Unicode
    // scalar value.
    let src = "\u{FFFC}\u{FFFC}"; // OBJECT REPLACEMENT CHARACTER twice.
    let mut pos = Position::new();
    let e1 = scan_token(src, pos).unwrap_err();
    assert!(e1.resume_position().offset() > pos.offset());
    pos = e1.resume_position();

    let e2 = scan_token(src, pos).unwrap_err();
    assert!(e2.resume_position().offset() > pos.offset());
    pos = e2.resume_position();

    assert_eq!(pos.offset(), src.len());
    assert_eq!(scan_token(src, pos).unwrap(), None);
}

#[test]
fn error_is_copy() {
    fn take_copy<T: Copy>(_: T) {}
    let err = scan_token("\u{2603}", Position::new()).unwrap_err();
    take_copy(err);
    let _ = err.position();
    let _ = err.resume_position();
}

// ============================================================
// Tokenizer finish/text tests
// ============================================================

#[test]
fn tokenizer_finish() {
    let src = String::from("hello world");
    let tokenizer = Tokenizer::new(src);
    let recovered: String = tokenizer.finish();
    assert_eq!(recovered, "hello world");
}

#[test]
fn tokenizer_text() {
    let src = "hello world";
    let tokenizer = Tokenizer::new(src);
    assert_eq!(tokenizer.text(), "hello world");
}

// ============================================================
// Real-world Erlang patterns
// ============================================================

#[test]
fn tokenize_gen_server_callback() {
    let src = "handle_call(Request, From, State) ->\n    {reply, ok, State}.";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "handle_call");
    assert!(tokens.contains(&"->".to_string()));
    assert!(tokens.contains(&"{".to_string()));
    assert!(tokens.contains(&"}".to_string()));
}

#[test]
fn tokenize_if_expression() {
    let src = "if X > 0 -> positive; true -> non_positive end";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "if");
    assert!(tokens.contains(&"true".to_string()));
    assert!(tokens.last().unwrap() == "end");
}

#[test]
fn tokenize_attribute() {
    let src = "-behaviour(gen_server).";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "-");
    assert_eq!(tokens[1], "behaviour");
}

#[test]
fn tokenize_include() {
    let src = "-include(\"header.hrl\").";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "-");
    assert_eq!(tokens[1], "include");
    assert!(tokens.contains(&"\"header.hrl\"".to_string()));
}

#[test]
fn tokenize_record_definition() {
    let src = "-record(person, {name, age = 0}).";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"person".to_string()));
    assert!(tokens.contains(&"name".to_string()));
    assert!(tokens.contains(&"age".to_string()));
    assert!(tokens.contains(&"=".to_string()));
    assert!(tokens.contains(&"0".to_string()));
}

#[test]
fn tokenize_map_comprehension() {
    let src = "#{K => V || K := V <- Map}";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"#".to_string()));
    assert!(tokens.contains(&"=>".to_string()));
    assert!(tokens.contains(&"||".to_string()));
    assert!(tokens.contains(&":=".to_string()));
    assert!(tokens.contains(&"<-".to_string()));
}

#[test]
fn tokenize_binary_comprehension() {
    let src = "<< <<X:8>> || <<X:8>> <= Bin, X > 0 >>";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "<<");
    assert!(tokens.contains(&"<=".to_string()));
    assert!(tokens.last().unwrap() == ">>");
}

#[test]
fn tokenize_doc_attribute() {
    let src = "-doc \"This is a doc string\".";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "-");
    assert_eq!(tokens[1], "doc");
    assert!(tokens.contains(&"\"This is a doc string\"".to_string()));
}

#[test]
fn tokenize_begin_end() {
    let src = "begin X = 1, Y = 2, X + Y end";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "begin");
    assert!(tokens.last().unwrap() == "end");
}

#[test]
fn tokenize_logical_ops() {
    let src = "X andalso Y orelse Z";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"andalso".to_string()));
    assert!(tokens.contains(&"orelse".to_string()));
}

#[test]
fn tokenize_integer_div_rem() {
    let src = "X div Y rem Z";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"div".to_string()));
    assert!(tokens.contains(&"rem".to_string()));
}

#[test]
fn tokenize_pipe_in_list() {
    let src = "[H | T]";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"|".to_string()));
}

#[test]
fn tokenize_fun_reference() {
    let src = "fun foo/2";
    let tokens = tokenize!(src);
    assert_eq!(tokens[0], "fun");
    assert!(tokens.contains(&"foo".to_string()));
    assert!(tokens.contains(&"/".to_string()));
    assert!(tokens.contains(&"2".to_string()));
}

#[test]
fn tokenize_string_escape_sequences_in_context() {
    let src = r#"io:format("~p\n", [X])"#;
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"io".to_string()));
    assert!(tokens.contains(&":".to_string()));
    assert!(tokens.contains(&"format".to_string()));
}

#[test]
fn tokenize_nested_tuples() {
    let src = "{{a, {b, c}}, {d, e}}";
    let tokens = tokenize!(src);
    assert_eq!(tokens.iter().filter(|t| *t == "{").count(), 4);
    assert_eq!(tokens.iter().filter(|t| *t == "}").count(), 4);
}

#[test]
fn tokenize_multiline_string_context() {
    let src = "X = \"line1\\nline2\".";
    let tokens = tokenize!(src);
    assert!(tokens.contains(&"\"line1\\nline2\"".to_string()));
}

// ============================================================
// Tokenize real OTP-style source
// ============================================================

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
    let tokens: Vec<_> = Tokenizer::new(src).collect();
    for t in &tokens {
        assert!(t.is_ok(), "unexpected error: {t:?}");
    }
    assert!(tokens.len() > 100);
}
