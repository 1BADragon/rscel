mod context;
mod parser;
mod program;
mod value_cell;

pub use context::{CelContext, ExecContext, ExecError};
pub use program::{Program, ProgramError};

pub use serde;
pub use serde_json;

#[cfg(test)]
mod test {
    use crate::{CelContext, ExecContext};

    #[test]
    fn test_bad_func_call() {
        let mut ctx = CelContext::new();
        let exec_ctx = ExecContext::new();

        ctx.add_program_str("main", "foo(3)").unwrap();

        let res = ctx.exec("main", &exec_ctx);
        assert!(res.is_err());
    }
}
