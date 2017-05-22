extern crate erl_tokenize;
#[macro_use]
extern crate trackable;

use erl_tokenize::{Tokenizer, Result, Token};
use erl_tokenize::tokens;
use erl_tokenize::types::Location;

macro_rules! comment {
    ($line:expr, $column:expr, $comment:expr) => {
        Token::new(Location{line: $line, column: $column}, tokens::Comment($comment.into()))
    }
}

fn tokenize(s: &str) -> Result<Vec<Token>> {
    Tokenizer::new(s.chars()).collect()
}

#[test]
fn tokenize_comments() {
    let src = "% foo";
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens, [comment!(1, 1, "% foo")]);

    let src = r#"
% foo
  % bar
"#;
    let tokens = track_try_unwrap!(tokenize(src));
    assert_eq!(tokens, [comment!(1, 1, "% foo")]);
}
