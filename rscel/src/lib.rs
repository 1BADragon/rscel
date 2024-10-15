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
//! assert_eq!(res, 6.into());
//! ```
//! As of 0.10.0 binding protobuf messages from the protobuf crate is now available! Given
//! the following protobuf message:
//! ```protobuf
//!
//! message Point {
//!   int32 x = 1;
//!   int32 y = 2;
//! }
//!
//! ```
//! The following code can be used to evaluate a CEL expression on a Point message:
//!
//! ```ignore
//! use rscel::{CelContext, BindContext};
//!
//! // currently rscel required protobuf messages to be in a box
//! let p = Box::new(protos::Point::new());
//! p.x = 4;
//! p.y = 5;
//!
//! let mut ctx = CelContext::new();
//! let mut exec_ctx = BindContext::new();
//!
//! ctx.add_program_str("main", "p.x + 3").unwrap();
//! exec_ctx.bind_protobuf_msg("p", p);
//!
//! assert_eq!(ctx.exec("main", &exec_ctx).unwrap(), 7.into());
//!
//! ```
mod compiler;
mod context;
mod interp;
mod program;
mod types;

// Export some public interface
pub mod utils;
pub use compiler::{
    ast_node::AstNode, compiler::CelCompiler, grammar::*, source_location::SourceLocation,
    source_range::SourceRange, string_tokenizer::StringTokenizer, tokenizer::Tokenizer,
};
pub use context::{BindContext, CelContext, RsCelFunction, RsCelMacro};
pub use interp::ByteCode;
pub use program::{Program, ProgramDetails};
pub use types::{CelError, CelResult, CelValue, CelValueDyn};

// Some re-exports to allow a consistent use of serde
pub use serde;
pub use serde_json;

#[cfg(test)]
mod tests;
