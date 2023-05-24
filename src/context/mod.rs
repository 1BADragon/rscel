use std::collections::HashMap;

mod default_funcs;
mod default_macros;
mod exec_context;
mod utils;
use crate::{
    ast::grammar::Expr,
    program::{eval_expr, Program, ProgramDetails, ProgramResult},
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
    ValueCellInner,
};

pub use exec_context::{ExecContext, RsCellFunction, RsCellMacro};

/// The CelContext is the core context in RsCel. This context contains
/// Program information as well as the primary entry point for evaluating
/// an expression.
pub struct CelContext<'a> {
    progs: HashMap<String, Program>,

    current_ctx: Option<&'a ExecContext>,
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
}

/// Result wrapper with ExecError as the error type
pub type ExecResult<T> = Result<T, ExecError>;

impl<'a> CelContext<'a> {
    /// Constructs a new empty CelContext
    pub fn new() -> CelContext<'a> {
        CelContext {
            progs: HashMap::new(),
            current_ctx: None,
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
    pub fn program_details(&self, name: &str) -> Option<ProgramDetails> {
        let prog = self.progs.get(name)?;

        Some(prog.details())
    }

    pub(crate) fn get_param_by_name<'l: 'a>(&'l self, name: &str) -> Option<&'l ValueCell> {
        let value = self.current_ctx?.get_param(name)?;

        Some(value)
    }

    pub(crate) fn get_func_by_name<'l: 'a>(&'l self, name: &str) -> Option<&'l RsCellFunction> {
        self.current_ctx?.get_func(name)
    }

    pub(crate) fn get_macro_by_name<'l: 'a>(&'l self, name: &str) -> Option<&'l RsCellMacro> {
        self.current_ctx?.get_macro(name)
    }

    pub(crate) fn exec_context(&self) -> Option<ExecContext> {
        Some((*self.current_ctx?).clone())
    }

    pub(crate) fn resolve_fqn(&self, fqn: &[ValueCell]) -> ValueCellResult<ValueCell> {
        let mut iter = fqn.iter();

        let mut current = if let Some(vc) = iter.next() {
            match vc.inner() {
                ValueCellInner::Ident(ident) => match self.get_param_by_name(ident) {
                    Some(val) => val.clone(),
                    None => {
                        return Err(ValueCellError::with_msg(&format!(
                            "Ident '{}' does not exist",
                            ident
                        )))
                    }
                },
                other => other.clone().into(),
            }
        } else {
            return Err(ValueCellError::with_msg("Empty Ident"));
        };

        for member_name in iter {
            match current.inner() {
                ValueCellInner::Map(map) => {
                    if let ValueCellInner::Ident(member_name_str) = member_name.inner() {
                        current = if let Some(member) = map.get(member_name_str) {
                            member.clone()
                        } else {
                            return Err(ValueCellError::with_msg(&format!(
                                "member {} does not exist on {:?}",
                                member_name_str, &current
                            )));
                        }
                    } else {
                        return Err(ValueCellError::with_msg(
                            "Only idents can be member accesses",
                        ));
                    }
                }
                _ => {
                    return Err(ValueCellError::with_msg(&format!(
                        "member access invalid on {:?}",
                        current
                    )))
                }
            }
        }

        return Ok(current);
    }

    /// Evaluate a Program with the given name with a provided ExecContext. A single CelContext
    /// can be run multiple times with different ExecContext's. The return of this function is
    /// a Result with either a ValueCell representing the final solution of the Program or an Error
    /// that is discovered during execution, such as mismatch of types
    pub fn exec<'l: 'a>(&'l mut self, name: &str, ctx: &'l ExecContext) -> ExecResult<ValueCell> {
        self.current_ctx = Some(ctx);

        let res = match self.progs.get(name) {
            Some(prog) => match prog.eval(self) {
                Ok(res) => Ok(res),
                Err(err) => Err(ExecError::from_str(err.into_str())),
            },
            None => Err(ExecError::new(&format!("Program {} does not exist", name))),
        };

        self.current_ctx = None;
        return res;
    }

    pub(crate) fn eval_expr<'l: 'a>(
        &'l mut self,
        expr: &Expr,
        ctx: &'l ExecContext,
    ) -> ValueCellResult<ValueCell> {
        self.current_ctx = Some(ctx);

        let res = eval_expr(expr, self);

        self.current_ctx = None;
        return res;
    }
}

impl<'a> Clone for CelContext<'a> {
    fn clone(&self) -> Self {
        CelContext {
            progs: self.progs.clone(),
            current_ctx: None,
        }
    }
}

#[cfg(test)]
mod test {

    use super::{CelContext, ExecContext};

    #[test]
    fn test_eval_basic() {
        let mut ctx = CelContext::new();
        let exec_ctx = ExecContext::new();

        ctx.add_program_str("test_main", "3 + 4").unwrap();

        let res = ctx.exec("test_main", &exec_ctx).unwrap();

        assert!(TryInto::<i64>::try_into(res).unwrap() == 7)
    }
}
