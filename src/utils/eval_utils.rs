use crate::{interp::Interpreter, ByteCode, CelError, CelResult, CelValue};

pub fn eval_ident(prog: &[ByteCode]) -> CelResult<String> {
    let interp = Interpreter::empty();

    if let CelValue::Ident(ident) = interp.run_raw(prog)? {
        Ok(ident)
    } else {
        Err(CelError::misc("ident required"))
    }
}
