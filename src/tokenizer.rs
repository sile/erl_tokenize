use std::path::Path;

use crate::{Position, PositionRange, Result, Token};

/// Tokenizer.
///
/// This is an iterator which tokenizes Erlang source code and iterates on the resulting tokens.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Tokenizer;
///
/// let src = r#"io:format("Hello")."#;
/// let tokens = Tokenizer::new(src).collect::<Result<Vec<_>, _>>().unwrap();
///
/// assert_eq!(tokens.iter().map(|t| t.text()).collect::<Vec<_>>(),
///            ["io", ":", "format", "(", r#""Hello""#, ")", "."]);
/// ```
#[derive(Debug)]
pub struct Tokenizer<T> {
    text: T,
    next_pos: Position,
}
impl<T> Tokenizer<T>
where
    T: AsRef<str>,
{
    /// Makes a new `Tokenizer` instance which tokenize the Erlang source code text.
    pub fn new(text: T) -> Self {
        let init_pos = Position::new();
        Tokenizer {
            text,
            next_pos: init_pos,
        }
    }

    /// Sets the file path of the succeeding tokens.
    pub fn set_filepath<P: AsRef<Path>>(&mut self, filepath: P) {
        self.next_pos.set_filepath(filepath);
    }

    /// Returns the input text.
    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    /// Finishes tokenization and returns the target text.
    pub fn finish(self) -> T {
        self.text
    }

    /// Returns the cursor position from which this tokenizer will start to scan the next token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Tokenizer;
    ///
    /// let src = r#"io:format(
    ///   "Hello")."#;
    ///
    /// let mut tokenizer = Tokenizer::new(src);
    /// assert_eq!(tokenizer.next_position().offset(), 0);
    ///
    /// assert_eq!(tokenizer.next().unwrap().map(|t| t.text().to_owned()).unwrap(), "io");
    /// assert_eq!(tokenizer.next_position().offset(), 2);
    /// tokenizer.next(); // ':'
    /// tokenizer.next(); // 'format'
    /// tokenizer.next(); // '('
    /// tokenizer.next(); // '\n'
    /// assert_eq!(tokenizer.next_position().offset(), 11);
    /// assert_eq!(tokenizer.next_position().line(), 2);
    /// assert_eq!(tokenizer.next_position().column(), 1);
    /// assert_eq!(tokenizer.next().unwrap().map(|t| t.text().to_owned()).unwrap(), " ");
    /// assert_eq!(tokenizer.next_position().offset(), 12);
    /// assert_eq!(tokenizer.next_position().line(), 2);
    /// assert_eq!(tokenizer.next_position().column(), 2);
    /// ```
    pub fn next_position(&self) -> Position {
        self.next_pos.clone()
    }

    /// Sets the current position.
    ///
    /// Note that it's the responsibility of the user to specify a valid position.
    /// Otherwise, the following tokenization process will raise an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Tokenizer;
    ///
    /// let src = r#"io:format(
    ///   "Hello")."#;
    ///
    /// let mut tokenizer = Tokenizer::new(src);
    /// assert_eq!(tokenizer.next_position().offset(), 0);
    ///
    /// assert_eq!(tokenizer.next().unwrap().map(|t| t.text().to_owned()).unwrap(), "io");
    ///
    /// let position = tokenizer.next_position();
    /// assert_eq!(tokenizer.next().unwrap().map(|t| t.text().to_owned()).unwrap(), ":");
    /// tokenizer.next(); // 'format'
    /// tokenizer.next(); // '('
    /// tokenizer.next(); // '\n'
    ///
    /// tokenizer.set_position(position);
    /// assert_eq!(tokenizer.next().unwrap().map(|t| t.text().to_owned()).unwrap(), ":");
    /// ```
    pub fn set_position(&mut self, position: Position) {
        self.next_pos = position;
    }

    /// Consumes the next char.
    ///
    /// This method can be used to recover from a tokenization error.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Tokenizer;
    ///
    /// let src = r#"io:format("Hello")."#;
    ///
    /// let mut tokenizer = Tokenizer::new(src);
    /// assert_eq!(tokenizer.next_position().offset(), 0);
    ///
    /// tokenizer.consume_char();
    /// assert_eq!(tokenizer.next_position().offset(), 1);
    /// ```
    pub fn consume_char(&mut self) -> Option<char> {
        if let Some(c) = self.text.as_ref()[self.next_pos.offset()..].chars().next() {
            self.next_pos = self.next_pos.clone().step_by_char(c);
            Some(c)
        } else {
            None
        }
    }
}
impl<T> Iterator for Tokenizer<T>
where
    T: AsRef<str>,
{
    type Item = Result<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_pos.offset() >= self.text.as_ref().len() {
            None
        } else {
            let text = unsafe {
                self.text
                    .as_ref()
                    .get_unchecked(self.next_pos.offset()..self.text.as_ref().len())
            };
            let cur_pos = self.next_pos.clone();
            match Token::from_text(text, cur_pos) {
                Err(e) => Some(Err(e)),
                Ok(t) => {
                    self.next_pos = t.end_position();
                    Some(Ok(t))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://github.com/sile/erlls/issues/5
    //
    // v0.8.1 caused the following error:
    // ```
    // thread 'tokenizer::tests::erlls_issue_5' panicked at src/tokenizer.rs:133:44:
    // byte index 32 is not a char boundary; it is inside '应' (bytes 31..34) of `-module(repro).
    // -moduledoc """
    // 应该报错
    // "".`
    // ```
    #[test]
    fn erlls_issue_5() {
        let text = r#"-module(repro).
-moduledoc """
应该报错
""."#;
        let mut tokenizer = Tokenizer::new(text);
        while let Some(token) = tokenizer.next() {
            let Ok(_token) = token else {
                tokenizer.consume_char();
                continue;
            };
        }
    }
}
