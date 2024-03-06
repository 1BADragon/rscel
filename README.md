# rscel

RsCel is a CEL evaluator written in Rust. CEL is a google project that
describes a turing-incomplete language that can be used to evaluate
a user provdided expression. The language specification can be found
[here](https://github.com/google/cel-spec/blob/master/doc/langdef.md).

The design goals of this project were are as follows:
  * Flexible enough to allow for a user to bend the spec if needed
  * Sandbox'ed in such a way that only specific values can be bound
  * Can be used as a wasm depenedency (or other ffi)

The basic example of how to use:
```rust
use rscel::{CelContext, BindContext};

let mut ctx = CelContext::new();
let mut exec_ctx = BindContext::new();

ctx.add_program_str("main", "foo + 3").unwrap();
exec_ctx.bind_param("foo", 3.into()); // convert to CelValue

let res = ctx.exec("main", &exec_ctx).unwrap(); // CelValue::Int(6)
assert_eq!(res, 6.into());
```

As of 0.10.0 binding protobuf messages from the protobuf crate is now available! Given 
the following protobuf message:
```protobuf

message Point {
  int32 x = 1;
  int32 y = 2;
}
  
```
The following code can be used to evaluate a CEL expression on a Point message:

```rust
use rscel::{CelContext, BindContext};

// currently rscel required protobuf messages to be in a box 
let p = Box::new(protos::Point::new());
p.x = 4;
p.y = 5;

let mut ctx = CelContext::new();
let mut exec_ctx = BindContext::new();

ctx.add_program_str("main", "p.x + 3").unwrap();
exec_ctx.bind_protobuf_msg("p", p);

assert_eq!(ctx.exec("main", &exec_ctx), 7.into());
  
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
