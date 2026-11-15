use super::helpers;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue};

// reduce [].reduce(curr, next, expression, starting)
pub fn reduce_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 4 {
        return CelValue::from_err(CelError::argument("reduce() macro expects 4 arguments"));
    }

    let curr_name = match args[0].as_binder() {
        Ok(s) => s.to_owned(),
        Err(e) => return e.into(),
    };
    let next_name = match args[1].as_binder() {
        Ok(s) => s.to_owned(),
        Err(e) => return e.into(),
    };

    let mut cur_value = match args[3].eval(ctx) {
        Ok(val) => val,
        Err(err) => return err.into(),
    };

    match this {
        CelValue::List(list) => {
            let (cel, mut bindings) = helpers::child_scope(ctx);

            for next in list.into_iter() {
                bindings.bind_param(&next_name, next);
                bindings.bind_param(&curr_name, cur_value);

                let interp = Interpreter::new(&cel, &bindings);
                cur_value = match args[2].eval(&interp) {
                    Ok(val) => val,
                    Err(err) => return err.into(),
                };
            }

            cur_value
        }
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("reduce() only availble on list")),
    }
}
