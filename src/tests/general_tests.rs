use crate::{
    compiler::{compiler::CelCompiler, string_tokenizer::StringTokenizer},
    BindContext, CelContext, CelError, CelValue, Program,
};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use std::{assert, assert_eq, collections::HashMap, str::FromStr, sync::Arc};
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
#[test_case("4+2*6-5", 11.into(); "test op order")]
#[test_case("4-2+5*2", (12).into(); "test op order 2")]
#[test_case("[1, 2, 3].map(x, x+2)", CelValue::from_list(vec![3.into(), 4.into(), 5.into()]); "test map")]
#[test_case("[1, 2, 3].map(x, x % 2 == 1, x + 1)", CelValue::from_list(vec![2.into(), 4.into()]); "test map 2")]
#[test_case("[1,2,3][1]", 2.into(); "array index")]
#[test_case("{\"foo\": 3}.foo", 3.into(); "obj dot access")]
#[test_case("size([1,2,3,4])", 4u64.into(); "test list size")]
#[test_case("true || false", true.into(); "or")]
#[test_case("true || undefined", true.into(); "or shortcut")]
#[test_case("false && undefined", false.into(); "and shortcut")]
#[test_case("false && true", false.into(); "and falsy")]
#[test_case("true && true", true.into(); "and true")]
#[test_case("[1,2].map(x, x+1).map(x, x*2)", CelValue::from_list(vec![4.into(), 6.into()]); "double map")]
#[test_case("\"hello world\".contains(\"hello\")", true.into(); "test contains")]
#[test_case("\"hello world\".endsWith(\"world\")", true.into(); "test endsWith")]
#[test_case("\"hello world\".startsWith(\"hello\")", true.into(); "test startsWith")]
#[test_case("\"abc123\".matches(\"[a-z]{3}[0-9]{3}\")", true.into(); "test matches")]
#[test_case("string(1)", "1".into(); "test string")]
#[test_case("type(1)", CelValue::int_type(); "test type")]
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
#[test_case("[1,2,3].reduce(curr, next, curr + next, 0)", 6.into(); "reduce")]
#[test_case("{}", CelValue::from_map(HashMap::new()); "empty object")]
#[test_case("[]", CelValue::from_list(Vec::new()); "empy list")]
#[test_case("has(foo) && foo > 10", false.into(); "has works")]
#[test_case("true ? 4 : 3", 4.into(); "ternary true")]
#[test_case("false ? 4 : 3", 3.into(); "ternary false")]
#[test_case("bool(0)", false.into(); "bool zero")]
#[test_case("bool(1)", true.into(); "bool nonzero")]
#[test_case("bool(\"3\")", true.into(); "bool nonempty string")]
#[test_case("bool(\"\")", false.into(); "bool empty string")]
#[test_case("bool([])", false.into(); "bool empty list")]
#[test_case("bool([1])", true.into(); "bool nonempty list")]
#[test_case("bool(true)", true.into(); "bool true")]
#[test_case("bool(false)", false.into(); "bool false")]
#[test_case("bool(null)", false.into(); "bool null")]
#[test_case("bool({})", false.into(); "bool empty map")]
#[test_case("bool({\"foo\": 42})", true.into(); "bool nonempty map")]
#[test_case("2 * 4 * 8 * 72 / 144", 32.into(); "long multiply operation")]
#[test_case("2 * 3 + 7", 13.into(); "long mixed operation")]
#[test_case("true && false || true && true", true.into(); "long logic operation")]
#[test_case("2 + 3 - 1", 4.into(); "long add/sub operation")]
#[test_case("2 < 3 >= 1", true.into(); "type prop: chained cmp")]
#[test_case("3 * 2 - 1 / 4 * 2", 6.into(); "large op 2")]
#[test_case("true || unbound || unbound", true.into(); "Or short cut")]
#[test_case("true == true || false == true && false", true.into(); "Incorrect equality precedence")]
#[test_case("5 < 10 || 10 < 5 && false", true.into(); "Incorrect less-than precedence")]
#[test_case("true || false && false", true.into(); "Incorrect AND precedence")]
#[test_case("false && true || true", true.into(); "Incorrect OR precedence")]
#[test_case("5 + 5 == 10 || 10 - 5 == 5 && false", true.into(); "Incorrect addition precedence")]
#[test_case("6 / 2 == 3 || 2 * 3 == 6 && false", true.into(); "Incorrect division precedence")]
#[test_case("(true || false) && false", false.into(); "Incorrect parentheses precedence")]
#[test_case("'foo' in 'foot'", true.into(); "in string operator")]
#[test_case("'foot' in 'foo'", false.into(); "in string operator false")]
#[test_case("type(3) == type(3)", true.into(); "type eq")]
#[test_case("type(null) == null_type", true.into(); "null_type eq")]
#[test_case("type(3) == int", true.into(); "int type eq")]
#[test_case("type(3u) == uint", true.into(); "uint type eq")]
#[test_case("type('foo') == string", true.into(); "string type eq")]
#[test_case("type(true) == bool", true.into(); "bool type eq true")]
#[test_case("type(false) == bool", true.into(); "bool type eq false")]
#[test_case("type(3.2) == double", true.into(); "double type eq")]
#[test_case("type(3.2) == float", true.into(); "float type eq")]
#[test_case("type(true) == double", false.into(); "bool type neq")]
#[test_case("type(true) != double", true.into(); "bool type neq 2")]
#[test_case("type([1,2,3]) == type([])", true.into(); "list type neq")]
#[test_case("type({'foo': 3}) == type({})", true.into(); "map type neq")]
#[test_case("coalesce()", CelValue::from_null(); "coalesce none")]
#[test_case("coalesce(null, 3)", 3.into(); "coalesce explicit null")]
#[test_case("coalesce(foo, 4)", 4.into(); "coalesce unbound var")]
#[test_case("coalesce(1, 2, 3)", 1.into(); "coalesce first val ok")]
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
    ctx.add_program_str("func3", "foo.bar").unwrap();

    let mut foo: HashMap<String, CelValue> = HashMap::new();
    foo.insert("bar".to_owned(), 7.into());
    exec_ctx.bind_param("foo", foo.into());

    assert_eq!(ctx.exec("func1", &exec_ctx).unwrap(), 11.into());
    assert_eq!(ctx.exec("func2", &exec_ctx).unwrap(), 3.into());
    assert_eq!(ctx.exec("func3", &exec_ctx).unwrap(), 7.into());
}

