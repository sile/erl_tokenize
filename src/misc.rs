use std::borrow::Cow;
use std::str::Chars;

use Result;

pub fn is_atom_head_char(c: char) -> bool {
    if let 'a'...'z' = c { true } else { false }
}

pub fn is_atom_non_head_char(c: char) -> bool {
    match c {
        '@' | '_' | '0'...'9' => true,
        _ => c.is_alphabetic(),
    }
}

pub fn parse_string(input: &str, terminator: char) -> Result<(Cow<str>, usize)> {
    panic!()
}

pub struct StringParser<'a> {
    chars: Chars<'a>,
    buf: String,
}
impl<'a> StringParser<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        StringParser {
            chars,
            buf: String::new(),
        }
    }
    pub fn parse(self) -> Result<String> {
        panic!()
    }
}
