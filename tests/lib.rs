use erl_tokenize::tokens::{
    AtomToken, CharToken, CommentToken, FloatToken, IntegerToken, KeywordToken, SigilStringToken,
    StringToken, SymbolToken, VariableToken, WhitespaceToken,
};
use erl_tokenize::values::{Keyword, Symbol, Whitespace};
use erl_tokenize::{Lexer, Position, PositionRange, Token, Tokenizer};

macro_rules! tokenize {
    ($text:expr) => {
        Tokenizer::new($text)
            .map(|t| t.unwrap().text().to_string())
            .collect::<Vec<_>>()
    };
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
}

#[test]
fn atom_vs_keyword() {
    let token = Token::from_text("case", pos()).unwrap();
    assert!(token.as_keyword_token().is_some());

    let token = Token::from_text("case_x", pos()).unwrap();
    assert!(token.as_atom_token().is_some());

    let token = Token::from_text("foo", pos()).unwrap();
    assert!(token.as_atom_token().is_some());
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
    let t = CharToken::from_text(r"$\^a", pos()).unwrap();
    assert_eq!(t.value(), '\u{1}');

    let t = CharToken::from_text(r"$\^A", pos()).unwrap();
    assert_eq!(t.value(), '\u{1}');

    let t = CharToken::from_text(r"$\^z", pos()).unwrap();
    assert_eq!(t.value(), '\u{1A}');

    let t = CharToken::from_text(r"$\^]", pos()).unwrap();
    assert_eq!(t.value(), '\u{1D}');

    let t = CharToken::from_text(r"$\^?", pos()).unwrap();
    assert_eq!(t.value(), '\u{1F}');
}

#[test]
fn char_octal() {
    let t = CharToken::from_text(r"$\123", pos()).unwrap();
    assert_eq!(t.value(), 'S'); // 0o123 = 83 = 'S'

    let t = CharToken::from_text(r"$\17", pos()).unwrap();
    assert_eq!(t.value() as u32, 15); // 0o17 = 15

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
    let tokens: Vec<String> = Tokenizer::new(r"$\1234")
        .map(|t| t.unwrap().text().to_string())
        .collect();
    assert_eq!(tokens, [r"$\123", "4"]);
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
}

#[test]
fn char_from_value() {
    let t = CharToken::from_value('a', pos());
    assert_eq!(t.text(), "$a");
    assert_eq!(t.value(), 'a');

    let t = CharToken::from_value('\\', pos());
    assert_eq!(t.text(), r"$\\");
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
    let tokens: Vec<String> = Tokenizer::new("")
        .map(|t| t.unwrap().text().to_string())
        .collect();
    assert!(tokens.is_empty());
}

// ============================================================
// Lexer tests
// ============================================================

#[test]
fn lexer_filters_whitespace() {
    let src = "foo bar";
    let tokens: Vec<String> = Lexer::new(src)
        .map(|t| t.unwrap().text().to_owned())
        .collect();
    assert_eq!(tokens, ["foo", "bar"]);
}

#[test]
fn lexer_filters_comments() {
    let src = "foo % comment\nbar";
    let tokens: Vec<String> = Lexer::new(src)
        .map(|t| t.unwrap().text().to_owned())
        .collect();
    assert_eq!(tokens, ["foo", "bar"]);
}

#[test]
fn lexer_complex() {
    let src = "-module(foo).\n\n-export([bar/0]).\n\nbar() -> ok.";
    let tokens: Vec<String> = Lexer::new(src)
        .map(|t| t.unwrap().text().to_owned())
        .collect();
    assert_eq!(
        tokens,
        [
            "-", "module", "(", "foo", ")", ".", "-", "export", "(", "[", "bar", "/", "0", "]",
            ")", ".", "bar", "(", ")", "->", "ok", "."
        ]
    );
}

#[test]
fn lexer_finish() {
    let src = String::from("hello");
    let lexer = Lexer::new(src);
    let recovered: String = lexer.finish();
    assert_eq!(recovered, "hello");
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
    assert_eq!(t.text(), "bar");
}

#[test]
fn position_consume_char() {
    let src = "abc";
    let mut tokenizer = Tokenizer::new(src);

    assert_eq!(tokenizer.consume_char(), Some('a'));
    assert_eq!(tokenizer.next_position().offset(), 1);

    assert_eq!(tokenizer.consume_char(), Some('b'));
    assert_eq!(tokenizer.next_position().offset(), 2);

    assert_eq!(tokenizer.consume_char(), Some('c'));
    assert_eq!(tokenizer.next_position().offset(), 3);

    assert_eq!(tokenizer.consume_char(), None);
}

