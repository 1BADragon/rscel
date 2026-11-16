//! Compiled programs must survive a serialize/deserialize round trip in every format the
//! bindings expose (JSON) and still produce the same result.

use crate::{BindContext, CelContext, CelValue, Program};
use test_case::test_case;

fn bindings() -> BindContext<'static> {
    let mut bindings = BindContext::new();
    bindings.bind_param("x", CelValue::from_list(vec![1.into(), 2.into(), 3.into()]));
    bindings.bind_param("y", 2.into());
    bindings.bind_param("s", "hello".into());
    bindings
}

fn exec(prog: Program) -> CelValue {
    let mut ctx = CelContext::new();
    ctx.add_program("main", prog);
    ctx.exec("main", &bindings()).unwrap()
}

fn json_round_trip(prog: &Program) -> Program {
    let json = serde_json::to_string(prog).expect("program should serialize to json");
    serde_json::from_str(&json).expect("program should deserialize from json")
}

// Single-identifier call arguments compile to `CelValue::Binder`, so nearly every case here
// exercises it; "3 + 3" and "has(...)" are controls without one.
#[test_case("3 + 3"; "no binder")]
#[test_case("size(x)"; "function call ident arg")]
#[test_case("string(y)"; "conversion ident arg")]
#[test_case("max(y, 5)"; "multi arg call")]
#[test_case("s.startsWith('he')"; "method call")]
#[test_case("s.contains(string(y))"; "method call nested call arg")]
#[test_case("[1, 2].map(i, i * 2)"; "map macro")]
#[test_case("x.filter(v, v > 1)"; "filter macro")]
#[test_case("x.all(v, v > 0)"; "all macro")]
#[test_case("x.exists(v, v == y)"; "exists macro")]
#[test_case("x.exists_one(v, v == y)"; "exists_one macro")]
#[test_case("x.reduce(acc, v, acc + v, 0)"; "reduce macro")]
#[test_case("size(x.map(i, i + y))"; "nested macro in call")]
#[test_case("has({'a': 1}.a)"; "has macro")]
#[test_case("f'{y} {s}'"; "format string")]
fn test_program_serialization_round_trip(source: &str) {
    let prog = Program::from_source(source).unwrap();
    let expected = exec(prog.clone());

    assert_eq!(exec(json_round_trip(&prog)), expected, "json round trip");
}
