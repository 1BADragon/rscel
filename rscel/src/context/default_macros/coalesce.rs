use crate::interp::Interpreter;
use crate::types::CelByteCode;
use crate::{CelError, CelValue};

pub fn coalesce_impl(ctx: &Interpreter, _this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    for arg in bytecode.iter() {
        match ctx.run_raw(arg, true) {
            Ok(CelValue::Null) => {}
            Ok(val) => return val,
            Err(CelError::Binding { .. }) | Err(CelError::Attribute { .. }) => {}
            Err(err) => return CelValue::from_err(err),
        }
    }

    CelValue::from_null()
}
