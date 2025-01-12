use crate::{interp::Interpreter, types::CelByteCode, CelError, CelResult, CelValue};

pub fn eval_ident(prog: &CelByteCode) -> CelResult<String> {
    let interp = Interpreter::empty();

    if let CelValue::Ident(ident) = interp.run_raw(prog, false)? {
        Ok(ident)
    } else {
        Err(CelError::misc("ident required"))
    }
}
