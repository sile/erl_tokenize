use {Result, Token};

#[derive(Debug)]
pub struct Tokenizer<'a> {
    text: &'a str,
    position: usize,
}
impl<'a> Tokenizer<'a> {
    pub fn new(text: &'a str) -> Self {
        Tokenizer { text, position: 0 }
    }
    pub fn text(&self) -> &'a str {
        self.text
    }
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
