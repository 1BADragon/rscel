use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue};

pub fn has_impl(ctx: &Interpreter, _this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 1 {
        return CelValue::from_err(CelError::argument("has() macro expects exactly 1 argument"));
    }

    match args[0].eval(ctx) {
        Ok(_) => CelValue::true_(),
        Err(err) => match err {
            CelError::Binding { .. } | CelError::Attribute { .. } => CelValue::false_(),
            other => CelValue::from_err(other),
        },
    }
}
