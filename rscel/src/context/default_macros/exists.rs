use super::helpers;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue, CelValueDyn};

pub fn exists_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "exists() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match args[0].as_binder() {
        Ok(s) => s.to_owned(),
        Err(e) => return e.into(),
    };

    match this {
        CelValue::List(list) => {
            let (cel, mut bindings) = helpers::child_scope(ctx);

            for value in list.into_iter() {
                bindings.bind_param(&ident_name, value.clone());
                let interp = Interpreter::new(&cel, &bindings);

                let res = match args[1].eval(&interp) {
                    Ok(val) => val,
                    Err(err) => return err.into(),
                };

                if res.is_truthy() {
                    return true.into();
                }
            }

            false.into()
        }
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("exists() only available on list")),
    }
}
