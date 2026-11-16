use crate::{interp::Interpreter, CelError, CelValue, MacroArg};

pub fn has_any_impl(ctx: &Interpreter, _this: CelValue, args: &[MacroArg]) -> CelValue {
    for arg in args.iter() {
        match arg.eval(ctx) {
            Ok(_) => return CelValue::true_(),
            Err(err) => match err {
                CelError::Binding { .. } | CelError::Attribute { .. } => {}
                other => return CelValue::from_err(other),
            },
        }
    }

    CelValue::false_()
}