#[test]
fn test_serialization() {
    let json_str = {
        let mut tokenizer = StringTokenizer::with_input("4+7*2");
        let prog = CelCompiler::with_tokenizer(&mut tokenizer)
            .compile()
            .unwrap();
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
fn test_has_through() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    ctx.add_program_str("entry", "has(foo) ? foo + 3 : 42")
        .unwrap();

    assert_eq!(ctx.exec("entry", &exec).unwrap(), 42.into());

    exec.bind_param("foo", 10.into());
    assert_eq!(ctx.exec("entry", &exec).unwrap(), 13.into());

    ctx.add_program_str("entry2", "has(a.b.c)").unwrap();
    assert_eq!(ctx.exec("entry2", &exec).unwrap(), false.into());

    let mut a = HashMap::<String, CelValue>::new();
    exec.bind_param("a", a.clone().into());
    assert_eq!(ctx.exec("entry2", &exec).unwrap(), false.into());

    let mut b = HashMap::<String, CelValue>::new();
    b.insert("c".to_string(), 4.into());
    a.insert("b".to_string(), b.into());
    exec.bind_param("a", a.into());
    assert_eq!(ctx.exec("entry2", &exec).unwrap(), true.into());
}

#[test]
fn test_object_access_in_array() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    ctx.add_program_str("entry", "my_list[0].foo").unwrap();

    let mut obj_map = HashMap::<String, CelValue>::new();
    obj_map.insert("foo".to_owned(), "value".into());

    let obj = CelValue::from_val_slice(&vec![obj_map.into()]);
    exec.bind_param("my_list", obj);

    assert_eq!(ctx.exec("entry", &exec).unwrap(), "value".into());
}

