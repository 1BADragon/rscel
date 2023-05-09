use crate::parser::Expr;
use std::str::FromStr;

mod program_details;
mod program_error;

// Re-export
pub use program_error::ProgramError;

use program_details::ProgramDetails;

pub type ProgramResult<T> = Result<T, ProgramError>;

pub struct Program {
    source: String,
    details: program_details::ProgramDetails,

    ast: Expr,
}

impl Program {
    pub fn from_source(source: &str) -> ProgramResult<Program> {
        let ast: Expr = match parsel::parse_str(source) {
            Ok(expr) => expr,
            Err(err) => {
                let span = err.span();
                return Err(ProgramError::new(&format!(
                    "Error on {}:{} ending at {}:{}",
                    span.start().line,
                    span.start().column,
                    span.end().line,
                    span.end().column
                )));
            }
        };

        Ok(Program {
            source: String::from_str(source).unwrap(),
            details: ProgramDetails::from_ast(&ast),
            ast,
        })
    }

    pub fn params<'a>(&'a self) -> Vec<&'a str> {
        self.details.params()
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
