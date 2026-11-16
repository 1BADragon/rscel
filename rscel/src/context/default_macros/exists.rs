use crate::context::bound_predicate_executor::BoundPredicateExecutor;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue, CelValueDyn};

pub fn exists_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "exists() macro expects exactly 2 arguments",
        ));
    }

    match this {
        CelValue::List(list) => {
            let mut p = match BoundPredicateExecutor::new(&args[0], &args[1], ctx) {
                Ok(p) => p,
                Err(e) => return e.into(),
            };

            for value in list.into_iter() {
                let res = match p.eval(value.clone()) {
                    Ok(val) => val,
                    Err(err) => return err.into(),
                };

                if res.is_truthy() {
                    return true.into();
                }
            }

            false.into()
        }
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("exists() only available on list")),
    }
}
