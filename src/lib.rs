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
//! use rscel::{CelContext, BindContext, serde_json};
//!
//! let mut ctx = CelContext::new();
//! let mut exec_ctx = BindContext::new();
//!
//! ctx.add_program_str("main", "foo + 3").unwrap();
//! exec_ctx.bind_param("foo", 3.into()); // 3 converted to ValueCell
//!
//! let res = ctx.exec("main", &exec_ctx).unwrap(); // ValueCell::Int(6)
//! assert!(TryInto::<i64>::try_into(res).unwrap() == 6);
//! ```
mod ast;
mod cel_error;
mod cel_value;
mod context;
mod interp;
mod program;

// Export some public interface
pub mod utils;
pub use cel_error::{CelError, CelResult};
pub use cel_value::{CelValue, CelValueInner};
pub use context::{BindContext, CelContext, RsCelFunction, RsCelMacro};
pub use interp::ByteCode;
pub use program::Program;

// If any of the binding featurs are enabled, export them
#[cfg(any(feature = "python", feature = "wasm"))]
pub mod bindings;

// Some re-exports to allow a consistent use of serde
pub use serde;
pub use serde_json;

#[cfg(feature = "python")]
pub use bindings::python::*;

#[cfg(feature = "wasm")]
pub use bindings::wasm::*;

#[cfg(test)]
mod test {
    use crate::{BindContext, CelContext, CelValue, Program};
    use chrono::DateTime;
    use std::{assert, assert_eq, collections::HashMap};
    use test_case::test_case;

    #[test]
    fn test_bad_func_call() {
        let mut ctx = CelContext::new();
        let exec_ctx = BindContext::new();

        ctx.add_program_str("main", "foo(3)").unwrap();

        let res = ctx.exec("main", &exec_ctx);
        assert!(res.is_err());
    }

    #[test]
    fn test_contains() {
        let mut ctx = CelContext::new();
        let exec_ctx = BindContext::new();

        ctx.add_program_str("main", "\"hello there\".contains(\"hello\")")
            .unwrap();

        let _res = ctx.exec("main", &exec_ctx).unwrap();
    }

    #[test_case("3+3", 6.into(); "add signed")]
    #[test_case("4-3", 1.into(); "sub signed")]
    #[test_case("4u + 3u", 7u64.into(); "add unsigned")]
    #[test_case("7 % 2", 1.into(); "test mod")]
    #[test_case("(4+2) * (6-5)", 6.into(); "test parens")]
    #[test_case("[1, 2, 3].map(x, x+2)", CelValue::from_list(vec![3.into(), 4.into(), 5.into()]); "test map")]
    #[test_case("[1,2,3][1]", 2.into(); "array index")]
    #[test_case("{\"foo\": 3}.foo", 3.into(); "obj dot access")]
    #[test_case("size([1,2,3,4])", 4u64.into(); "test list size")]
    #[test_case("true || false", true.into(); "or")]
    #[test_case("false && true", false.into(); "and falsy")]
    #[test_case("true && true", true.into(); "and true")]
    #[test_case("[1,2].map(x, x+1).map(x, x*2)", CelValue::from_list(vec![4.into(), 6.into()]); "double map")]
    #[test_case("\"hello world\".contains(\"hello\")", true.into(); "test contains")]
    #[test_case("\"hello world\".endsWith(\"world\")", true.into(); "test endsWith")]
    #[test_case("\"hello world\".startsWith(\"hello\")", true.into(); "test startsWith")]
    #[test_case("\"abc123\".matches(\"[a-z]{3}[0-9]{3}\")", true.into(); "test matches")]
    #[test_case("string(1)", "1".into(); "test string")]
    #[test_case("type(1)", CelValue::from_type("int"); "test type")]
    #[test_case("4 > 5", false.into(); "test gt")]
    #[test_case("4 < 5", true.into(); "test lt")]
    #[test_case("4 >= 4", true.into(); "test ge")]
    #[test_case("5 <= 4", false.into(); "test le")]
    #[test_case("5 == 5", true.into(); "test eq")]
    #[test_case("5 != 5", false.into(); "test ne")]
    #[test_case("3 in [1,2,3,4,5]", true.into(); "test in")]
    #[test_case(r#"has({"foo": 3}.foo)"#, true.into(); "test has")]
    #[test_case("[1,2,3,4].all(x, x < 5)", true.into(); "test all true")]
    #[test_case("[1,2,3,4,5].all(x, x < 5)", false.into(); "test all false")]
    #[test_case("[1,2,3,4].exists(x, x < 3)", true.into(); "test exists true")]
    #[test_case("[1,2,3,4].exists(x, x == 5)", false.into(); "test exists false")]
    #[test_case("[1,2,3,4].exists_one(x, x == 4)", true.into(); "test exists one true")]
    #[test_case("[1,2,3,4].exists_one(x, x == 5)", false.into(); "test exists one false")]
    #[test_case("[1,2,3,4].filter(x, x % 2 == 0)", CelValue::from_list(vec![2.into(), 4.into()]); "test filter")]
    #[test_case("abs(-9)", 9.into(); "abs")]
    #[test_case("sqrt(9.0)", 3.0.into(); "sqrt")]
    #[test_case("pow(2, 2)", 4.into(); "pow")]
    #[test_case("pow(2.0, 2)", 4.0.into(); "pow2")]
    #[test_case("log(1)", 0u64.into(); "log")]
    #[test_case("ceil(2.3)", 3.into(); "ceil")]
    #[test_case("floor(2.7)", 2.into(); "floor")]
    #[test_case("round(2.2)", 2.into(); "round down")]
    #[test_case("round(2.5)", 3.into(); "round up")]
    #[test_case("min(1,2,3)", 1.into(); "min")]
    #[test_case("max(1,2,3)", 3.into(); "max")]
    fn test_equation(prog: &str, res: CelValue) {
        let mut ctx = CelContext::new();
        let exec_ctx = BindContext::new();

        ctx.add_program_str("main", prog).unwrap();

        let eval_res = ctx.exec("main", &exec_ctx).unwrap();
        assert_eq!(eval_res, res);
    }

