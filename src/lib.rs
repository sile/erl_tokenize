//! Erlang source code tokenizer.
//!
//! The public entry point is the free function [`scan_token`]:
//! given the whole source string and the current [`Position`], it returns
//! the next [`Token`] or [`None`] when the end of the source is reached.
//! On failure, the returned [`Error`] carries a diagnostic position and a
//! resume position that can be passed straight back into [`scan_token`]
//! so a bad token never spins in place.
//!
//! # Examples
//!
//! Tokenize the Erlang code `io:format("Hello").`:
//!
//! ```
//! use erl_tokenize::{Position, scan_token};
//!
//! let src = r#"io:format("Hello")."#;
//! let mut pos = Position::new();
//! let mut texts = Vec::new();
//! while let Some(token) = scan_token(src, pos).unwrap() {
//!     texts.push(token.text(src).to_owned());
//!     pos = token.end();
//! }
//! assert_eq!(texts, ["io", ":", "format", "(", r#""Hello""#, ")", "."]);
//! ```
//!
//! # References
//!
//! - [`erl_scan`][erl_scan] module
//! - [Erlang Data Types][Data Types]
//!
//! [erl_scan]: http://erlang.org/doc/man/erl_scan.html
//! [Data Types]: http://erlang.org/doc/reference_manual/data_types.html
#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub use crate::error::Error;
pub use crate::keyword::Keyword;
pub use crate::position::Position;
pub use crate::symbol::Symbol;
pub use crate::token::{Token, TokenKind, TokenValue, scan_token};

mod error;
mod keyword;
mod lex;
mod position;
mod symbol;
mod token;
mod util;

/// This crate's `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;
