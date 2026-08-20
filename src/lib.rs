//! Erlang source code tokenizer.
//!
//! The public entry point is the free function [`scan_token`]: given the
//! whole source string and the current [`Position`], it returns the next
//! [`Token`], or `Ok(None)` when the end of the source is reached. On
//! failure, the returned [`Error`] carries a diagnostic position and a
//! resume position that can be passed straight back into [`scan_token`]
//! so a bad token never spins in place.
//!
//! # Design
//!
//! - One call scans one token.
//! - A [`Token`] does not borrow the source and does not carry any
//!   decoded value. Value extraction happens only when the caller
//!   invokes [`Token::value`].
//! - The caller owns the source string and drives the current position
//!   from token to token.
//! - Comments and whitespace are returned as ordinary tokens. Callers
//!   that only care about grammatical tokens filter them out via
//!   [`TokenKind::is_lexical`] or [`TokenKind::is_hidden`].
//! - [`Position`] does not carry a file path. Associating a scanned
//!   source with its file name or buffer identifier is the caller's
//!   responsibility.
//!
//! # Examples
//!
//! Tokenize the Erlang code `io:format("Hello").`:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let src = r#"io:format("Hello")."#;
//! let mut position = erl_tokenize::Position::new();
//! let mut texts = Vec::new();
//! while let Some(token) = erl_tokenize::scan_token(src, position)? {
//!     texts.push(token.text(src));
//!     position = token.end();
//! }
//! assert_eq!(texts, ["io", ":", "format", "(", r#""Hello""#, ")", "."]);
//! # Ok(())
//! # }
//! ```
//!
//! Skip comments and whitespace on the caller side:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let src = "%% greeting\nhello world";
//! let mut position = erl_tokenize::Position::new();
//! let mut lexical = Vec::new();
//! while let Some(token) = erl_tokenize::scan_token(src, position)? {
//!     if token.kind().is_lexical() {
//!         lexical.push(token.text(src));
//!     }
//!     position = token.end();
//! }
//! assert_eq!(lexical, ["hello", "world"]);
//! # Ok(())
//! # }
//! ```
//!
//! Resume after a lexical error using [`Error::resume_position`]:
//!
//! ```
//! let src = "\u{2603} foo";
//! let mut position = erl_tokenize::Position::new();
//! let mut texts = Vec::new();
//! loop {
//!     match erl_tokenize::scan_token(src, position) {
//!         Ok(Some(token)) => {
//!             texts.push(token.text(src));
//!             position = token.end();
//!         }
//!         Ok(None) => break,
//!         Err(error) => {
//!             position = error.resume_position;
//!         }
//!     }
//! }
//! assert_eq!(texts, [" ", "foo"]);
//! ```
//!
//! # Compatibility target
//!
//! This crate aims to match the behavior of Erlang/OTP's
//! [`erl_scan`][erl_scan] module at the [`OTP-29.0.5`][OTP-29.0.5] tag.
//! CI runs the token diff against that tag's stdlib source.
//!
//! # References
//!
//! - [`erl_scan`][erl_scan] module
//! - [Erlang Data Types][Data Types]
//!
//! [erl_scan]: http://erlang.org/doc/man/erl_scan.html
//! [Data Types]: http://erlang.org/doc/reference_manual/data_types.html
//! [OTP-29.0.5]: https://github.com/erlang/otp/tree/OTP-29.0.5
#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub use crate::error::{Error, ErrorKind};
pub use crate::keyword::Keyword;
pub use crate::position::Position;
pub use crate::symbol::Symbol;
pub use crate::token::{Token, TokenKind, TokenValue, scan_token};

mod charset;
mod error;
mod escape;
mod keyword;
mod lex;
mod lex_atom;
mod lex_char;
mod lex_comment;
mod lex_float;
mod lex_integer;
mod lex_sigil;
mod lex_string;
mod lex_symbol;
mod lex_variable;
mod lex_whitespace;
mod position;
mod symbol;
mod token;

/// This crate's `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;
