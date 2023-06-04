use crate::{ByteCode, ValueCellResult, interp::Interpreter, ValueCellInner, ValueCellError};

pub fn eval_ident(prog: &[ByteCode]) -> ValueCellResult<String> {
    let interp = Interpreter::empty();

    if let ValueCellInner::Ident(ident) = interp.run_raw(prog)?.into_inner() {
        Ok(ident)
    } else {
        Err(ValueCellError::with_msg("ident required"))
    }
}