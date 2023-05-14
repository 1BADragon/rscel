use std::collections::HashMap;

mod default_funcs;
mod exec_context;
use crate::{
    program::{Program, ProgramDetails, ProgramResult},
    value_cell::{ValueCell, ValueCellError, ValueCellResult},
};

pub use exec_context::ExecContext;
use serde_json::Value;

use self::exec_context::RsCellCallback;

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

    pub fn get_param_by_name<'l: 'a>(&'l self, name: &str) -> Option<&'l Value> {
        let json_value = self.current_ctx?.param(name)?;

        Some(json_value)
    }

    pub fn get_func_by_name<'l: 'a>(&'l self, name: &str) -> Option<&'l RsCellCallback> {
        self.current_ctx?.func(name)
    }

    pub fn resolve_fqn(&self, fqn: &[ValueCell]) -> ValueCellResult<ValueCell> {
        let mut iter = fqn.iter();
        let mut current = match iter.next() {
            Some(ValueCell::Ident(ident)) => match self.get_param_by_name(ident) {
                Some(val) => ValueCell::from(val),
                None => {
                    return Err(ValueCellError::with_msg(&format!(
                        "Ident '{}' does not exist",
                        ident
                    )))
                }
            },
            Some(other) => other.clone(),
            None => return Err(ValueCellError::with_msg("Empty Ident")),
        };

        for member_name in iter {
            match &current {
                ValueCell::Map(map) => {
                    if let ValueCell::Ident(member_name_str) = member_name {
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
