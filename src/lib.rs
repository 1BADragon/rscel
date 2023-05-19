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
mod ast;
mod context;
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
    use crate::{CelContext, ExecContext, ValueCell};
    use test_case::test_case;

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

    #[test_case("3+3", ValueCell::Int(6))]
    #[test_case("4-3", ValueCell::Int(1))]
    #[test_case("4u + 3u", 7u64.into())]
    #[test_case("7 % 2", ValueCell::Int(1))]
    #[test_case("(4+2) * (6-5)", ValueCell::Int(6))]
    #[test_case("[1, 2, 3].map(x, x+2)", ValueCell::List(vec![ValueCell::Int(3), ValueCell::Int(4), ValueCell::Int(5)]))]
    #[test_case("[1,2,3][1]", ValueCell::Int(2))]
    #[test_case("{\"foo\": 3}.foo", ValueCell::Int(3))]
    #[test_case("size([1,2,3,4])", ValueCell::UInt(4))]
    #[test_case("true || false", ValueCell::Bool(true))]
    #[test_case("false && true", ValueCell::Bool(false))]
    #[test_case("true && true", ValueCell::Bool(true))]
    #[test_case("[1,2].map(x, x+1).map(x, x*2)", ValueCell::List(vec![ValueCell::Int(4), ValueCell::Int(6)]))]
    #[test_case("\"hello world\".contains(\"hello\")", true.into(); "test contains")]
    #[test_case("\"hello world\".endsWith(\"world\")", true.into(); "test endsWith")]
    #[test_case("\"hello world\".startsWith(\"hello\")", true.into(); "test startsWith")]
    #[test_case("\"abc123\".matches(\"[a-z]{3}[0-9]{3}\")", true.into(); "test matches")]
    #[test_case("string(1)", "1".into(); "test string")]
    fn test_equation(prog: &str, res: ValueCell) {
        let mut ctx = CelContext::new();
        let exec_ctx = ExecContext::new();

        ctx.add_program_str("main", prog).unwrap();

        let eval_res = ctx.exec("main", &exec_ctx).unwrap();
        assert!(eval_res == res);
    }
}
