erl_tokenize
============

[![erl_tokenize](https://img.shields.io/crates/v/erl_tokenize.svg)](https://crates.io/crates/erl_tokenize)
[![Documentation](https://docs.rs/erl_tokenize/badge.svg)](https://docs.rs/erl_tokenize)
[![Actions Status](https://github.com/sile/erl_tokenize/workflows/CI/badge.svg)](https://github.com/sile/erl_tokenize/actions)
[![Coverage Status](https://coveralls.io/repos/github/sile/erl_tokenize/badge.svg?branch=master)](https://coveralls.io/github/sile/erl_tokenize?branch=master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Erlang source code tokenizer written in Rust.

[Documentation](https://docs.rs/erl_tokenize)

Examples
--------

Tokenizes the Erlang code `io:format("Hello").`:

```rust
use erl_tokenize::Tokenizer;

let src = r#"io:format("Hello")."#;
let tokenizer = Tokenizer::new(src);
let tokens = tokenizer.collect::<Result<Vec<_>, _>>().unwrap();

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

[Position { filepath: None, offset: 0, line: 1, column: 1 }] Symbol(Hyphen)
[Position { filepath: None, offset: 1, line: 1, column: 2 }] Atom("module")
[Position { filepath: None, offset: 7, line: 1, column: 8 }] Symbol(OpenParen)
[Position { filepath: None, offset: 8, line: 1, column: 9 }] Atom("foo")
[Position { filepath: None, offset: 11, line: 1, column: 12 }] Symbol(CloseParen)
[Position { filepath: None, offset: 12, line: 1, column: 13 }] Symbol(Dot)
[Position { filepath: None, offset: 13, line: 1, column: 14 }] Whitespace(Newline)
[Position { filepath: None, offset: 14, line: 2, column: 1 }] Whitespace(Newline)
[Position { filepath: None, offset: 15, line: 3, column: 1 }] Symbol(Hyphen)
[Position { filepath: None, offset: 16, line: 3, column: 2 }] Atom("export")
[Position { filepath: None, offset: 22, line: 3, column: 8 }] Symbol(OpenParen)
[Position { filepath: None, offset: 23, line: 3, column: 9 }] Symbol(OpenSquare)
[Position { filepath: None, offset: 24, line: 3, column: 10 }] Atom("bar")
[Position { filepath: None, offset: 27, line: 3, column: 13 }] Symbol(Slash)
[Position { filepath: None, offset: 28, line: 3, column: 14 }] Integer(BigUint { data: [] })
[Position { filepath: None, offset: 29, line: 3, column: 15 }] Symbol(CloseSquare)
[Position { filepath: None, offset: 30, line: 3, column: 16 }] Symbol(CloseParen)
[Position { filepath: None, offset: 31, line: 3, column: 17 }] Symbol(Dot)
[Position { filepath: None, offset: 32, line: 3, column: 18 }] Whitespace(Newline)
[Position { filepath: None, offset: 33, line: 4, column: 1 }] Whitespace(Newline)
[Position { filepath: None, offset: 34, line: 5, column: 1 }] Atom("bar")
[Position { filepath: None, offset: 37, line: 5, column: 4 }] Symbol(OpenParen)
[Position { filepath: None, offset: 38, line: 5, column: 5 }] Symbol(CloseParen)
[Position { filepath: None, offset: 39, line: 5, column: 6 }] Whitespace(Space)
[Position { filepath: None, offset: 40, line: 5, column: 7 }] Symbol(RightArrow)
[Position { filepath: None, offset: 42, line: 5, column: 9 }] Whitespace(Space)
[Position { filepath: None, offset: 43, line: 5, column: 10 }] Atom("qux")
[Position { filepath: None, offset: 46, line: 5, column: 13 }] Symbol(Dot)
[Position { filepath: None, offset: 47, line: 5, column: 14 }] Whitespace(Newline)
TOKEN COUNT: 29
ELAPSED: 0.007222 seconds
```

References
----------

- [erl_scan](http://erlang.org/doc/man/erl_scan.html) module
- [Erlang Data Types](http://erlang.org/doc/reference_manual/data_types.html)
