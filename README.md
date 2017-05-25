erl_tokenize
============

[![erl_tokenize](http://meritbadge.herokuapp.com/erl_tokenize)](https://crates.io/crates/erl_tokenize)
[![Documentation](https://docs.rs/erl_tokenize/badge.svg)](https://docs.rs/erl_tokenize)
[![Build Status](https://travis-ci.org/sile/erl_tokenize.svg?branch=master)](https://travis-ci.org/sile/erl_tokenize)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Erlang source code tokenizer written in Rust.

[Documentation](https://docs.rs/erl_tokenize)

Examples
--------

Tokenizes the Erlang code `io:format("Hello").`:

```rust
use erl_tokenize::{Tokenizer, TokenKind};

let src = r#"io:format("Hello")."#;
let tokenizer = Tokenizer::new(src);
let tokens = tokenizer.collect::<Result<Vec<_>, _>>().unwrap();

assert_eq!(tokens.iter().map(|t| t.kind()).collect::<Vec<_>>(),
           [TokenKind::Atom, TokenKind::Symbol, TokenKind::Atom, TokenKind::Symbol,
            TokenKind::String, TokenKind::Symbol, TokenKind::Symbol]);

assert_eq!(tokens.iter().map(|t| t.text()).collect::<Vec<_>>(),
           ["io", ":", "format", "(", r#""Hello""#, ")", "."]);
```

Executes the example `tokenize` command:

```bash
$ cargo run --example tokenize -- /dev/stdin <<EOS
-module(foo).

-export([bar/0]).

bar() -> qux.
EOS

[line:1] Symbol(SymbolToken { value: Hyphen, text: "-" })
[line:1] Atom(AtomToken { value: "module", text: "module" })
[line:1] Symbol(SymbolToken { value: OpenParen, text: "(" })
[line:1] Atom(AtomToken { value: "foo", text: "foo" })
[line:1] Symbol(SymbolToken { value: CloseParen, text: ")" })
[line:1] Symbol(SymbolToken { value: Dot, text: "." })
[line:1] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
[line:2] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
[line:3] Symbol(SymbolToken { value: Hyphen, text: "-" })
[line:3] Atom(AtomToken { value: "export", text: "export" })
[line:3] Symbol(SymbolToken { value: OpenParen, text: "(" })
[line:3] Symbol(SymbolToken { value: OpenSquare, text: "[" })
[line:3] Atom(AtomToken { value: "bar", text: "bar" })
[line:3] Symbol(SymbolToken { value: Slash, text: "/" })
[line:3] Integer(IntegerToken { value: BigUint { data: [] }, text: "0" })
[line:3] Symbol(SymbolToken { value: CloseSquare, text: "]" })
[line:3] Symbol(SymbolToken { value: CloseParen, text: ")" })
[line:3] Symbol(SymbolToken { value: Dot, text: "." })
[line:3] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
[line:4] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
[line:5] Atom(AtomToken { value: "bar", text: "bar" })
[line:5] Symbol(SymbolToken { value: OpenParen, text: "(" })
[line:5] Symbol(SymbolToken { value: CloseParen, text: ")" })
[line:5] Whitespace(WhitespaceToken { value: Space, text: " " })
[line:5] Symbol(SymbolToken { value: RightAllow, text: "->" })
[line:5] Whitespace(WhitespaceToken { value: Space, text: " " })
[line:5] Atom(AtomToken { value: "qux", text: "qux" })
[line:5] Symbol(SymbolToken { value: Dot, text: "." })
[line:5] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
```

References
----------

- [erl_scan](http://erlang.org/doc/man/erl_scan.html) module
- [Erlang Data Types](http://erlang.org/doc/reference_manual/data_types.html)
