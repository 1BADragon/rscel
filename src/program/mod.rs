mod compile;
mod program_cache;
mod program_details;
mod program_error;

use crate::interp::ByteCode;
use compile::ProgramCompiler;
pub use program_details::ProgramDetails;
pub use program_error::ProgramError;
use serde::{Deserialize, Serialize};

pub type ProgramResult<T> = Result<T, ProgramError>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Program {
    source: String,
    details: program_details::ProgramDetails,

    bytecode: Vec<ByteCode>,
}

impl Program {
    pub fn from_source(source: &str) -> ProgramResult<Program> {
        match program_cache::check_cache(source) {
            Some(prog) => prog,
            None => Program::from_source_nocache(source),
        }
    }

    pub fn from_source_nocache(source: &str) -> ProgramResult<Program> {
        ProgramCompiler::new().with_source(source).build()
    }

    pub fn params<'a>(&'a self) -> Vec<&'a str> {
        self.details.params()
    }

    pub fn source<'a>(&'a self) -> &'a str {
        &self.source
    }

    pub fn details<'a>(&'a self) -> &'a ProgramDetails {
        &self.details
    }

    pub fn bytecode<'a>(&'a self) -> &'a [ByteCode] {
        &self.bytecode
    }

    pub fn dumps_bc(&self) -> String {
        let mut lines = Vec::new();

        for code in self.bytecode.iter() {
            lines.push(format!("{:?}", code))
        }

        lines.join("\n")
    }
}

impl Clone for Program {
    fn clone(&self) -> Self {
        Program {
            source: self.source.clone(),
            details: self.details.clone(),
            bytecode: self.bytecode.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Program;

    #[test]
    fn test_basic_prog() {
        let prog = Program::from_source("foo + 3").unwrap();

        assert!(prog.params().len() == 1);
        assert!(prog.params()[0] == "foo");
    }

    #[test]
    fn test_complex_prog() {
        let prog = Program::from_source("((foo.bar + 2) * foo.baz) / bam").unwrap();

        assert!(prog.params().len() == 2);
    }
}
