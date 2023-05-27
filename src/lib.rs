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
pub mod bindings;
mod context;
mod program;
mod value_cell;

pub use context::{CelContext, ExecContext, ExecError, ExecResult, RsCellFunction, RsCellMacro};
pub use program::{Program, ProgramError};
pub use value_cell::{ValueCell, ValueCellError, ValueCellInner, ValueCellResult};

pub use serde;
pub use serde_json;

pub use serde_json::Value;

#[cfg(feature = "python")]
pub use bindings::python::*;

#[cfg(feature = "wasm")]
pub use bindings::wasm::*;

#[cfg(test)]
mod test {
    use std::collections::HashMap;

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

    #[test_case("3+3", 6.into(); "add signed")]
    #[test_case("4-3", 1.into(); "sub signed")]
    #[test_case("4u + 3u", 7u64.into(); "add unsigned")]
    #[test_case("7 % 2", 1.into(); "test mod")]
    #[test_case("(4+2) * (6-5)", 6.into(); "test parens")]
    #[test_case("[1, 2, 3].map(x, x+2)", ValueCell::from_list(&[3.into(), 4.into(), 5.into()]); "test map")]
    #[test_case("[1,2,3][1]", 2.into(); "array index")]
    #[test_case("{\"foo\": 3}.foo", 3.into(); "obj dot access")]
    #[test_case("size([1,2,3,4])", 4u64.into(); "test list size")]
    #[test_case("true || false", true.into(); "or")]
    #[test_case("false && true", false.into(); "and falsy")]
    #[test_case("true && true", true.into(); "and true")]
    #[test_case("[1,2].map(x, x+1).map(x, x*2)", ValueCell::from_list(&[4.into(), 6.into()]); "double map")]
    #[test_case("\"hello world\".contains(\"hello\")", true.into(); "test contains")]
    #[test_case("\"hello world\".endsWith(\"world\")", true.into(); "test endsWith")]
    #[test_case("\"hello world\".startsWith(\"hello\")", true.into(); "test startsWith")]
    #[test_case("\"abc123\".matches(\"[a-z]{3}[0-9]{3}\")", true.into(); "test matches")]
    #[test_case("string(1)", "1".into(); "test string")]
    #[test_case("type(1)", ValueCell::from_type("int"); "test type")]
    #[test_case("4 > 5", false.into(); "test gt")]
    #[test_case("4 < 5", true.into(); "test lt")]
    #[test_case("4 >= 4", true.into(); "test ge")]
    #[test_case("5 <= 4", false.into(); "test le")]
    #[test_case("5 == 5", true.into(); "test eq")]
    #[test_case("5 != 5", false.into(); "test ne")]
    #[test_case("3 in [1,2,3,4,5]", true.into(); "test in")]
    fn test_equation(prog: &str, res: ValueCell) {
        let mut ctx = CelContext::new();
        let exec_ctx = ExecContext::new();

        ctx.add_program_str("main", prog).unwrap();

        let eval_res = ctx.exec("main", &exec_ctx).unwrap();
        assert!(eval_res == res);
    }

    #[test]
    fn test_binding() {
        let mut ctx = CelContext::new();
        let mut exec_ctx = ExecContext::new();

        ctx.add_program_str("func1", "foo.bar + 4").unwrap();
        ctx.add_program_str("func2", "foo.bar % 4").unwrap();

        let mut foo: HashMap<String, ValueCell> = HashMap::new();
        foo.insert("bar".to_owned(), 7.into());
        exec_ctx.bind_param("foo", foo.into());

        assert!(ctx.exec("func1", &exec_ctx).unwrap() == 11.into());
        assert!(ctx.exec("func2", &exec_ctx).unwrap() == 3.into());
    }
}
