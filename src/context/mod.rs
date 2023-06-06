use std::collections::HashMap;

mod bind_context;
mod default_funcs;
mod default_macros;
use crate::{
    interp::Interpreter,
    program::{Program, ProgramDetails, ProgramResult},
    value_cell::ValueCell,
};
pub use bind_context::{BindContext, RsCallable, RsCellFunction, RsCellMacro};

/// The CelContext is the core context in RsCel. This context contains
/// Program information as well as the primary entry point for evaluating
/// an expression.
pub struct CelContext {
    progs: HashMap<String, Program>,
}

/// ExecError is the error type returned by CelContext operations.
#[derive(Debug)]
pub struct ExecError {
    msg: String,
}

impl ExecError {
    /// Constructs a new ExecError with a given message.
    pub fn new(msg: &str) -> ExecError {
        ExecError {
            msg: msg.to_owned(),
        }
    }

    /// No-copy construction of ExecError
    pub fn from_str(msg: String) -> ExecError {
        ExecError { msg }
    }

    /// Error message contained in the ExecError
    pub fn str<'a>(&'a self) -> &'a str {
        &self.msg
    }

    /// Drop the error returning the underlying error message
    pub fn into_str(self) -> String {
        self.msg
    }
}

/// Result wrapper with ExecError as the error type
pub type ExecResult<T> = Result<T, ExecError>;

impl CelContext {
    /// Constructs a new empty CelContext
    pub fn new() -> CelContext {
        CelContext {
            progs: HashMap::new(),
        }
    }

    /// Add an already constructed Program to the context with a given name. Using
    /// This method can allow a Program to be constructed once and shared between
    /// contexts, if desired. Will override an existing program with same name.
    pub fn add_program(&mut self, name: &str, prog: Program) {
        self.progs.insert(name.to_owned(), prog);
    }

    /// Add a Program to the context with the given name and source string. Return of this
    /// function indicates parseing result of the constructed Program. This method will not
    /// allow for a Program to be shared. Will override an existing program with same name.
    pub fn add_program_str(&mut self, name: &str, prog_str: &str) -> ProgramResult<()> {
        let prog = Program::from_source(prog_str)?;

        self.add_program(name, prog);
        Ok(())
    }

    /// Returns ProgramDetails for a program by name if it exists.
    pub fn program_details<'a>(&'a self, name: &str) -> Option<&'a ProgramDetails> {
        let prog = self.progs.get(name)?;

        Some(prog.details())
    }

    pub fn get_program<'a>(&'a self, name: &str) -> Option<&'a Program> {
        self.progs.get(name)
    }

    /// Evaluate a Program with the given name with a provided ExecContext. A single CelContext
    /// can be run multiple times with different ExecContext's. The return of this function is
    /// a Result with either a ValueCell representing the final solution of the Program or an Error
    /// that is discovered during execution, such as mismatch of types
    pub fn exec<'l>(&'l mut self, name: &str, bindings: &'l BindContext) -> ExecResult<ValueCell> {
        let interp = Interpreter::new(&self, bindings);

        match interp.run_program(name) {
            Ok(good) => Ok(good),
            Err(err) => Err(ExecError::from_str(err.into_string())),
        }
    }

    // pub(crate) fn eval_expr(
    //     &mut self,
    //     expr: &Expr,
    //     ctx: &BindContext,
    // ) -> ValueCellResult<ValueCell> {
    //     self.current_ctx = Some(ctx.clone());

    //     let res = eval_expr(expr, self);

    //     self.current_ctx = None;
    //     return res;
    // }
}

impl Clone for CelContext {
    fn clone(&self) -> Self {
        CelContext {
            progs: self.progs.clone(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::{BindContext, CelContext};

    #[test]
    fn test_eval_basic() {
        let mut ctx = CelContext::new();
        let exec_ctx = BindContext::new();

        ctx.add_program_str("test_main", "3 + 4").unwrap();

        let res = ctx.exec("test_main", &exec_ctx).unwrap();

        assert!(TryInto::<i64>::try_into(res).unwrap() == 7)
    }
}
