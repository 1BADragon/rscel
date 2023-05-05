use core::fmt;

use crate::compiler::lexer::{ParseError, Token};

pub struct SemanticError {
    parse_token: Option<Token>,
    parse_error: Option<ParseError>,

    message: String,
}

impl SemanticError {
    pub fn new(parse_token: Option<Token>, parse_error: Option<ParseError>, message: &str) -> Self {
        Self {
            parse_token,
            parse_error,
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Semantic Error Encountered")
    }
}
