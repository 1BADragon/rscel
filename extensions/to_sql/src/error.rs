#[derive(Debug)]
pub enum ToSqlError {
    Unsupported(String),
}

pub type ToSqlResult<T> = Result<T, ToSqlError>;

impl ToSqlError {
    pub fn unsupported(error_msg: &str) -> Self {
        ToSqlError::Unsupported(error_msg.to_owned())
    }
}

impl std::fmt::Display for ToSqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToSqlError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
        }
    }
}
