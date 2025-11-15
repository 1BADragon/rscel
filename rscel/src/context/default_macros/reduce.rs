use super::helpers;
use crate::interp::Interpreter;
use crate::types::CelByteCode;
use crate::utils::eval_ident;
use crate::{CelError, CelValue};

// reduce [].reduce(curr, next, expression, starting)
pub fn reduce_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if bytecode.len() != 4 {
        return CelValue::from_err(CelError::argument("reduce() macro expects 4 arguments"));
    }

    let curr_name = match eval_ident(bytecode[0]) {
        Ok(name) => name,
        Err(err) => return err.into(),
    };
    let next_name = match eval_ident(bytecode[1]) {
        Ok(name) => name,
        Err(err) => return err.into(),
    };

    let mut cur_value = match ctx.run_raw(bytecode[3], true) {
        Ok(val) => val,
        Err(err) => return err.into(),
    };

    match this {
        CelValue::List(list) => {
            let (cel, mut bindings) = helpers::setup_context(ctx);

            for next in list.into_iter() {
                bindings.bind_param(&next_name, next);
                bindings.bind_param(&curr_name, cur_value);

                let interp = Interpreter::new(&cel, &bindings);
                cur_value = match interp.run_raw(bytecode[2], true) {
                    Ok(val) => val,
                    Err(err) => return err.into(),
                };
            }

            cur_value
        }
        _ => CelValue::from_err(CelError::value("reduce() only availble on list")),
    }
}
