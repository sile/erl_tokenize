use erl_tokenize::{Token, Tokenizer};

macro_rules! tokenize {
    ($text:expr) => {
        Tokenizer::new($text)
            .map(|t| t.unwrap().text().to_string())
            .collect::<Vec<_>>()
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
    fn tokenize(text: &str) -> Result<String, usize> {
        match Tokenizer::new(text).next() {
            Some(Ok(Token::String(t))) => Ok(t.value().to_owned()),
            Some(Err(e)) => Err(e.position().offset()),
            t => panic!("{t:?}"),
        }
    }

    // OK
    let src = r#""""
foo
""""#;
    assert_eq!(tokenize(src), Ok("foo".to_owned()));

    let src = r#""""
 foo
 """"#;
    assert_eq!(tokenize(src), Ok("foo".to_owned()));

    let src = r#"""""
foo
"""""#;
    assert_eq!(tokenize(src), Ok("foo".to_owned()));

    let src = r#""""
""""#;
    assert_eq!(tokenize(src), Ok("".to_owned()));

    let src = r#""""

""""#;
    assert_eq!(tokenize(src), Ok("".to_owned()));

    let src = r#""""


""""#;
    assert_eq!(tokenize(src), Ok("\n".to_owned()));

    // NG
    let src = r#""""foo""""#;
    assert_eq!(tokenize(src), Err(0));

    let src = r#""""\nfoo\n """"#;
    assert_eq!(tokenize(src), Err(0));

    let src = r#""""erl\nfoo\n""""#;
    assert_eq!(tokenize(src), Err(0));

    let src = r#""a""b""#; // Strings concatenation without intervening whitespace
    assert_eq!(tokenize(src), Err(3));
}

#[test]
fn tokenize_sigils() {
    fn tokenize(text: &str) -> Option<(String, String, String)> {
        if let Some(Ok(Token::SigilString(t))) = Tokenizer::new(text).next() {
            let (prefix, content, suffix) = t.value();
            Some(value(prefix, content, suffix))
        } else {
            None
        }
    }

    fn value(prefix: &str, content: &str, suffix: &str) -> (String, String, String) {
        (prefix.to_owned(), content.to_owned(), suffix.to_owned())
    }

    let src = "~\"\"";
    assert_eq!(tokenize(src), Some(value("", "", "")));

    let src = "~a(b)c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a[b]c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a{b}c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a<b>c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a/b/c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a|b|c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a'b'c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a\"b\"c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a`b`c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a#b#c";
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = r#"~a"""
    b
    """c"#;
    assert_eq!(tokenize(src), Some(value("a", "b", "c")));

    let src = "~a`b`c 10";
    assert_eq!(tokenize!(src), ["~a`b`c", " ", "10"]);
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
