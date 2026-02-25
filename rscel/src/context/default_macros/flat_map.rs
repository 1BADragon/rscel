use super::helpers;
use crate::interp::Interpreter;
use crate::types::CelByteCode;
use crate::utils::eval_ident;
use crate::{CelError, CelValue};

pub fn flat_map_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if bytecode.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "flatMap() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match eval_ident(bytecode[0]) {
        Ok(name) => name,
        Err(err) => return err.into(),
    };

    match this {
        CelValue::List(list) => {
            let (cel, mut bindings) = helpers::setup_context(ctx);
            let mut out = Vec::new();

            for value in list.into_iter() {
                bindings.bind_param(&ident_name, value);
                let interp = Interpreter::new(&cel, &bindings);

                let mapped = match interp.run_raw(bytecode[1], true) {
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
