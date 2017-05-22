extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use token::Token;
pub use tokenizer::Tokenizer;

pub mod tokens;

mod char_reader;
mod error;
mod token;
mod tokenizer;

pub type Result<T> = ::std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
