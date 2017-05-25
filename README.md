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

References
----------

- [erl_scan][erl_scan] module
- [Erlang Data Types][Data Types]
