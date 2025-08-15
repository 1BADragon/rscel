use crate::{CelContext, CelValue, SqlCompiler, SqlError, SqlFragment};

fn compile_sql(expr: &str) -> Result<SqlFragment, SqlError> {
    let mut ctx = CelContext::new();
    ctx.add_program_str("test", expr).unwrap();
    let prog = ctx.get_program("test").unwrap();
    let ast = prog.details().ast().unwrap();
    SqlCompiler::compile(ast)
}

#[test]
fn sql_abs_function() {
    let frag = compile_sql("abs(1)").unwrap();
    assert_eq!(frag.sql, "ABS($1)");
    assert_eq!(frag.params, vec![CelValue::from_int(1)]);
}

#[test]
fn sql_contains_function() {
    let frag = compile_sql("contains('foobar', 'oba')").unwrap();
    assert_eq!(frag.sql, "($1 LIKE '%' || $2 || '%')");
    assert_eq!(frag.params[0], CelValue::from_string("foobar".to_string()));
    assert_eq!(frag.params[1], CelValue::from_string("oba".to_string()));
}

#[test]
fn sql_get_full_year_function() {
    let frag = compile_sql("getFullYear(ts)").unwrap();
    assert_eq!(frag.sql, "EXTRACT(YEAR FROM ts)");
    assert!(frag.params.is_empty());
}

#[test]
fn sql_unsupported_function_error() {
    let err = compile_sql("unknownFunc(1)").unwrap_err();
    match err {
        SqlError::UnsupportedFeature(_) => {}
        _ => panic!("unexpected error type"),
    }
}

#[test]
fn sql_invalid_property_access_error() {
    let err = compile_sql("'str'.foo").unwrap_err();
    match err {
        SqlError::InvalidType(_) => {}
        _ => panic!("unexpected error type"),
    }
}