    #[test]
    fn test_timestamp() {
        let mut ctx = CelContext::new();
        let exec_ctx = BindContext::new();

        ctx.add_program_str("main", r#"timestamp("2023-04-20T12:00:00Z")"#)
            .unwrap();
        let eval_res = ctx.exec("main", &exec_ctx).unwrap();

        let dt = DateTime::parse_from_rfc3339("2023-04-20T12:00:00Z").unwrap();
        assert_eq!(eval_res, dt.into());
    }

    #[test]
    fn test_timeduration() {
        let mut ctx = CelContext::new();
        let exec_ctx = BindContext::new();

        ctx.add_program_str(
            "main",
            r#"timestamp("2023-04-20T12:00:00Z") + duration("1h")"#,
        )
        .unwrap();
        let eval_res = ctx.exec("main", &exec_ctx).unwrap();

        let dt = DateTime::parse_from_rfc3339("2023-04-20T13:00:00Z").unwrap();
        assert_eq!(eval_res, dt.into());
    }

    #[test]
    fn test_binding() {
        let mut ctx = CelContext::new();
        let mut binding = BindContext::new();

        ctx.add_program_str("main", "foo + 9").unwrap();

        binding.bind_param("foo", 3.into());
        assert_eq!(ctx.exec("main", &binding).unwrap(), 12.into());
    }

    #[test]
    fn test_dict_binding() {
        let mut ctx = CelContext::new();
        let mut exec_ctx = BindContext::new();

        ctx.add_program_str("func1", "foo.bar + 4").unwrap();
        ctx.add_program_str("func2", "foo.bar % 4").unwrap();

        let mut foo: HashMap<String, CelValue> = HashMap::new();
        foo.insert("bar".to_owned(), 7.into());
        exec_ctx.bind_param("foo", foo.into());

        assert_eq!(ctx.exec("func1", &exec_ctx).unwrap(), 11.into());
        assert_eq!(ctx.exec("func2", &exec_ctx).unwrap(), 3.into());
    }

    #[test]
    fn test_serialization() {
        let json_str = {
            let prog = Program::from_source_nocache("4+7*2").unwrap();
            serde_json::to_string(&prog).unwrap()
        };

        let prog: Program = serde_json::from_str(&json_str).unwrap();

        let mut cel = CelContext::new();
        cel.add_program("main", prog);
        let bindings = BindContext::new();

        assert_eq!(cel.exec("main", &bindings).unwrap(), 18.into());
    }

    #[test]
    fn test_nested() {
        let mut ctx = CelContext::new();
        let mut exec_ctx = BindContext::new();

        ctx.add_program_str("foo", "val + 3").unwrap();
        ctx.add_program_str("bar", "foo * 3").unwrap();

        exec_ctx.bind_param("val", 7.into());

        assert_eq!(ctx.exec("bar", &exec_ctx).unwrap(), 30.into());
    }

    #[test]
    fn test_call_depth_failure() {
        let mut ctx = CelContext::new();
        let exec = BindContext::new();

        ctx.add_program_str("entry", "entry + 3").unwrap();

        assert!(ctx.exec("entry", &exec).is_err());
    }

    #[test]
    fn test_binding_filter() {
        let prog = Program::from_source("foo + int(3)").unwrap();

        let mut dets = prog.details().clone();
        let bindings = BindContext::new();

        assert!(dets.params().contains(&"int"));
        assert!(dets.params().contains(&"int"));

        dets.filter_from_bindings(&bindings);

        assert!(!dets.params().contains(&"int"));
        assert!(dets.params().contains(&"foo"));
    }
}
