extern crate erl_tokenize;
#[macro_use]
extern crate trackable;

use erl_tokenize::{Tokenizer, Result, Token};
use erl_tokenize::tokens;

fn nl() -> Token {
    Token::from(tokens::Whitespace::Newline)
}
fn space() -> Token {
    Token::from(tokens::Whitespace::Space)
}
fn comment(s: &str) -> Token {
    Token::from(tokens::Comment(s.into()))
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
