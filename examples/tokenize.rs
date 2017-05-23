extern crate clap;
extern crate erl_tokenize;
#[macro_use]
extern crate trackable;

use std::fs::File;
use std::io::Read;
use clap::{App, Arg};
use erl_tokenize::{Tokenizer, Token};
use erl_tokenize::tokens::Whitespace;

fn main() {
    let matches = App::new("tokenize")
        .arg(Arg::with_name("SOURCE_FILE").index(1).required(true))
        .get_matches();
    let src_file = matches.value_of("SOURCE_FILE").unwrap();

    let mut src = String::new();
    let mut file = File::open(src_file).expect("Cannot open file");
    file.read_to_string(&mut src).expect("Cannot read file");

    let mut line = 1;
    let tokenizer = Tokenizer::new(src.chars());
    for token in tokenizer {
        let token = track_try_unwrap!(token, "line={}", line);
        if token == Token::Whitespace(Whitespace::Newline) {
            line += 1;
        }
        println!("[line:{}] {:?}", line, token);
    }
}
