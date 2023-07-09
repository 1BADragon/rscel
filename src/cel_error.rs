#[derive(Debug)]
pub struct CelError {
    msg: String,
}

impl CelError {
    pub fn with_msg(msg: &str) -> CelError {
        CelError {
            msg: msg.to_owned(),
        }
    }

    pub fn msg<'a>(&'a self) -> &'a str {
        return &self.msg;
    }

    pub fn into_string(self) -> String {
        self.msg
    }
}

pub type CelResult<T> = Result<T, CelError>;
