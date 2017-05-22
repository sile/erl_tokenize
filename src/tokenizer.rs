use std::iter::Peekable;

use types::Location;

#[derive(Debug)]
pub struct Tokenizer<T>
    where T: Iterator<Item = char>
{
    chars: Peekable<T>,
    current: Location,
}
impl<T> Tokenizer<T>
    where T: Iterator<Item = char>
{
    pub fn new(chars: T) -> Self {
        Tokenizer {
            chars: chars.peekable(),
            current: Location { line: 0, column: 0 },
        }
    }
}
