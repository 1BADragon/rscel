use serde::{Deserialize, Serialize};
use std::fmt;

use super::source_location::SourceLocation;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyntaxError {
    loc: SourceLocation,

    message: Option<String>,
}

impl SyntaxError {
    pub fn from_location(loc: SourceLocation) -> SyntaxError {
        SyntaxError { loc, message: None }
    }

    pub fn with_message(mut self, msg: String) -> SyntaxError {
        self.message = Some(msg);
        self
    }

    pub fn message<'a>(&'a self) -> Option<&'a str> {
        self.message.as_deref()
    }

    pub fn loc(&self) -> SourceLocation {
        self.loc
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: line {}, column {}",
            self.message().unwrap_or("SYNTAX ERROR"),
            self.loc().line(),
            self.loc().col()
        )
    }
}
