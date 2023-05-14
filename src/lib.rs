//! RsCel is a CEL evaluator written in Rust. CEL is a google project that
//! describes a turing-incomplete language that can be used to evaluate
//! a user provdided expression. The language specification can be found
//! [here](https://github.com/google/cel-spec/blob/master/doc/langdef.md).
//!
//! The design goals of this project were are as follows:
//!   * Flexible enough to allow for a user to bend the spec if needed
//!   * Sandbox'ed in such a way that only specific values can be bound
//!   * Can be used as a wasm depenedency (or other ffi)
//!
//! While Google's CEL spec was designed around the protobuf message,
//! I decided to focus on using the JSON message instead (CEL spec specifies
//! how to convert from JSON to CEL types).
//!
//! The basic example of how to use:
//! ```
//! use rscel::{CelContext, ExecContext, serde_json};
//!
//! let mut ctx = CelContext::new();
//! let mut exec_ctx = ExecContext::new();
//!
//! ctx.add_program_str("main", "foo + 3").unwrap();
//! exec_ctx.bind_param("foo", 3.into()); // convert to serde_json::Value
//!
//! let res = ctx.exec("main", &exec_ctx).unwrap(); // ValueCell::Int(6)
//! assert!(TryInto::<i64>::try_into(res).unwrap() == 6);
//! ```
mod context;
mod parser;
mod program;
mod value_cell;

pub use context::{CelContext, ExecContext, ExecError, ExecResult, RsCellFunction, RsCellMacro};
pub use program::{Program, ProgramError};
pub use value_cell::{ValueCell, ValueCellError, ValueCellResult};

pub use serde;
pub use serde_json;

pub use serde_json::Value;

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

    #[test]
    fn test_contains() {
        let mut ctx = CelContext::new();
        let exec_ctx = ExecContext::new();

        ctx.add_program_str("main", "\"hello there\".contains(\"hello\")")
            .unwrap();

        let _res = ctx.exec("main", &exec_ctx).unwrap();
    }
}
