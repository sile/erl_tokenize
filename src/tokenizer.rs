use {Result, Token};

/// Tokenizer.
///
/// This is an iterator which tokenizes Erlang source code and iterates on the resulting tokens.
///
/// # Examples
///
/// ```
/// use erl_tokenize::{Tokenizer, TokenKind};
///
/// let src = r#"io:format("Hello")."#;
/// let tokens = Tokenizer::new(src).collect::<Result<Vec<_>, _>>().unwrap();
///
/// assert_eq!(tokens.iter().map(|t| t.text()).collect::<Vec<_>>(),
///            ["io", ":", "format", "(", r#""Hello""#, ")", "."]);
/// ```
#[derive(Debug)]
pub struct Tokenizer<'a> {
    text: &'a str,
    position: usize,
}
impl<'a> Tokenizer<'a> {
    /// Makes a new `Tokenizer` instance which tokenize the Erlang source code text.
    pub fn new(text: &'a str) -> Self {
        Tokenizer { text, position: 0 }
    }

    /// Returns the input text.
    pub fn text(&self) -> &'a str {
        self.text
    }

    /// Returns the cursor position from which this tokenizer will start to scan the next token.
    ///
    /// # Examples
    ///
    /// ```
    /// use erl_tokenize::Tokenizer;
    ///
    /// let src = r#"io:format("Hello")."#;
    ///
    /// let mut tokenizer = Tokenizer::new(src);
    /// assert_eq!(tokenizer.position(), 0);
    ///
    /// assert_eq!(tokenizer.next().unwrap().unwrap().text(), "io");
    /// assert_eq!(tokenizer.position(), 2);
    /// ```
    pub fn position(&self) -> usize {
        self.position
    }
}
impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.text.len() {
            None
        } else {
            let text = unsafe { self.text.slice_unchecked(self.position, self.text.len()) };
            match track!(Token::from_text(text)) {
                Err(e) => {
                    self.position = self.text.len();
                    Some(Err(e))
                }
                Ok(t) => {
                    self.position += t.text().len();
                    Some(Ok(t))
                }
            }
        }
    }
}
