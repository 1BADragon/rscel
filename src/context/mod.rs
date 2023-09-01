use std::collections::HashMap;

mod bind_context;
mod default_funcs;
mod default_macros;
use crate::{
    cel_error::CelResult,
    compiler::{compiler::CelCompiler, string_tokenizer::StringTokenizer},
    interp::Interpreter,
    program::{Program, ProgramDetails},
    CelValue,
};
pub use bind_context::{BindContext, RsCelFunction, RsCelMacro};

/// The CelContext is the core context in RsCel. This context contains
/// Program information as well as the primary entry point for evaluating
/// an expression.
pub struct CelContext {
    progs: HashMap<String, Program>,
}

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
    pub fn add_program_str(&mut self, name: &str, prog_str: &str) -> CelResult<()> {
        let mut tokenizer = StringTokenizer::with_input(prog_str);
        let prog = CelCompiler::with_tokenizer(&mut tokenizer).compile()?;

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
    pub fn exec<'l>(&'l mut self, name: &str, bindings: &'l BindContext) -> CelResult<CelValue> {
        let interp = Interpreter::new(&self, bindings);

        interp.run_program(name)
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
