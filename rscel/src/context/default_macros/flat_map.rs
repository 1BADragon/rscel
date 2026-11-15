use super::helpers;
use crate::interp::Interpreter;
use crate::types::MacroArg;
use crate::{CelError, CelValue};

pub fn flat_map_impl(ctx: &Interpreter, this: CelValue, args: &[MacroArg]) -> CelValue {
    if args.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "flatMap() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match args[0].as_binder() {
        Ok(s) => s.to_owned(),
        Err(e) => return e.into(),
    };

    match this {
        CelValue::List(list) => {
            let (cel, mut bindings) = helpers::child_scope(ctx);
            let mut out = Vec::new();

            for value in list.into_iter() {
                bindings.bind_param(&ident_name, value);
                let interp = Interpreter::new(&cel, &bindings);

                let mapped = match args[1].eval(&interp) {
                    Ok(val) => val,
                    Err(err) => return err.into(),
                };

                match mapped {
                    CelValue::List(inner) => out.extend(inner),
                    other => out.push(other),
                }
            }

            out.into()
        }
        CelValue::Err(e) => CelValue::Err(e),
        _ => CelValue::from_err(CelError::value("flatMap() only available on list")),
    }
}
