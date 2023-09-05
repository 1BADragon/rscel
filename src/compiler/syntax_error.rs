use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct SyntaxError {
    pub line: usize,
    pub column: usize,

    message: Option<String>,
}

impl SyntaxError {
    pub fn from_location(loc: (usize, usize)) -> SyntaxError {
        SyntaxError {
            line: loc.0,
            column: loc.1,
            message: None,
        }
    }

    pub fn with_message(mut self, msg: String) -> SyntaxError {
        self.message = Some(msg);
        self
    }

    pub fn message<'a>(&'a self) -> Option<&'a str> {
        self.message.as_deref()
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}
