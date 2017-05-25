//! Erlang source code tokenizer.
//!
//! # Examples
//!
//! ```
//! use erl_tokenize::{Tokenizer, TokenKind};
//!
//! let src = r#"io:format("Hello")."#;
//! let tokenizer = Tokenizer::new(src);
//! let tokens = tokenizer.collect::<Result<Vec<_>, _>>().unwrap();
//!
//! assert_eq!(tokens.iter().map(|t| t.kind()).collect::<Vec<_>>(),
//!            [TokenKind::Atom, TokenKind::Symbol, TokenKind::Atom, TokenKind::Symbol,
//!             TokenKind::String, TokenKind::Symbol, TokenKind::Symbol]);
//!
//! assert_eq!(tokens.iter().map(|t| t.text()).collect::<Vec<_>>(),
//!            ["io", ":", "format", "(", r#""Hello""#, ")", "."]);
//! ```
//!
//! # References
//!
//! - [erl_scan][erl_scan] module
//! - [Erlang Data Types][Data Types]
//!
//! [erl_scan]: http://erlang.org/doc/man/erl_scan.html
//! [Data Types]: http://erlang.org/doc/reference_manual/data_types.html
extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use token::{Token, TokenKind};
pub use tokenizer::Tokenizer;

pub mod tokens;
pub mod types;

mod error;
mod token;
mod tokenizer;
mod util;

pub type Result<T> = ::std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
