use crate::{BindContext, CelContext, CelValue};
use std::assert;
use test_case::test_case;

#[test_case("3 + 2.1", 5.1; "type prop: int plus float")]
#[test_case("2.1 + 3", 5.1; "type prop: float plus int")]
#[test_case("3u + 2.1", 5.1; "type prop; unsigned plus float")]
#[test_case("2.1 + 3u", 5.1; "type prop; float plus unsigned")]
#[test_case("3 * 2.1", 6.3; "type prop: int times float")]
#[test_case("2.1 * 3", 6.3; "type prop: float times int")]
#[test_case("3u * 2.1", 6.3; "type prop; unsigned times float")]
#[test_case("2.1 * 3u", 6.3; "type prop; float times unsigned")]
#[test_case("3 - 2.1", 0.9; "type prop: int minus float")]
#[test_case("2.1 - 3", -0.9; "type prop: float minus int")]
#[test_case("3u - 2.1", 0.9; "type prop; unsigned minus float")]
#[test_case("2.1 - 3u", -0.9; "type prop; float minus unsigned")]
#[test_case("4 / 2.0", 2.0; "type prop: int div float")]
#[test_case("4.0 / 2", 2.0; "type prop: float div int")]
#[test_case("4u / 2.0", 2.0; "type prop; unsigned div float")]
#[test_case("4.0 - 2u", 2.0; "type prop; float div unsigned")]
fn test_equation_float(prog: &str, res: f64) {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", prog).unwrap();

    let eval_res = ctx.exec("main", &exec_ctx);
    if cfg!(feature = "type_prop") {
        if let CelValue::Float(f) = eval_res.unwrap() {
            assert!((f - res) < 0.00001);
        } else {
            panic!("Invalid type, expected float")
        }
    } else {
        assert!(eval_res.is_err());
    }
}

#[test_case("2 + 1u", 3.into(); "type prop: int plus unsigned")]
#[test_case("1u + 2", 3.into(); "type prop: unsigned plus ing")]
#[test_case("1 && 1", true.into(); "type prop: int and int")]
#[test_case("0 && 1u", false.into(); "type prop: int and unsigned")]
#[test_case("0 || 1u", true.into(); "type prop: int or unsigned")]
#[test_case("0 || 0", false.into(); "type prop: int or int")]
#[test_case("0 ? 1 : 2", 2.into(); "type prop: int as bool")]
#[test_case("\"\" ? 1 : 2", 2.into(); "type prop: empty string as bool")]
#[test_case("[] ? 1 : 2", 2.into(); "type prop: empty list as bool")]
#[test_case("{} ? 1 : 2", 2.into(); "type prop: empty obj as bool")]
#[test_case("\"1\" ? 1 : 2", 1.into(); "type prop: full string as bool")]
#[test_case("[1] ? 1 : 2", 1.into(); "type prop: full list as bool")]
#[test_case("{\"foo\": 1} ? 1 : 2", 1.into(); "type prop: full obj as bool")]
#[test_case("5 + (5 == 10 || 10 - 5 == 5) && false", false.into(); "Complex mixed precedence")]
fn test_equation(prog: &str, res: CelValue) {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", prog).unwrap();

    let eval_res = ctx.exec("main", &exec_ctx);
    println!("{:?}", eval_res);
    if cfg!(feature = "type_prop") {
        assert_eq!(eval_res.unwrap(), res);
    } else {
        assert!(eval_res.is_err());
    }
}
