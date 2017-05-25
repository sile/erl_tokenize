//! Erlang source code tokenizer.
//!
//! # Examples
//!
//! ```text
//! use erl_tokenize::Tokenizer;
//! use erl_tokenize::tokens::{Atom, Symbol, Str};
//!
//! let src = r#"io:format("Hello")."#;
//! let tokenizer = Tokenizer::new(src.chars());
//! let tokens = tokenizer.collect::<Result<Vec<_>, _>>().unwrap();
//! assert_eq!(tokens,
//!            [Atom("io".into()).into(), Symbol::Colon.into(), Atom("format".into()).into(),
//!             Symbol::OpenParen.into(), Str("Hello".into()).into(), Symbol::CloseParen.into(),
//!             Symbol::Dot.into()]);
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
pub use token::Token;
// pub use tokenizer::Tokenizer;

pub mod tokens;
pub mod types;

// mod char_reader;
mod error;
mod misc;
mod token;
// mod tokenizer;

pub type Result<T> = ::std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
