use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue};

pub fn coalesce_impl(ctx: &Interpreter, _this: CelValue, args: &[MacroArg]) -> CelValue {
    for arg in args.iter() {
        match arg.eval(ctx) {
            Ok(CelValue::Null) => {}
            Ok(val) => return val,
            Err(CelError::Binding { .. }) | Err(CelError::Attribute { .. }) => {}
            Err(err) => return CelValue::from_err(err),
        }
    }

    CelValue::from_null()
}
