pub use super::syntax_error::SyntaxError;
use super::tokens::Token;

pub trait Tokenizer {
    fn peek(&mut self) -> Result<Option<Token>, SyntaxError>;
    fn next(&mut self) -> Result<Option<Token>, SyntaxError>;

    fn source<'a>(&'a self) -> &'a str;
    fn location(&self) -> (usize, usize);
}
