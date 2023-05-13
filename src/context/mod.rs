use std::collections::HashMap;

mod exec_context;
use crate::{
    program::{Program, ProgramDetails, ProgramResult},
    value_cell::ValueCell,
};

pub use exec_context::ExecContext;

pub struct CelContext<'a> {
    progs: HashMap<String, Program>,

    current_ctx: Option<&'a ExecContext>,
}

#[derive(Debug)]
pub struct ExecError {
    msg: String,
}

impl ExecError {
    pub fn new(msg: &str) -> ExecError {
        ExecError {
            msg: msg.to_owned(),
        }
    }

    pub fn from_str(msg: String) -> ExecError {
        ExecError { msg }
    }

    pub fn str<'a>(&'a self) -> &'a str {
        &self.msg
    }
}

type ExecResult<T> = Result<T, ExecError>;

impl<'a> CelContext<'a> {
    pub fn new() -> CelContext<'a> {
        CelContext {
            progs: HashMap::new(),
            current_ctx: None,
        }
    }

    pub fn add_program(&mut self, name: &str, prog: Program) {
        self.progs.insert(name.to_owned(), prog);
    }

    pub fn add_program_str(&mut self, name: &str, prog_str: &str) -> ProgramResult<()> {
        let prog = Program::from_source(prog_str)?;

        self.add_program(name, prog);
        Ok(())
    }

    pub fn program_details(&self, name: &str) -> Option<ProgramDetails> {
        let prog = self.progs.get(name)?;

        Some(prog.details())
    }

    pub fn get_param_by_name(&self, name: &str) -> Option<ValueCell> {
        let json_value = self.current_ctx?.param(name)?;

        Some(ValueCell::from_ident(json_value))
    }

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
}

#[cfg(test)]
mod test {
    use crate::value_cell::ValueCell;

    use super::{CelContext, ExecContext};

    #[test]
    fn test_eval_basic() {
        let mut ctx = CelContext::new();
        let exec_ctx = ExecContext::new();

        ctx.add_program_str("test_main", "3 + 4").unwrap();

        let res = ctx.exec("test_main", &exec_ctx).unwrap();

        println!("{:?}", res);

        if let ValueCell::Int(val) = res {
            assert!(val == 7);
        } else {
            assert!(false);
        }
    }
}
