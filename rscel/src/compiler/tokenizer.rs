pub use super::syntax_error::SyntaxError;
use super::tokens::Token;

#[derive(Debug, PartialEq, Clone)]
pub struct TokenWithLoc {
    token: Token,
    start: (usize, usize),
    end: (usize, usize),
}

pub trait Tokenizer {
    fn peek(&mut self) -> Result<Option<TokenWithLoc>, SyntaxError>;
    fn next(&mut self) -> Result<Option<TokenWithLoc>, SyntaxError>;

    fn source<'a>(&'a self) -> &'a str;
    fn location(&self) -> (usize, usize);
}

impl TokenWithLoc {
    pub fn new(token: Token, start: (usize, usize), end: (usize, usize)) -> Self {
        TokenWithLoc { token, start, end }
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn start(&self) -> (usize, usize) {
        self.start
    }

    pub fn end(&self) -> (usize, usize) {
        self.end
    }
}
