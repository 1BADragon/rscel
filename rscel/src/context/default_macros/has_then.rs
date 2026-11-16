use crate::context::default_macros::helpers;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue};

// Test if the access path exists and returns either false if it doesn't or allows the
// evaluation of an additional predicate without having to create large complex components.
//
// Example:
//         hasThen(foo.bar.baz, x, x > 300)
//   where x resolves to foo.bar.baz, if it exists.
pub fn has_then_impl(ctx: &Interpreter, _this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 3 {
        return CelValue::from_err(CelError::argument(
            "hasThen() macro expects exactly 3 arguments",
        ));
    }

    match args[0].eval(ctx) {
        Ok(value) => {
            let ident_name = match args[1].as_binder() {
                Ok(s) => s.to_owned(),
                Err(e) => return e.into(),
            };

            let (cel, mut bindings) = helpers::child_scope(ctx);
            bindings.bind_param(&ident_name, value);

            let interp = Interpreter::new(&cel, &bindings);
            args[2].eval(&interp).into()
        }
        Err(err) => match err {
            CelError::Binding { .. } | CelError::Attribute { .. } => CelValue::false_(),
            other => CelValue::from_err(other),
        },
    }
}
