use erl_tokenize::Tokenizer;

macro_rules! tokenize {
    ($text:expr) => {
        Tokenizer::new($text)
            .map(|t| t.unwrap().text().to_string())
            .collect::<Vec<_>>()
    };
}

macro_rules! is_tokenize_ok {
    ($text:expr) => {
        Tokenizer::new($text)
            .map(|t| t.map(|t| t.text().to_string()))
            .collect::<Result<Vec<_>, _>>()
            .is_ok()
    };
}

#[test]
fn tokenize_comments() {
    let src = "% foo";
    assert_eq!(tokenize!(src), ["% foo"]);

    let src = r#"
% foo
 % bar"#;
    assert_eq!(tokenize!(src), ["\n", "% foo", "\n", " ", "% bar"]);
}

#[test]
fn tokenize_numbers() {
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
fn tokenize_atoms() {
    let src = "foo 'BAR' comté äfunc";
    assert_eq!(
        tokenize!(src),
        ["foo", " ", "'BAR'", " ", "comté", " ", "äfunc"]
    );
}

#[test]
fn tokenize_variables() {
    let src = "Foo BAR _ _Baz";
    assert_eq!(tokenize!(src), ["Foo", " ", "BAR", " ", "_", " ", "_Baz"]);
}

#[test]
fn tokenize_strings() {
    let src = r#""foo" "b\tar""#;
    assert_eq!(tokenize!(src), [r#""foo""#, " ", r#""b\tar""#]);
}

#[test]
fn tokenize_triple_quoted_strings() {
    // OK
    let src = r#""""\nfoo\n""""#;
    assert_eq!(tokenize!(src), [r#""foo""#]);

    let src = r#""""\n foo\n """"#;
    assert_eq!(tokenize!(src), [r#""foo""#]);

    let src = r#"""""\nfoo\n"""""#;
    assert_eq!(tokenize!(src), [r#""foo""#]);

    // NG
    let src = r#""""foo""""#;
    assert_eq!(is_tokenize_ok!(src), false);

    let src = r#""""\nfoo\n """"#;
    assert_eq!(is_tokenize_ok!(src), false);

    let src = r#""""erl\nfoo\n""""#;
    assert_eq!(is_tokenize_ok!(src), false);

    let src = r#""""""#; // Strings concatenation without intervening whitespace
    assert_eq!(is_tokenize_ok!(src), false);
}

#[test]
fn tokenize_chars() {
    let src = r#"$a $\t $\^a $\^]"#;
    assert_eq!(
        tokenize!(src),
        ["$a", " ", r#"$\t"#, " ", r#"$\^a"#, " ", r#"$\^]"#]
    );
}

#[test]
fn tokenize_module_declaration() {
    let src = "-module(foo).";
    assert_eq!(tokenize!(src), ["-", "module", "(", "foo", ")", "."]);
}

#[test]
fn tokenize_multibyte_whitespaces() {
    let src = "a\u{a0}b";
    assert_eq!(tokenize!(src), ["a", "\u{a0}", "b"]);
}
