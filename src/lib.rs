//! Erlang source code tokenizer.
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
pub use crate::error::Error;
pub use crate::position::{Position, PositionRange};
pub use crate::token::{Token, TokenKind, scan_token};
pub use crate::tokenizer::Tokenizer;
pub use crate::values::{Keyword, Symbol, Whitespace};

pub mod tokens;
pub mod values;

mod error;
mod lex;
mod position;
mod token;
mod tokenizer;
mod util;

/// This crate specific `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;
