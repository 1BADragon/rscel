use crate::{
    context::bound_predicate_executor::BoundPredicateExecutor, interp::Interpreter, CelError,
    CelValue, MacroArg,
};

pub fn bind_impl(ctx: &Interpreter, _this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 3 {
        return CelValue::from_err(CelError::argument(
            "bind() macro expects exactly 3 arguments",
        ));
    }

    let value = match args[1].eval(ctx) {
        Ok(v) => v,
        Err(e) => return e.into(),
    };

    let mut p = match BoundPredicateExecutor::new(&args[0], &args[2], ctx) {
        Ok(p) => p,
        Err(e) => return e.into(),
    };

    p.eval(value).into()
}
