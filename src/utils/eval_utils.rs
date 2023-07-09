use crate::{interp::Interpreter, ByteCode, CelError, CelResult, CelValueInner};

pub fn eval_ident(prog: &[ByteCode]) -> CelResult<String> {
    let interp = Interpreter::empty();

    if let CelValueInner::Ident(ident) = interp.run_raw(prog)?.into_inner() {
        Ok(ident)
    } else {
        Err(CelError::with_msg("ident required"))
    }
}
