mod eval;
mod program_cache;
mod program_details;
mod program_error;
use std::sync::Arc;

use parsel::FromStr;
// Re-export
pub use eval::eval_expr;
pub use program_error::ProgramError;

pub use program_details::ProgramDetails;

use crate::{ast::grammar::Expr, value_cell::ValueCell, CelContext};

pub type ProgramResult<T> = Result<T, ProgramError>;

#[derive(Debug)]
pub struct Program {
    source: String,
    details: Arc<program_details::ProgramDetails>,

    ast: Arc<Expr>,
}

impl Program {
    pub fn from_source(source: &str) -> ProgramResult<Program> {
        match program_cache::check_cache(source) {
            Some(prog) => prog,
            None => Program::from_source_nocache(source),
        }
    }

    pub fn from_source_nocache(source: &str) -> ProgramResult<Program> {
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
            details: Arc::new(ProgramDetails::from_ast(&ast)),
            ast: Arc::new(ast),
        })
    }

    pub fn params<'a>(&'a self) -> Vec<&'a str> {
        self.details.params()
    }

    pub fn source<'a>(&'a self) -> &'a str {
        &self.source
    }

    pub fn details(&self) -> ProgramDetails {
        self.details.as_ref().clone()
    }

    pub fn eval(&self, ctx: &CelContext) -> ProgramResult<ValueCell> {
        match eval_expr(&self.ast, ctx) {
            Ok(val) => Ok(val),
            Err(err) => Err(ProgramError::new(err.msg())),
        }
    }
}

impl Clone for Program {
    fn clone(&self) -> Self {
        Program {
            source: self.source.clone(),
            details: self.details.clone(),
            ast: self.ast.clone(),
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
