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

[pos:Position { offset: 0, line: 1, column: 1 }] Symbol(SymbolToken { value: Hyphen, text: "-" })
[pos:Position { offset: 1, line: 1, column: 2 }] Atom(AtomToken { value: "module", text: "module" })
[pos:Position { offset: 7, line: 1, column: 8 }] Symbol(SymbolToken { value: OpenParen, text: "(" })
[pos:Position { offset: 8, line: 1, column: 9 }] Atom(AtomToken { value: "foo", text: "foo" })
[pos:Position { offset: 11, line: 1, column: 12 }] Symbol(SymbolToken { value: CloseParen, text: ")" })
[pos:Position { offset: 12, line: 1, column: 13 }] Symbol(SymbolToken { value: Dot, text: "." })
[pos:Position { offset: 13, line: 1, column: 14 }] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
[pos:Position { offset: 14, line: 2, column: 1 }] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
[pos:Position { offset: 15, line: 3, column: 1 }] Symbol(SymbolToken { value: Hyphen, text: "-" })
[pos:Position { offset: 16, line: 3, column: 2 }] Atom(AtomToken { value: "export", text: "export" })
[pos:Position { offset: 22, line: 3, column: 8 }] Symbol(SymbolToken { value: OpenParen, text: "(" })
[pos:Position { offset: 23, line: 3, column: 9 }] Symbol(SymbolToken { value: OpenSquare, text: "[" })
[pos:Position { offset: 24, line: 3, column: 10 }] Atom(AtomToken { value: "bar", text: "bar" })
[pos:Position { offset: 27, line: 3, column: 13 }] Symbol(SymbolToken { value: Slash, text: "/" })
[pos:Position { offset: 28, line: 3, column: 14 }] Integer(IntegerToken { value: BigUint { data: [] }, text: "0" })
[pos:Position { offset: 29, line: 3, column: 15 }] Symbol(SymbolToken { value: CloseSquare, text: "]" })
[pos:Position { offset: 30, line: 3, column: 16 }] Symbol(SymbolToken { value: CloseParen, text: ")" })
[pos:Position { offset: 31, line: 3, column: 17 }] Symbol(SymbolToken { value: Dot, text: "." })
[pos:Position { offset: 32, line: 3, column: 18 }] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
[pos:Position { offset: 33, line: 4, column: 1 }] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
[pos:Position { offset: 34, line: 5, column: 1 }] Atom(AtomToken { value: "bar", text: "bar" })
[pos:Position { offset: 37, line: 5, column: 4 }] Symbol(SymbolToken { value: OpenParen, text: "(" })
[pos:Position { offset: 38, line: 5, column: 5 }] Symbol(SymbolToken { value: CloseParen, text: ")" })
[pos:Position { offset: 39, line: 5, column: 6 }] Whitespace(WhitespaceToken { value: Space, text: " " })
[pos:Position { offset: 40, line: 5, column: 7 }] Symbol(SymbolToken { value: RightArrow, text: "->" })
[pos:Position { offset: 42, line: 5, column: 9 }] Whitespace(WhitespaceToken { value: Space, text: " " })
[pos:Position { offset: 43, line: 5, column: 10 }] Atom(AtomToken { value: "qux", text: "qux" })
[pos:Position { offset: 46, line: 5, column: 13 }] Symbol(SymbolToken { value: Dot, text: "." })
[pos:Position { offset: 47, line: 5, column: 14 }] Whitespace(WhitespaceToken { value: Newline, text: "\n" })
```

References
----------

- [erl_scan](http://erlang.org/doc/man/erl_scan.html) module
- [Erlang Data Types](http://erlang.org/doc/reference_manual/data_types.html)
