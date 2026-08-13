use crate::{Position, Result, Token, scan_token};

/// Convenience iterator that yields tokens from a source string by
/// repeatedly calling [`scan_token`].
///
/// Errors advance the cursor to the resume position carried in the error
/// (see [`Error::resume_position`](crate::Error::resume_position)), so a
/// bad token does not halt the iterator or spin at the same position.
///
/// # Examples
///
/// ```
/// use erl_tokenize::Tokenizer;
///
/// let src = r#"io:format("Hello")."#;
/// let mut tokenizer = Tokenizer::new(src);
/// let tokens = tokenizer.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
/// let texts: Vec<_> = tokens.iter().map(|t| t.text(src)).collect();
/// assert_eq!(texts, ["io", ":", "format", "(", r#""Hello""#, ")", "."]);
/// ```
#[derive(Debug, Clone)]
pub struct Tokenizer<T> {
    text: T,
    next_pos: Position,
}

impl<T> Tokenizer<T>
where
    T: AsRef<str>,
{
    /// Makes a new `Tokenizer` instance that will tokenize `text` from
    /// the start.
    pub fn new(text: T) -> Self {
        Tokenizer {
            text,
            next_pos: Position::new(),
        }
    }

    /// Returns the input text.
    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    /// Finishes tokenization and returns the target text.
    pub fn finish(self) -> T {
        self.text
    }

    /// Returns the cursor position from which this tokenizer will scan
    /// the next token.
    pub fn next_position(&self) -> Position {
        self.next_pos
    }

    /// Sets the cursor position for the next scan.
    ///
    /// It is the caller's responsibility to specify a position that lies
    /// on a UTF-8 boundary of the input text.
    pub fn set_position(&mut self, position: Position) {
        self.next_pos = position;
    }
}

impl<T> Iterator for Tokenizer<T>
where
    T: AsRef<str>,
{
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match scan_token(self.text.as_ref(), self.next_pos) {
            Ok(Some(token)) => {
                self.next_pos = token.end();
                Some(Ok(token))
            }
            Ok(None) => None,
            Err(err) => {
                self.next_pos = err.resume_position();
                Some(Err(err))
            }
        }
    }
}
