erl_tokenize
============

[![erl_tokenize](https://img.shields.io/crates/v/erl_tokenize.svg)](https://crates.io/crates/erl_tokenize)
[![Documentation](https://docs.rs/erl_tokenize/badge.svg)](https://docs.rs/erl_tokenize)
[![Actions Status](https://github.com/sile/erl_tokenize/workflows/CI/badge.svg)](https://github.com/sile/erl_tokenize/actions)
![License](https://img.shields.io/crates/l/erl_tokenize)

Erlang source code tokenizer written in Rust.

[Documentation](https://docs.rs/erl_tokenize)

How it works
------------

You give the tokenizer a source string and a current position, and it
returns the next token. You then advance the position to the end of
that token and ask again. When you reach the end of the source, you get
`Ok(None)` and stop.

The tokens themselves are lightweight: they know their kind (atom,
integer, string, and so on) and where they sit in the source, but they
do not hold onto a copy of the source text or a decoded value. If you
want the token's text, you ask for it with the source you already have
in your hand; if you want the decoded value, you ask for that
separately. Nothing is copied or allocated until you ask.

Positions are pure `line:column` (plus a byte offset). They deliberately
carry no file name, so when you report an error you attach the file name
yourself.

Examples
--------

Tokenize the Erlang code `io:format("Hello").`:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = r#"io:format("Hello")."#;
    let mut position = erl_tokenize::Position::new();
    let mut texts = Vec::new();
    while let Some(token) = erl_tokenize::scan_token(src, position)? {
        texts.push(token.text(src));
        position = token.end();
    }
    assert_eq!(texts, ["io", ":", "format", "(", r#""Hello""#, ")", "."]);
    Ok(())
}
```

Comments and whitespace are returned as ordinary tokens. Skip them on
the caller side with `Token::is_lexical`:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = "%% greeting\nhello world";
    let mut position = erl_tokenize::Position::new();
    let mut lexical = Vec::new();
    while let Some(token) = erl_tokenize::scan_token(src, position)? {
        if token.is_lexical() {
            lexical.push(token.text(src));
        }
        position = token.end();
    }
    assert_eq!(lexical, ["hello", "world"]);
    Ok(())
}
```

Recover from a lexical error by resuming at `Error::resume_position`:

```rust
fn main() {
    let src = "\u{2603} foo";
    let mut position = erl_tokenize::Position::new();
    let mut texts = Vec::new();
    loop {
        match erl_tokenize::scan_token(src, position) {
            Ok(Some(token)) => {
                texts.push(token.text(src));
                position = token.end();
            }
            Ok(None) => break,
            Err(error) => {
                eprintln!("skipping at {}: {}", error.position(), error);
                position = error.resume_position();
            }
        }
    }
    assert_eq!(texts, [" ", "foo"]);
}
```

Extract only the string values with `Token::value`. Because `TokenValue`
is a single enum, matching on the variant is all that is needed; no
kind-specific downcast method is required:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = r#"io:format("Hello", ["World"])."#;
    let mut position = erl_tokenize::Position::new();
    let mut strings = Vec::new();
    while let Some(token) = erl_tokenize::scan_token(src, position)? {
        if let erl_tokenize::TokenValue::String(text) = token.value(src) {
            strings.push(text.into_owned());
        }
        position = token.end();
    }
    assert_eq!(strings, ["Hello", "World"]);
    Ok(())
}
```

Executes the example `tokenize` command:

```bash
$ cargo run --example tokenize -- /dev/stdin <<EOS
-module(foo).

-export([bar/0]).

bar() -> qux.
EOS

[1:1] "-"
[1:2] "module"
[1:8] "("
[1:9] "foo"
[1:12] ")"
[1:13] "."
[1:14] "\n"
[2:1] "\n"
[3:1] "-"
[3:2] "export"
[3:8] "("
[3:9] "["
[3:10] "bar"
[3:13] "/"
[3:14] "0"
[3:15] "]"
[3:16] ")"
[3:17] "."
[3:18] "\n"
[4:1] "\n"
[5:1] "bar"
[5:4] "("
[5:5] ")"
[5:6] " "
[5:7] "->"
[5:9] " "
[5:10] "qux"
[5:13] "."
[5:14] "\n"
TOKEN COUNT: 29
ELAPSED: 0.000166 seconds
```

`Position` displays as `line:column` and does not carry a file path; the
example above pairs each token with the caller-supplied input file name
only in its own error output, not through the `Position` value itself.

The `\n` characters on lines that contain nothing else appear as
standalone whitespace tokens because at most one line feed is allowed at
the start of a whitespace token; a run that would otherwise pack two
consecutive line feeds is split in two.

References
----------

- [erl_scan](http://erlang.org/doc/man/erl_scan.html) module
- [Erlang Data Types](http://erlang.org/doc/reference_manual/data_types.html)