#[test]
fn test_has_in_reduce() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    ctx.add_program_str(
        "entry",
        "my_list.reduce(curr, next, curr + int(has(next.foo)), 0)",
    )
    .unwrap();

    let obj: CelValue = serde_json::from_str::<Value>("[{\"foo\": 1}, {}, {\"foo\": 1}]")
        .unwrap()
        .into();

    exec.bind_param("my_list", obj.into());

    assert_eq!(ctx.exec("entry", &exec).unwrap(), 2.into());
}

#[test]
#[allow(deprecated)]
fn test_timestamp_functions() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    let dt = Utc
        .ymd(2024, 01, 10)
        .and_hms_milli_opt(8, 57, 45, 123)
        .unwrap();
    exec.bind_param("time", CelValue::from_timestamp(&dt));

    let progs = [
        ("time.getDate()", 10),
        ("time.getDayOfMonth()", 9),
        ("time.getDayOfWeek()", 3),
        ("time.getDayOfYear()", 9),
        ("time.getFullYear()", 2024),
        ("time.getHours()", 8),
        ("time.getMilliseconds()", 123),
        ("time.getMinutes()", 57),
        ("time.getMonth()", 0),
        ("time.getSeconds()", 45),
    ];

    for prog in progs.iter() {
        ctx.add_program_str("entry", prog.0).unwrap();

        let res = ctx.exec("entry", &exec).unwrap();
        println!("{}:{} == {}", prog.0, res, prog.1);
        assert!(res == prog.1.into());
    }
}

#[test]
fn test_coalesce() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    exec.bind_params_from_json_obj(Value::from_str("{\"foo\": 4, \"bar\":{\"a\": 3}}").unwrap())
        .unwrap();

    ctx.add_program_str("prog1", "coalesce(foo, 3)").unwrap();
    ctx.add_program_str("prog2", "coalesce(bar.a, 4)").unwrap();
    ctx.add_program_str("prog3", "coalesce(bar.b, bar.a)")
        .unwrap();

    assert_eq!(ctx.exec("prog1", &exec).unwrap(), 4.into());
    assert_eq!(ctx.exec("prog2", &exec).unwrap(), 3.into());
    assert_eq!(ctx.exec("prog3", &exec).unwrap(), 3.into());
}

#[test]
fn test_dyn_value() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();
    let mut exec2 = BindContext::new();

    ctx.add_program_str("main", "foo.bar")
        .expect("Failed to compile prog");
    ctx.add_program_str("prog2", "foo[\"bar\"]")
        .expect("Failed to compile prog2");
    ctx.add_program_str("prog3", "e == e")
        .expect("Failed to compile prog 3");

    let mut inner_map = HashMap::new();
    inner_map.insert("bar".to_string(), 5.into());
    let foo = CelValue::from_dyn(Arc::new(CelValue::from_map(inner_map)));
    exec.bind_param("foo", foo);

    exec.bind_param("e", CelValue::from_dyn(Arc::new(CelValue::from_int(4))));

    let mut inner_map = HashMap::new();
    inner_map.insert("bar".to_string(), 5.into());
    let foo = CelValue::from_map(inner_map);

    exec2.bind_param("foo", foo);
    assert_eq!(ctx.exec("main", &exec).unwrap(), 5.into());
    assert_eq!(ctx.exec("prog2", &exec).unwrap(), 5.into());
    assert_eq!(ctx.exec("prog2", &exec2).unwrap(), 5.into());
    assert_eq!(ctx.exec("prog3", &exec).unwrap(), true.into());
}

#[test]
fn test_keywords_as_access_idents() {
    let mut ctx = CelContext::new();

    ctx.add_program_str("main", "foo.timestamp")
        .expect("Failed to compile program");

    let mut exec1 = BindContext::new();
    let mut map1 = HashMap::new();
    map1.insert("timestamp".to_string(), 4.into());
    exec1.bind_param("foo", map1.into());

    assert_eq!(ctx.exec("main", &exec1).unwrap(), 4.into());

    let mut exec2 = BindContext::new();
    let map2 = HashMap::new();
    exec2.bind_param("foo", map2.into());

    match ctx.exec("main", &exec2) {
        Err(CelError::Attribute { .. }) => {}
        _ => panic!(),
    }
}
