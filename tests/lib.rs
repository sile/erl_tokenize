extern crate erl_tokenize;
#[macro_use]
extern crate trackable;

use erl_tokenize::{Tokenizer, Result, Token};
use erl_tokenize::tokens::{self, Symbol};

fn nl() -> Token {
    Token::from(tokens::Whitespace::Newline)
}
fn space() -> Token {
    Token::from(tokens::Whitespace::Space)
}
fn comment(s: &str) -> Token {
    Token::from(tokens::Comment(s.into()))
}
fn atom(s: &str) -> Token {
    Token::from(tokens::Atom(s.into()))
}
fn var(s: &str) -> Token {
    Token::from(tokens::Var(s.into()))
}
fn int(n: u64) -> Token {
    Token::from(tokens::Int(n.into()))
}
fn string(s: &str) -> Token {
    Token::from(tokens::Str(s.into()))
}
fn ch(c: char) -> Token {
    Token::from(tokens::Char(c))
}
fn float(n: f64) -> Token {
    Token::from(tokens::Float(n.into()))
}

fn tokenize(s: &str) -> Result<Vec<Token>> {
    Tokenizer::new(s.chars()).collect()
}

#[test]
fn tokenize_comments() {
    let src = "% foo";
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens, [comment(" foo")]);

    let src = r#"
% foo
 % bar
"#;
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens,
               [nl(), comment(" foo"), nl(), space(), comment(" bar"), nl()]);
}

#[test]
fn tokenize_numbers() {
    let src = "10 1.02";
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens, [int(10), space(), float(1.02)]);
}

#[test]
fn tokenize_atoms() {
    let src = "foo 'BAR' comté";
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens,
               [atom("foo"), space(), atom("BAR"), space(), atom("comté")]);
}

#[test]
fn tokenize_variables() {
    let src = "Foo BAR _ _Baz";
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens,
               [var("Foo"),
                space(),
                var("BAR"),
                space(),
                var("_"),
                space(),
                var("_Baz")]);
}

#[test]
fn tokenize_strings() {
    let src = r#""foo" "b\tar""#;
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens, [string("foo"), space(), string("b\tar")]);
}

#[test]
fn tokenize_chars() {
    let src = r#"$a $\t"#;
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens, [ch('a'), space(), ch('\t')]);
}

#[test]
fn tokenize_module_declaration() {
    let src = "-module(foo).";
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens,
               [Symbol::Hyphen.into(),
                atom("module"),
                Symbol::OpenParen.into(),
                atom("foo"),
                Symbol::CloseParen.into(),
                Symbol::Dot.into()]);
}
