use crate::{interp::Interpreter, CelError, CelValue, MacroArg};

pub fn has_all_impl(ctx: &Interpreter, _this: CelValue, args: &[MacroArg]) -> CelValue {
    for arg in args.iter() {
        match arg.eval(ctx) {
            Ok(_) => {}
            Err(err) => match err {
                CelError::Binding { .. } | CelError::Attribute { .. } => return CelValue::false_(),
                other => return CelValue::from_err(other),
            },
        }
    }

    CelValue::true_()
}
