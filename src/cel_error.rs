use std::fmt;

#[derive(Debug)]
pub enum CelError {
    Misc(String),
}
pub type CelResult<T> = Result<T, CelError>;

impl CelError {
    pub fn misc(msg: &str) -> CelError {
        CelError::Misc(msg.to_owned())
    }

    pub fn type_string(&self) -> &'static str {
        use CelError::*;

        match self {
            Misc(_msg) => "MISC",
        }
    }
}

impl fmt::Display for CelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CelError::*;

        match self {
            Misc(msg) => write!(f, "{}", msg),
        }
    }
}
