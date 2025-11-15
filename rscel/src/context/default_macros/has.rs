use crate::interp::Interpreter;
use crate::types::CelByteCode;
use crate::{CelError, CelValue};

pub fn has_impl(ctx: &Interpreter, _this: CelValue, exprlist: &[&CelByteCode]) -> CelValue {
    if exprlist.len() != 1 {
        return CelValue::from_err(CelError::argument("has() macro expects exactly 1 argument"));
    }

    match ctx.run_raw(&exprlist[0], true) {
        Ok(_) => CelValue::true_(),
        Err(err) => match err {
            CelError::Binding { .. } | CelError::Attribute { .. } => CelValue::false_(),
            other => CelValue::from_err(other),
        },
    }
}