#[test]
fn position_filepath() {
    let src = "foo";
    let mut tokenizer = Tokenizer::new(src);
    tokenizer.set_filepath("test.erl");

    let t = tokenizer.next().unwrap().unwrap();
    let p = t.start_position();
    assert_eq!(p.filepath().unwrap().to_str().unwrap(), "test.erl");
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
// Token conversion tests
// ============================================================

#[test]
fn token_is_lexical_or_hidden() {
    let atom = Token::from_text("foo", pos()).unwrap();
    assert!(atom.is_lexical_token());
    assert!(!atom.is_hidden_token());

    let ws = Token::from_text(" ", pos()).unwrap();
    assert!(!ws.is_lexical_token());
    assert!(ws.is_hidden_token());

    let comment = Token::from_text("% test", pos()).unwrap();
    assert!(!comment.is_lexical_token());
    assert!(comment.is_hidden_token());
}

#[test]
fn token_into_lexical() {
    let atom = Token::from_text("foo", pos()).unwrap();
    assert!(atom.into_lexical_token().is_ok());

    let ws = Token::from_text(" ", pos()).unwrap();
    assert!(ws.into_lexical_token().is_err());
}

#[test]
fn token_into_hidden() {
    let ws = Token::from_text(" ", pos()).unwrap();
    assert!(ws.into_hidden_token().is_ok());

    let comment = Token::from_text("% test", pos()).unwrap();
    assert!(comment.into_hidden_token().is_ok());

    let atom = Token::from_text("foo", pos()).unwrap();
    assert!(atom.into_hidden_token().is_err());
}

#[test]
fn token_as_accessors() {
    let t = Token::from_text("foo", pos()).unwrap();
    assert!(t.as_atom_token().is_some());
    assert!(t.as_char_token().is_none());

    let t = Token::from_text("$a", pos()).unwrap();
    assert!(t.as_char_token().is_some());

    let t = Token::from_text("42", pos()).unwrap();
    assert!(t.as_integer_token().is_some());

    let t = Token::from_text("1.5", pos()).unwrap();
    assert!(t.as_float_token().is_some());

    let t = Token::from_text("case", pos()).unwrap();
    assert!(t.as_keyword_token().is_some());

    let t = Token::from_text(r#""hello""#, pos()).unwrap();
    assert!(t.as_string_token().is_some());

    let t = Token::from_text(".", pos()).unwrap();
    assert!(t.as_symbol_token().is_some());

    let t = Token::from_text("Foo", pos()).unwrap();
    assert!(t.as_variable_token().is_some());

    let t = Token::from_text("% comment", pos()).unwrap();
    assert!(t.as_comment_token().is_some());

    let t = Token::from_text(" ", pos()).unwrap();
    assert!(t.as_whitespace_token().is_some());
}

#[test]
fn token_into_specific() {
    let t = Token::from_text("foo", pos()).unwrap();
    assert!(t.into_atom_token().is_ok());

    let t = Token::from_text("$a", pos()).unwrap();
    assert!(t.into_char_token().is_ok());

    let t = Token::from_text("42", pos()).unwrap();
    assert!(t.into_integer_token().is_ok());

    let t = Token::from_text("1.5", pos()).unwrap();
    assert!(t.into_float_token().is_ok());

    let t = Token::from_text("case", pos()).unwrap();
    assert!(t.into_keyword_token().is_ok());

    let t = Token::from_text(r#""hello""#, pos()).unwrap();
    assert!(t.into_string_token().is_ok());

    let t = Token::from_text(".", pos()).unwrap();
    assert!(t.into_symbol_token().is_ok());

    let t = Token::from_text("Foo", pos()).unwrap();
    assert!(t.into_variable_token().is_ok());

    let t = Token::from_text("% comment", pos()).unwrap();
    assert!(t.into_comment_token().is_ok());

    let t = Token::from_text(" ", pos()).unwrap();
    assert!(t.into_whitespace_token().is_ok());
}

// ============================================================
// Display / text round-trip tests
// ============================================================

#[test]
fn display_matches_text() {
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
        let t = Token::from_text(src, pos()).unwrap();
        assert_eq!(format!("{t}"), t.text(), "Display mismatch for: {src}");
    }
}

// ============================================================
// Error recovery tests
// ============================================================

#[test]
fn error_recovery_with_consume_char() {
    let text = "-module(repro).\n-moduledoc \"\"\"\n\u{5e94}\u{8be5}\u{62a5}\u{9519}\n\"\".";
    let mut tokenizer = Tokenizer::new(text);
    let mut token_count = 0;
    while let Some(token) = tokenizer.next() {
        match token {
            Ok(_) => token_count += 1,
            Err(_) => {
                tokenizer.consume_char();
            }
        }
    }
    assert!(token_count > 0);
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
