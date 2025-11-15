use super::helpers;
use crate::interp::Interpreter;
use crate::types::CelByteCode;
use crate::utils::eval_ident;
use crate::{CelError, CelValue, CelValueDyn};

pub fn map_impl(ctx: &Interpreter, this: CelValue, bytecode: &[&CelByteCode]) -> CelValue {
    if !(bytecode.len() == 2 || bytecode.len() == 3) {
        return CelValue::from_err(CelError::argument(
            "map() macro expects exactly 2 or 3 arguments",
        ));
    }

    let ident_name = match eval_ident(bytecode[0]) {
        Ok(name) => name,
        Err(err) => return err.into(),
    };

    match this {
        CelValue::List(list) => map_list(ctx, list, &ident_name, bytecode),
        CelValue::Map(map) => map_map(ctx, map, &ident_name, bytecode),
        _ => CelValue::from_err(CelError::value("map() only available on list")),
    }
}

fn map_list(
    ctx: &Interpreter,
    list: Vec<CelValue>,
    ident_name: &str,
    bytecode: &[&CelByteCode],
) -> CelValue {
    let (cel, mut bindings) = helpers::setup_context(ctx);
    let mut mapped = Vec::new();

    for value in list.into_iter() {
        bindings.bind_param(ident_name, value.clone());
        let interp = Interpreter::new(&cel, &bindings);

        if bytecode.len() == 2 {
            match interp.run_raw(bytecode[1], true) {
                Ok(val) => mapped.push(val),
                Err(err) => return err.into(),
            }
        } else {
            let predicate = match interp.run_raw(bytecode[1], true) {
                Ok(val) => val,
                Err(err) => return err.into(),
            };

            if predicate.is_truthy() {
                match interp.run_raw(bytecode[2], true) {
                    Ok(val) => mapped.push(val),
                    Err(err) => return err.into(),
                }
            }
        }
    }

    mapped.into()
}

fn map_map(
    ctx: &Interpreter,
    map: std::collections::HashMap<String, CelValue>,
    ident_name: &str,
    bytecode: &[&CelByteCode],
) -> CelValue {
    let (cel, mut bindings) = helpers::setup_context(ctx);
    let mut mapped = Vec::new();

    for key in map.into_keys() {
        let value: CelValue = key.into();
        bindings.bind_param(ident_name, value.clone());
        let interp = Interpreter::new(&cel, &bindings);

        if bytecode.len() == 2 {
            match interp.run_raw(bytecode[1], true) {
                Ok(val) => mapped.push(val),
                Err(err) => return err.into(),
            }
        } else {
            let predicate = match interp.run_raw(bytecode[1], true) {
                Ok(val) => val,
                Err(err) => return err.into(),
            };

            if predicate.is_truthy() {
                match interp.run_raw(bytecode[2], true) {
                    Ok(val) => mapped.push(val),
                    Err(err) => return err.into(),
                }
            }
        }
    }

    mapped.into()
}
