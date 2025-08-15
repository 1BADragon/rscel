# rscel

This crate provides the core implementation of the rscel CEL interpreter.

## SQL compilation

The `SqlCompiler` converts CEL abstract syntax trees (ASTs) into SQL fragments that can be executed in PostgreSQL.

```rust
use rscel::{Program, SqlCompiler, CelValue};

let program = Program::from_source("1 + 2").unwrap();
let ast = program.ast().unwrap();
let frag = SqlCompiler::compile(ast).unwrap();

assert_eq!(frag.sql, "($1 + $2)");
assert_eq!(frag.params, vec![CelValue::from_int(1), CelValue::from_int(2)]);
```

This produces:

```
SQL: ($1 + $2)
Params: [CelValue::from_int(1), CelValue::from_int(2)]
```

## Testing

Run the full test suite before committing:

```bash
cargo +nightly-2025-08-08 test
cargo +nightly-2025-08-08 test --no-default-features
```

