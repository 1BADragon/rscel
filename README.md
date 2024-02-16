# rscel

RsCel is a CEL evaluator written in Rust. CEL is a google project that
describes a turing-incomplete language that can be used to evaluate
a user provdided expression. The language specification can be found
[here](https://github.com/google/cel-spec/blob/master/doc/langdef.md).

The design goals of this project were are as follows:
  * Flexible enough to allow for a user to bend the spec if needed
  * Sandbox'ed in such a way that only specific values can be bound
  * Can be used as a wasm depenedency (or other ffi)

While Google's CEL spec was designed around the protobuf message,
I decided to focus on using the JSON message instead (CEL spec specifies
how to convert from JSON to CEL types).

The basic example of how to use:
```rust
use rscel::{CelContext, BindContext, serde_json};

let mut ctx = CelContext::new();
let mut exec_ctx = BindContext::new();

ctx.add_program_str("main", "foo + 3").unwrap();
exec_ctx.bind_param("foo", 3.into()); // convert to CelValue

let res = ctx.exec("main", &exec_ctx).unwrap(); // CelValue::Int(6)
assert!(TryInto::<i64>::try_into(res).unwrap() == 6);
```

## Current Benchmark Times
| Bench Run                | Time           |
|--------------------------|----------------|
|Run One No Binding        | 0.000070054S   |
|Run Many No Binding       | 0.003451698S   |
|Run One With Binding      | 0.000015177S   |
|Run Many With Bindings    | 0.004443471S   |
|Build Many                | 0.006989954S   |
|Build Many With Bindings  | 0.084859163S   |

Build status: [![Rust](https://github.com/1BADragon/rscel/actions/workflows/rust.yml/badge.svg)](https://github.com/1BADragon/rscel/actions/workflows/rust.yml)
