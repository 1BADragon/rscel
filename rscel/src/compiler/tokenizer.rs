pub use super::syntax_error::SyntaxError;
use super::{
    source_location::SourceLocation,
    source_range::SourceRange,
    tokens::{AsToken, IntoToken, Token},
};

#[derive(Debug, PartialEq, Clone)]
pub struct TokenWithLoc {
    pub token: Token,
    pub loc: SourceRange,
}

pub trait Tokenizer {
    fn peek(&mut self) -> Result<Option<&TokenWithLoc>, SyntaxError>;
    fn next(&mut self) -> Result<Option<TokenWithLoc>, SyntaxError>;

    fn source<'a>(&'a self) -> &'a str;
    fn location(&self) -> SourceLocation;
}

impl TokenWithLoc {
    pub fn new(token: Token, loc: SourceRange) -> Self {
        TokenWithLoc { token, loc }
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn start(&self) -> SourceLocation {
        self.loc.start()
    }

    pub fn end(&self) -> SourceLocation {
        self.loc.end()
    }

    pub fn into_token(self) -> Token {
        self.token
    }
}

impl AsToken for TokenWithLoc {
    fn as_token(&self) -> Option<&Token> {
        Some(&self.token)
    }
}

impl IntoToken for TokenWithLoc {
    fn into_token(self) -> Option<Token> {
        Some(self.token)
    }
}

impl AsToken for &TokenWithLoc {
    fn as_token(&self) -> Option<&Token> {
        (*self).as_token()
    }
}

impl AsToken for Option<TokenWithLoc> {
    fn as_token(&self) -> Option<&Token> {
        match self {
            Some(s) => s.as_token(),
            None => None,
        }
    }
}

impl IntoToken for Option<TokenWithLoc> {
    fn into_token(self) -> Option<Token> {
        match self {
            Some(t) => Some(t.into_token()),
            None => None,
        }
    }
}

impl AsToken for Option<&TokenWithLoc> {
    fn as_token(&self) -> Option<&Token> {
        match self {
            Some(s) => s.as_token(),
            None => None,
        }
    }
}
