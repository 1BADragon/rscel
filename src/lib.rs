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
//! exec_ctx.bind_param("foo", 3.into()); // 3 converted to CelValue
//!
//! let res = ctx.exec("main", &exec_ctx).unwrap(); // CelValue::Int(6)
//! assert!(TryInto::<i64>::try_into(res).unwrap() == 6);
//! ```

#![cfg_attr(feature = "python", feature(fn_traits))]
#![cfg_attr(feature = "python", feature(unboxed_closures))]
mod cel_error;
mod cel_value;
mod compiler;
mod context;
mod interp;
mod program;

// Export some public interface
pub mod utils;
pub use cel_error::{CelError, CelResult};
pub use cel_value::CelValue;
pub use compiler::{
    compiler::CelCompiler, string_tokenizer::StringTokenizer, tokenizer::Tokenizer,
};
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
mod tests;
