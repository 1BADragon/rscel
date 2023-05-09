use std::str::FromStr;

#[derive(Debug)]
pub struct ProgramError {
    msg: String,
}

impl ProgramError {
    pub fn new(msg: &str) -> ProgramError {
        ProgramError {
            msg: String::from_str(msg).unwrap(),
        }
    }

    pub fn msg<'a>(&'a self) -> &'a str {
        &self.msg
    }
}
