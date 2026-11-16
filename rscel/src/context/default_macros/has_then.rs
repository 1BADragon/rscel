use crate::context::bound_predicate_executor::BoundPredicateExecutor;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue};

// Test if the access path exists and returns either false if it doesn't or allows the
// evaluation of an additional predicate without having to create large complex components.
//
// An additional 4th argument can be used in place of returning false.
//
// Example:
//         hasThen(foo.bar.baz, x, x > 300)
//   where x resolves to foo.bar.baz, if it exists.
pub fn has_then_impl(ctx: &Interpreter, _this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 3 && args.len() != 4 {
        return CelValue::from_err(CelError::argument(
            "hasThen() macro expects 3 or 4 arguments",
        ));
    }

    match args[0].eval(ctx) {
        Ok(value) => match BoundPredicateExecutor::new(&args[1], &args[2], ctx) {
            Ok(mut b) => b.eval(value).into(),
            Err(e) => e.into(),
        },
        Err(err) => match err {
            CelError::Binding { .. } | CelError::Attribute { .. } => {
                if args.len() == 3 {
                    CelValue::false_()
                } else {
                    args[3].eval(ctx).into()
                }
            }
            other => CelValue::from_err(other),
        },
    }
}
