use super::helpers;
use crate::interp::Interpreter;
use crate::types::CelByteCode;
use crate::utils::eval_ident;
use crate::{CelError, CelValue, CelValueDyn};

pub fn exists_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if bytecode.len() != 2 {
        return CelValue::from_err(CelError::argument(
            "exists() macro expects exactly 2 arguments",
        ));
    }

    let ident_name = match eval_ident(bytecode[0]) {
        Ok(name) => name,
        Err(err) => return err.into(),
    };

    match this {
        CelValue::List(list) => {
            let (cel, mut bindings) = helpers::setup_context(ctx);

            for value in list.into_iter() {
                bindings.bind_param(&ident_name, value.clone());
                let interp = Interpreter::new(&cel, &bindings);

                let res = match interp.run_raw(bytecode[1], true) {
                    Ok(val) => val,
                    Err(err) => return err.into(),
                };

                if res.is_truthy() {
                    return true.into();
                }
            }

            false.into()
        }
        _ => CelValue::from_err(CelError::value("exists() only available on list")),
    }
}
