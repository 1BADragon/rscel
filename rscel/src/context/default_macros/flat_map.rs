use crate::context::bound_predicate_executor::BoundPredicateExecutor;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue};

pub fn flat_map_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "flatMap() macro expects exactly 2 arguments",
        ));
    }

    match this {
        CelValue::List(list) => {
            let mut p = match BoundPredicateExecutor::new(&args[0], &args[1], ctx) {
                Ok(p) => p,
                Err(e) => return e.into(),
            };
            let mut out = Vec::new();

            for value in list.into_iter() {
                let mapped = match p.eval(value) {
                    Ok(val) => val,
                    Err(err) => return err.into(),
                };

                match mapped {
                    CelValue::List(inner) => out.extend(inner),
                    other => out.push(other),
                }
            }

            out.into()
        }
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("flatMap() only available on list")),
    }
}
