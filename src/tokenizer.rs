use {Result, Token};

#[derive(Debug)]
pub struct Tokenizer<'a> {
    text: &'a str,
    start: usize,
}
impl<'a> Tokenizer<'a> {
    pub fn new(text: &'a str) -> Self {
        Tokenizer { text, start: 0 }
    }
}
impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.text.len() {
            None
        } else {
            let text = unsafe { self.text.slice_unchecked(self.start, self.text.len()) };
            match track!(Token::from_text(text)) {
                Err(e) => {
                    self.start = self.text.len();
                    Some(Err(e))
                }
                Ok(t) => {
                    self.start += t.text().len();
                    Some(Ok(t))
                }
            }
        }
    }
}
